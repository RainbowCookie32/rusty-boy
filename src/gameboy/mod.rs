mod cpu;
pub mod ppu;
pub mod memory;
pub mod disassembler;

use std::fmt;
use std::sync::{Arc, RwLock};
use std::sync::mpsc::Sender;

use cpu::GameboyCPU;
use ppu::GameboyPPU;

use memory::GameboyMemory;
use memory::cart::CartHeader;

pub struct Gameboy {
    gb_cyc: Arc<RwLock<usize>>,
    gb_cpu: Arc<RwLock<GameboyCPU>>,
    gb_ppu: Arc<RwLock<GameboyPPU>>,
    gb_mem: Arc<RwLock<GameboyMemory>>,
    gb_joy: Arc<RwLock<JoypadHandler>>,

    pub dbg_mode: EmulatorMode,
    pub dbg_do_step: bool,
    pub dbg_breakpoint_list: Vec<Breakpoint>
}

impl Gameboy {
    pub fn init(gb_mem: Arc<RwLock<GameboyMemory>>) -> Gameboy {
        let gb_cyc = Arc::new(RwLock::new(0));
        let gb_cpu = Arc::new(RwLock::new(GameboyCPU::init(gb_cyc.clone(), gb_mem.clone())));
        let gb_ppu = Arc::new(RwLock::new(GameboyPPU::init(gb_cyc.clone(), gb_mem.clone())));
        let gb_joy = gb_mem.read().unwrap().gb_joy();

        Gameboy {
            gb_cyc,
            gb_cpu,
            gb_ppu,
            gb_mem,
            gb_joy,

            dbg_mode: EmulatorMode::Paused,
            dbg_do_step: false,
            dbg_breakpoint_list: Vec::new()
        }
    }

    pub fn gb_start(gameboy: Arc<RwLock<Gameboy>>) -> Sender<()> {
        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            let exit_rx = rx;
            let gameboy = gameboy;
    
            loop {
                if let Ok(mut lock) = gameboy.try_write() {
                    if lock.dbg_mode == EmulatorMode::Running {
                        lock.gb_cpu_cycle();
                        lock.gb_ppu_cycle();
                    }
                    else if lock.dbg_mode == EmulatorMode::Stepping && lock.dbg_do_step {
                        lock.gb_cpu_cycle();
                        lock.gb_ppu_cycle();
                        lock.dbg_do_step = false;
                    }
                }

                if exit_rx.try_recv().is_ok() {
                    break;
                }
            }
        });

        tx
    }

    pub fn gb_reset(&mut self) {
        self.gb_cpu.write().unwrap().reset();
        self.gb_mem.write().unwrap().reset();

        if let Ok(mut cycles) = self.gb_cyc.write() {
            *cycles = 0;
        }

        self.dbg_mode = EmulatorMode::Paused;
    }

    pub fn gb_cpu_cycle(&mut self) {
        if let Ok(mut lock) = self.gb_cpu.write() {
            lock.cpu_cycle(&self.dbg_breakpoint_list, &mut self.dbg_mode);
        }
    }

    pub fn gb_ppu_cycle(&mut self) {
        if let Ok(mut lock) = self.gb_ppu.write() {
            lock.ppu_cycle();
        }
    }

    pub fn ui_get_header(&self) -> Arc<CartHeader> {
        self.gb_mem.read().unwrap().header()
    }

    pub fn ui_get_memory(&self) -> Arc<RwLock<GameboyMemory>> {
        self.gb_mem.clone()
    }

    pub fn ui_get_cpu_registers(&self) -> (u16, u16, u16, u16, u16, u16) {
        let lock = self.gb_cpu.read().unwrap();
        lock.get_all_registers()
    }

    pub fn ui_get_callstack(&self) -> Arc<RwLock<Vec<String>>> {
        self.gb_cpu.read().unwrap().get_callstack()
    }

    pub fn ui_get_serial_output(&self) -> Arc<RwLock<Vec<u8>>> {
        self.gb_mem.read().unwrap().serial_output()
    }

    pub fn ui_get_joypad_handler(&self) -> Arc<RwLock<JoypadHandler>> {
        self.gb_joy.clone()
    }

    pub fn ui_get_screen_data(&self) -> Arc<RwLock<Vec<u8>>> {
        self.gb_ppu.read().unwrap().get_screen_data()
    }

    pub fn ui_get_backgrounds_data(&self) -> Arc<RwLock<Vec<Vec<u8>>>> {
        self.gb_ppu.read().unwrap().get_backgrounds_data()
    }
}

