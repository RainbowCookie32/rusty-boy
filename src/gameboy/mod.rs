mod cpu;
mod gpu;
pub mod memory;
pub mod disassembler;

use std::fmt;
use std::sync::Arc;

use cpu::GameboyCPU;
use gpu::GameboyGPU;

use memory::GameboyMemory;
use memory::cart::CartHeader;

pub struct Gameboy {
    gb_cpu: GameboyCPU,
    gb_gpu: GameboyGPU,
    gb_mem: Arc<GameboyMemory>,

    pub dbg_mode: EmulatorMode,
    pub dbg_do_step: bool,
    pub dbg_breakpoint_list: Vec<Breakpoint>
}

impl Gameboy {
    pub fn init(gb_mem: Arc<GameboyMemory>) -> Gameboy {
        Gameboy {
            gb_cpu: GameboyCPU::init(gb_mem.clone()),
            gb_gpu: GameboyGPU::init(),
            gb_mem,

            dbg_mode: EmulatorMode::Paused,
            dbg_do_step: false,
            dbg_breakpoint_list: Vec::new()
        }
    }

    pub fn gb_reset(&mut self) {
        self.gb_cpu.reset();
        self.gb_mem.reset();

        self.dbg_mode = EmulatorMode::Paused;
    }

    pub fn gb_skip_bootrom(&mut self) {
        self.gb_cpu.skip_bootrom();
        // Disable the Bootrom by writing 1 to $FF50.
        self.gb_mem.write(0xFF50, 1);

        self.dbg_mode = EmulatorMode::Paused;
    }

    pub fn gb_cpu_cycle(&mut self) {
        self.gb_cpu.cpu_cycle(&self.dbg_breakpoint_list, &mut self.dbg_mode);
    }

    pub fn ui_get_header(&self) -> &CartHeader {
        self.gb_mem.header()
    }

    pub fn ui_get_cpu_registers(&self) -> (&u16, &u16, &u16, &u16, &u16, &u16) {
        self.gb_cpu.get_all_registers()
    }
}

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

    pub fn write(&self) -> &bool {
        &self.write
    }

    pub fn execute(&self) -> &bool {
        &self.execute
    }

    pub fn address(&self) -> &u16 {
        &self.address
    }

    /// Set the breakpoint's read.
    pub fn set_read(&mut self, read: bool) {
        self.read = read;
    }

    /// Set the breakpoint's write.
    pub fn set_write(&mut self, write: bool) {
        self.write = write;
    }

    /// Set the breakpoint's execute.
    pub fn set_execute(&mut self, execute: bool) {
        self.execute = execute;
    }

    /// Set the breakpoint's address.
    pub fn set_address(&mut self, address: u16) {
        self.address = address;
    }
}

#[derive(PartialEq)]
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