#[derive(Default)]
pub struct JoypadHandler {
    value: u8,

    down_pressed: bool,
    up_pressed: bool,
    left_pressed: bool,
    right_pressed: bool,

    start_pressed: bool,
    select_pressed: bool,
    b_pressed: bool,
    a_pressed: bool
}

impl JoypadHandler {
    pub fn set_value(&mut self, value: u8) {
        self.value = value;
    }

    pub fn get_buttons(&self) -> u8 {
        let mut result = 0b11001111;

        if self.value == 0x20 {
            if self.down_pressed {
                result &= 0b11000111;
            }

            if self.up_pressed {
                result &= 0b11001011;
            }

            if self.left_pressed {
                result &= 0b11001101;
            }

            if self.right_pressed {
                result &= 0b11001110;
            }
        }
        else if self.value == 0x10 {
            if self.start_pressed {
                result &= 0b11000111;
            }

            if self.select_pressed {
                result &= 0b11001011;
            }

            if self.b_pressed {
                result &= 0b11001101;
            }

            if self.a_pressed {
                result &= 0b11001110;
            }
        }

        result
    }

    pub fn set_down_state(&mut self, state: bool) {
        self.down_pressed = state;
    }

    pub fn set_up_state(&mut self, state: bool) {
        self.up_pressed = state;
    }

    pub fn set_left_state(&mut self, state: bool) {
        self.left_pressed = state;
    }

    pub fn set_right_state(&mut self, state: bool) {
        self.right_pressed = state;
    }

    pub fn set_start_state(&mut self, state: bool) {
        self.start_pressed = state;
    }

    pub fn set_select_state(&mut self, state: bool) {
        self.select_pressed = state;
    }

    pub fn set_b_state(&mut self, state: bool) {
        self.b_pressed = state;
    }

    pub fn set_a_state(&mut self, state: bool) {
        self.a_pressed = state;
    }
}

#[derive(Clone)]
pub struct Breakpoint {
    read: bool,
    write: bool,
    execute: bool,

    address: u16
}

impl Breakpoint {
    pub fn new(read: bool, write: bool, execute: bool, address: u16) -> Breakpoint {
        Breakpoint {
            read,
            write,
            execute,
            address
        }
    }

    pub fn read(&self) -> &bool {
        &self.read
    }

    pub fn read_mut(&mut self) -> &mut bool {
        &mut self.read
    }

    pub fn write(&self) -> &bool {
        &self.write
    }

    pub fn write_mut(&mut self) -> &mut bool {
        &mut self.write
    }

    pub fn execute(&self) -> &bool {
        &self.execute
    }

    pub fn execute_mut(&mut self) -> &mut bool {
        &mut self.execute
    }

    pub fn address(&self) -> &u16 {
        &self.address
    }

    pub fn is_valid(&self) -> bool {
        self.read || self.write || self.execute
    }

    /// Set the breakpoint's address.
    pub fn set_address(&mut self, address: u16) {
        self.address = address;
    }
}

#[derive(Clone, PartialEq)]
pub enum EmulatorMode {
    Paused,
    Running,
    Stepping,
    BreakpointHit,
    UnknownInstruction(bool, u8)
}

impl fmt::Display for EmulatorMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmulatorMode::Paused => write!(f, "Execution paused."),
            EmulatorMode::Running => write!(f, "Emulator running."),
            EmulatorMode::Stepping => write!(f, "Stepping through pain."),
            EmulatorMode::BreakpointHit => write!(f, "Paused on a breakpoint."),
            EmulatorMode::UnknownInstruction(prefixed, opcode) => {
                if *prefixed {
                    write!(f, "Unimplemented instruction $CB ${:02X}", opcode)
                }
                else {
                    write!(f, "Unimplemented instruction ${:02X}", opcode)
                }
            },
        }
    }
}
