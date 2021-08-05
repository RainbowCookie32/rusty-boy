mod interrupts;

use std::fmt;
use std::sync::{Arc, RwLock};

use interrupts::InterruptHandler;

use super::*;

#[derive(Clone, Copy)]
enum Condition {
    Zero(bool),
    Carry(bool)
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Condition::Zero(c) => {
                if *c {
                    write!(f, "Z")
                }
                else {
                    write!(f, "NZ")
                }
            }
            Condition::Carry(c) => {
                if *c {
                    write!(f, "C")
                }
                else {
                    write!(f, "NC")
                }
            }
        }
    }
}

enum Register {
    AF,
    BC(bool),
    DE(bool),
    HL(bool),
    SP
}

enum Flag {
    Zero(bool),
    Negative(bool),
    HalfCarry(bool),
    Carry(bool)
}

pub struct GameboyCPU {
    af: u16,
    bc: u16,
    de: u16,
    hl: u16,

    sp: u16,
    pc: u16,

    halted: bool,
    stopped: bool,

    cycles: usize,
    callstack: Arc<RwLock<Vec<String>>>,

    memory: Arc<GameboyMemory>,
    interrupt_handler: InterruptHandler
}

impl GameboyCPU {
    pub fn init(memory: Arc<GameboyMemory>) -> GameboyCPU {
        let interrupt_handler = InterruptHandler::init(memory.clone());

        GameboyCPU {
            af: 0,
            bc: 0,
            de: 0,
            hl: 0,

            sp: 0,
            pc: 0,

            halted: false,
            stopped: false,

            cycles: 0,
            callstack: Arc::new(RwLock::new(Vec::new())),

            memory,
            interrupt_handler
        }
    }

    fn get_flag(&self, flag: Flag) -> bool {
        match flag {
            Flag::Zero(_) => (self.af & 0x80) != 0,
            Flag::Negative(_) => (self.af & 0x40) != 0,
            Flag::HalfCarry(_) => (self.af & 0x20) != 0,
            Flag::Carry(_) => (self.af & 0x10) != 0,
        }
    }

    fn set_flag(&mut self, flag: Flag) {
        let mut flags = self.af & 0x00FF;

        match flag {
            Flag::Zero(value) => {
                if value {
                    flags |= 1 << 7;
                }
                else {
                    flags &= !(1 << 7);
                }
            }
            Flag::Negative(value) => {
                if value {
                    flags |= 1 << 6;
                }
                else {
                    flags &= !(1 << 6);
                }
            }
            Flag::HalfCarry(value) => {
                if value {
                    flags |= 1 << 5;
                }
                else {
                    flags &= !(1 << 5);
                }
            }
            Flag::Carry(value) => {
                if value {
                    flags |= 1 << 4;
                }
                else {
                    flags &= !(1 << 4);
                }
            }
        }

        self.af = (self.af & 0xFF00) | flags;
    }

    fn get_r8(&self, reg: &Register) -> u8 {
        match reg {
            Register::AF => {
                ((self.af & 0xFF00) >> 8) as u8
            }
            Register::BC(high) => {
                if *high {
                    ((self.bc & 0xFF00) >> 8) as u8
                }
                else {
                    (self.bc & 0x00FF) as u8
                }
            }
            Register::DE(high) => {
                if *high {
                    ((self.de & 0xFF00) >> 8) as u8
                }
                else {
                    (self.de & 0x00FF) as u8
                }
            }
            Register::HL(high) => {
                if *high {
                    ((self.hl & 0xFF00) >> 8) as u8
                }
                else {
                    (self.hl & 0x00FF) as u8
                }
            }
            _ => unreachable!()
        }
    }

    fn set_r8(&mut self, reg: Register, value: u8) {
        match reg {
            Register::AF => {
                self.af = (self.af & 0x00FF) | (value as u16) << 8;
            }
            Register::BC(high) => {
                if high {
                    self.bc = (self.bc & 0x00FF) | (value as u16) << 8;
                }
                else {
                    self.bc = (self.bc & 0xFF00) | value as u16;
                }
            }
            Register::DE(high) => {
                if high {
                    self.de = (self.de & 0x00FF) | (value as u16) << 8;
                }
                else {
                    self.de = (self.de & 0xFF00) | value as u16;
                }
            }
            Register::HL(high) => {
                if high {
                    self.hl = (self.hl & 0x00FF) | (value as u16) << 8;
                }
                else {
                    self.hl = (self.hl & 0xFF00) | value as u16;
                }
            }
            _ => unreachable!()
        }
    }

    fn get_rp(&self, reg: &Register) -> u16 {
        match reg {
            Register::AF => self.af,
            Register::BC(_) => self.bc,
            Register::DE(_) => self.de,
            Register::HL(_) => self.hl,
            Register::SP => self.sp
        }
    }

    fn set_rp(&mut self, reg: Register, value: u16) {
        match reg {
            Register::AF => self.af = value & 0xFFF0,
            Register::BC(_) => self.bc = value,
            Register::DE(_) => self.de = value,
            Register::HL(_) => self.hl = value,
            Register::SP => self.sp = value
        }
    }

    fn check_condition(&self, condition: Condition) -> bool {
        match condition {
            Condition::Zero(set) => {
                let flag = self.get_flag(Flag::Zero(false));
                if set {flag} else {!flag}
            }
            Condition::Carry(set) => {
                let flag = self.get_flag(Flag::Carry(false));
                if set {flag} else {!flag}
            }
        }
    }

    pub fn skip_bootrom(&mut self) {
        self.af = 0x01B0;
        self.bc = 0x0013;
        self.de = 0x00D8;
        self.hl = 0x014D;
        self.sp = 0xFFFE;
        self.pc = 0x0100;
    }

    pub fn get_callstack(&self) -> Arc<RwLock<Vec<String>>> {
        self.callstack.clone()
    }

    pub fn get_all_registers(&self) -> (&u16, &u16, &u16, &u16, &u16, &u16) {
        (&self.af, &self.bc, &self.de, &self.hl, &self.sp, &self.pc)
    }

    fn read_u8(&self, address: u16, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) -> (bool, u8) {
        let mut found_bp = false;
        let matching_bps: Vec<&Breakpoint> = breakpoints.iter().filter(|b| *b.address() == address).collect();

        for bp in matching_bps {
            // Don't trigger the breakpoint if we are stepping.
            // Assume you are paying attention to what's going on, and makes access breakpoints useable.
            if *bp.read() && *dbg_mode != EmulatorMode::Stepping {
                found_bp = true;
                break;
            }
        }

        (found_bp, self.memory.read(address))
    }

    fn read_u16(&self, address: u16, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) -> (bool, u16) {
        let mut found_bp = false;
        let matching_bps: Vec<&Breakpoint> = breakpoints.iter().filter(|b| *b.address() == address || *b.address() == address + 1).collect();

        for bp in matching_bps {
            // Same as in read_u8().
            if *bp.read() && *dbg_mode != EmulatorMode::Stepping {
                found_bp = true;
                break;
            }
        }

        let values = [self.memory.read(address), self.memory.read(address + 1)];

        (found_bp, u16::from_le_bytes(values))
    }

    fn write(&self, address: u16, value: u8, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) -> bool {
        let matching_bps: Vec<&Breakpoint> = breakpoints.iter().filter(|b| *b.address() == address).collect();

        for bp in matching_bps {
            // Same as in read_u8().
            if *bp.write() && *dbg_mode != EmulatorMode::Stepping {
                return true;
            }
        }

        self.memory.write(address, value);
        false
    }

    fn stack_read(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) -> (bool, u16) {
        let mut found_bp = false;
        let matching_bps: Vec<&Breakpoint> = breakpoints.iter().filter(|b| *b.address() == self.sp - 1 || *b.address() == self.sp - 2).collect();

        for bp in matching_bps {
            // Same as in read_u8().
            if *bp.read() && *dbg_mode != EmulatorMode::Stepping {
                found_bp = true;
                break;
            }
        }

        let values = [self.memory.read(self.sp), self.memory.read(self.sp + 1)];
        self.sp = self.sp.wrapping_add(2);

        (found_bp, u16::from_le_bytes(values))
    }

    fn stack_write(&mut self, value: u16, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) -> bool {
        let high = (value >> 8) as u8;
        let low = value as u8;

        self.sp = self.sp.wrapping_sub(1);
        if self.write(self.sp, high, breakpoints, dbg_mode) {
            return true;
        }

        self.sp = self.sp.wrapping_sub(1);
        if self.write(self.sp, low, breakpoints, dbg_mode) {
            return true;
        }

        false
    }

    pub fn reset(&mut self) {
        self.af = 0;
        self.bc = 0;
        self.de = 0;
        self.hl = 0;
        self.sp = 0;
        self.pc = 0;
        self.cycles = 0;
        
        if let Ok(mut lock) = self.callstack.write() {
            lock.clear();
        }
    }

    pub fn get_cycles(&mut self) -> &mut usize {
        &mut self.cycles
    }

    pub fn cpu_cycle(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        for bp in breakpoints {
            if self.pc == *bp.address() && *bp.execute() && *dbg_mode != EmulatorMode::Stepping {
                *dbg_mode = EmulatorMode::BreakpointHit;
                return;
            }
        }

        self.execute_instruction(breakpoints, dbg_mode);
    }

    fn execute_instruction(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (int_requested, int_address) = self.interrupt_handler.check_interrupts();

        if int_requested {
            if let Some(int) = int_address {    
                // FIXME: If a breakpoint *is* hit, the interrupt will be discarded.
                if self.stack_write(self.pc, breakpoints, dbg_mode) {
                    *dbg_mode = EmulatorMode::BreakpointHit;
                    return;
                }
    
                self.pc = int;
            }

            self.halted = false;
            self.stopped = false;
        }
        

        if self.halted || self.stopped {
            // HACK: Since the CPU is stopped, the cycle counter doesn't increase.
            // If the cycle counter doesn't increase, other parts of the system
            // wont' move either, so interrupts won't be triggered. In this case,
            // that'd mean it gets stuck on a halted or stop state forever.
            self.cycles += 4;
            return;
        }

        let (bp_hit, opcode) = self.read_u8(self.pc, breakpoints, dbg_mode);

        if bp_hit && *dbg_mode != EmulatorMode::Stepping {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        match opcode {
            0x00 => self.nop(),
            0x01 => self.load_u16_to_rp(breakpoints, dbg_mode, Register::BC(false)),
            0x02 => self.store_a_to_rp(breakpoints, dbg_mode, Register::BC(false)),
            0x03 => self.inc_rp(Register::BC(false)),
            0x04 => self.inc_r8(Register::BC(true)),
            0x05 => self.dec_r8(Register::BC(true)),
            0x06 => self.load_u8_to_r8(breakpoints, dbg_mode, Register::BC(true)),
            0x07 => self.rlca(),
            0x08 => self.store_sp_to_u16(breakpoints, dbg_mode),
            0x09 => self.add_hl_rp(Register::BC(false)),
            0x0A => self.load_a_from_rp(breakpoints, dbg_mode, Register::BC(false)),
            0x0B => self.dec_rp(Register::BC(false)),
            0x0C => self.inc_r8(Register::BC(false)),
            0x0D => self.dec_r8(Register::BC(false)),
            0x0E => self.load_u8_to_r8(breakpoints, dbg_mode, Register::BC(false)),
            0x0F => self.rrca(),

            // 0x10 => stop(),
            0x11 => self.load_u16_to_rp(breakpoints, dbg_mode, Register::DE(false)),
            0x12 => self.store_a_to_rp(breakpoints, dbg_mode, Register::DE(false)),
            0x13 => self.inc_rp(Register::DE(false)),
            0x14 => self.inc_r8(Register::DE(true)),
            0x15 => self.dec_r8(Register::DE(true)),
            0x16 => self.load_u8_to_r8(breakpoints, dbg_mode, Register::DE(true)),
            0x17 => self.rla(),
            0x18 => self.jump_relative(breakpoints, dbg_mode),
            0x19 => self.add_hl_rp(Register::DE(false)),
            0x1A => self.load_a_from_rp(breakpoints, dbg_mode, Register::DE(false)),
            0x1B => self.dec_rp(Register::DE(false)),
            0x1C => self.inc_r8(Register::DE(false)),
            0x1D => self.dec_r8(Register::DE(false)),
            0x1E => self.load_u8_to_r8(breakpoints, dbg_mode, Register::DE(false)),
            0x1F => self.rra(),

            0x20 => self.conditional_jump_relative(breakpoints, dbg_mode, Condition::Zero(false)),
            0x21 => self.load_u16_to_rp(breakpoints, dbg_mode, Register::HL(false)),
            0x22 => self.store_a_to_hl_and_inc(breakpoints, dbg_mode),
            0x23 => self.inc_rp(Register::HL(false)),
            0x24 => self.inc_r8(Register::HL(true)),
            0x25 => self.dec_r8(Register::HL(true)),
            0x26 => self.load_u8_to_r8(breakpoints, dbg_mode, Register::HL(true)),
            0x27 => self.daa(),
            0x28 => self.conditional_jump_relative(breakpoints, dbg_mode, Condition::Zero(true)),
            0x29 => self.add_hl_rp(Register::HL(false)),
            0x2A => self.load_a_from_hl_and_inc(breakpoints, dbg_mode),
            0x2B => self.dec_rp(Register::HL(false)),
            0x2C => self.inc_r8(Register::HL(false)),
            0x2D => self.dec_r8(Register::HL(false)),
            0x2E => self.load_u8_to_r8(breakpoints, dbg_mode, Register::HL(false)),
            0x2F => self.cpl(),

            0x30 => self.conditional_jump_relative(breakpoints, dbg_mode, Condition::Carry(false)),
            0x31 => self.load_u16_to_rp(breakpoints, dbg_mode, Register::SP),
            0x32 => self.store_a_to_hl_and_dec(breakpoints, dbg_mode),
            0x33 => self.inc_rp(Register::SP),
            0x34 => self.inc_hl(breakpoints, dbg_mode),
            0x35 => self.dec_hl(breakpoints, dbg_mode),
            0x36 => self.store_u8_to_hl(breakpoints, dbg_mode),
            0x37 => self.scf(),
            0x38 => self.conditional_jump_relative(breakpoints, dbg_mode, Condition::Carry(true)),
            0x39 => self.add_hl_rp(Register::SP),
            0x3A => self.load_a_from_hl_and_dec(breakpoints, dbg_mode),
            0x3B => self.dec_rp(Register::SP),
            0x3C => self.inc_r8(Register::AF),
            0x3D => self.dec_r8(Register::AF),
            0x3E => self.load_u8_to_r8(breakpoints, dbg_mode, Register::AF),
            0x3F => self.ccf(),

            0x40 => self.load_r8_to_r8(Register::BC(true), Register::BC(true)),
            0x41 => self.load_r8_to_r8(Register::BC(true), Register::BC(false)),
            0x42 => self.load_r8_to_r8(Register::BC(true), Register::DE(true)),
            0x43 => self.load_r8_to_r8(Register::BC(true), Register::DE(false)),
            0x44 => self.load_r8_to_r8(Register::BC(true), Register::HL(true)),
            0x45 => self.load_r8_to_r8(Register::BC(true), Register::HL(false)),
            0x46 => self.load_hl_to_r8(breakpoints, dbg_mode, Register::BC(true)),
            0x47 => self.load_r8_to_r8(Register::BC(true), Register::AF),
            0x48 => self.load_r8_to_r8(Register::BC(false), Register::BC(true)),
            0x49 => self.load_r8_to_r8(Register::BC(false), Register::BC(false)),
            0x4A => self.load_r8_to_r8(Register::BC(false), Register::DE(true)),
            0x4B => self.load_r8_to_r8(Register::BC(false), Register::DE(false)),
            0x4C => self.load_r8_to_r8(Register::BC(false), Register::HL(true)),
            0x4D => self.load_r8_to_r8(Register::BC(false), Register::HL(false)),
            0x4E => self.load_hl_to_r8(breakpoints, dbg_mode, Register::BC(false)),
            0x4F => self.load_r8_to_r8(Register::BC(false), Register::AF),

            0x50 => self.load_r8_to_r8(Register::DE(true), Register::BC(true)),
            0x51 => self.load_r8_to_r8(Register::DE(true), Register::BC(false)),
            0x52 => self.load_r8_to_r8(Register::DE(true), Register::DE(true)),
            0x53 => self.load_r8_to_r8(Register::DE(true), Register::DE(false)),
            0x54 => self.load_r8_to_r8(Register::DE(true), Register::HL(true)),
            0x55 => self.load_r8_to_r8(Register::DE(true), Register::HL(false)),
            0x56 => self.load_hl_to_r8(breakpoints, dbg_mode, Register::DE(true)),
            0x57 => self.load_r8_to_r8(Register::DE(true), Register::AF),
            0x58 => self.load_r8_to_r8(Register::DE(false), Register::BC(true)),
            0x59 => self.load_r8_to_r8(Register::DE(false), Register::BC(false)),
            0x5A => self.load_r8_to_r8(Register::DE(false), Register::DE(true)),
            0x5B => self.load_r8_to_r8(Register::DE(false), Register::DE(false)),
            0x5C => self.load_r8_to_r8(Register::DE(false), Register::HL(true)),
            0x5D => self.load_r8_to_r8(Register::DE(false), Register::HL(false)),
            0x5E => self.load_hl_to_r8(breakpoints, dbg_mode, Register::DE(false)),
            0x5F => self.load_r8_to_r8(Register::DE(false), Register::AF),

            0x60 => self.load_r8_to_r8(Register::HL(true), Register::BC(true)),
            0x61 => self.load_r8_to_r8(Register::HL(true), Register::BC(false)),
            0x62 => self.load_r8_to_r8(Register::HL(true), Register::DE(true)),
            0x63 => self.load_r8_to_r8(Register::HL(true), Register::DE(false)),
            0x64 => self.load_r8_to_r8(Register::HL(true), Register::HL(true)),
            0x65 => self.load_r8_to_r8(Register::HL(true), Register::HL(false)),
            0x66 => self.load_hl_to_r8(breakpoints, dbg_mode, Register::HL(true)),
            0x67 => self.load_r8_to_r8(Register::HL(true), Register::AF),
            0x68 => self.load_r8_to_r8(Register::HL(false), Register::BC(true)),
            0x69 => self.load_r8_to_r8(Register::HL(false), Register::BC(false)),
            0x6A => self.load_r8_to_r8(Register::HL(false), Register::DE(true)),
            0x6B => self.load_r8_to_r8(Register::HL(false), Register::DE(false)),
            0x6C => self.load_r8_to_r8(Register::HL(false), Register::HL(true)),
            0x6D => self.load_r8_to_r8(Register::HL(false), Register::HL(false)),
            0x6E => self.load_hl_to_r8(breakpoints, dbg_mode, Register::HL(false)),
            0x6F => self.load_r8_to_r8(Register::HL(false), Register::AF),

            0x70 => self.store_r8_to_hl(breakpoints, dbg_mode, Register::BC(true)),
            0x71 => self.store_r8_to_hl(breakpoints, dbg_mode, Register::BC(false)),
            0x72 => self.store_r8_to_hl(breakpoints, dbg_mode, Register::DE(true)),
            0x73 => self.store_r8_to_hl(breakpoints, dbg_mode, Register::DE(false)),
            0x74 => self.store_r8_to_hl(breakpoints, dbg_mode, Register::HL(true)),
            0x75 => self.store_r8_to_hl(breakpoints, dbg_mode, Register::HL(false)),
            0x76 => self.halt(),
            0x77 => self.store_r8_to_hl(breakpoints, dbg_mode, Register::AF),
            0x78 => self.load_r8_to_r8(Register::AF, Register::BC(true)),
            0x79 => self.load_r8_to_r8(Register::AF, Register::BC(false)),
            0x7A => self.load_r8_to_r8(Register::AF, Register::DE(true)),
            0x7B => self.load_r8_to_r8(Register::AF, Register::DE(false)),
            0x7C => self.load_r8_to_r8(Register::AF, Register::HL(true)),
            0x7D => self.load_r8_to_r8(Register::AF, Register::HL(false)),
            0x7E => self.load_hl_to_r8(breakpoints, dbg_mode, Register::AF),
            0x7F => self.load_r8_to_r8(Register::AF, Register::AF),

            0x80 => self.add_r8(Register::BC(true)),
            0x81 => self.add_r8(Register::BC(false)),
            0x82 => self.add_r8(Register::DE(true)),
            0x83 => self.add_r8(Register::DE(false)),
            0x84 => self.add_r8(Register::HL(true)),
            0x85 => self.add_r8(Register::HL(false)),
            0x86 => self.add_hl(breakpoints, dbg_mode),
            0x87 => self.add_r8(Register::AF),
            0x88 => self.adc_r8(Register::BC(true)),
            0x89 => self.adc_r8(Register::BC(false)),
            0x8A => self.adc_r8(Register::DE(true)),
            0x8B => self.adc_r8(Register::DE(false)),
            0x8C => self.adc_r8(Register::HL(true)),
            0x8D => self.adc_r8(Register::HL(false)),
            0x8E => self.adc_hl(breakpoints, dbg_mode),
            0x8F => self.adc_r8(Register::AF),

            0x90 => self.sub_r8(Register::BC(true)),
            0x91 => self.sub_r8(Register::BC(false)),
            0x92 => self.sub_r8(Register::DE(true)),
            0x93 => self.sub_r8(Register::DE(false)),
            0x94 => self.sub_r8(Register::HL(true)),
            0x95 => self.sub_r8(Register::HL(false)),
            0x96 => self.sub_hl(breakpoints, dbg_mode),
            0x97 => self.sub_r8(Register::AF),
            0x98 => self.sbc_r8(Register::BC(true)),
            0x99 => self.sbc_r8(Register::BC(false)),
            0x9A => self.sbc_r8(Register::DE(true)),
            0x9B => self.sbc_r8(Register::DE(false)),
            0x9C => self.sbc_r8(Register::HL(true)),
            0x9D => self.sbc_r8(Register::HL(false)),
            0x9E => self.sbc_hl(breakpoints, dbg_mode),
            0x9F => self.sbc_r8(Register::AF),

            0xA0 => self.and_r8(Register::BC(true)),
            0xA1 => self.and_r8(Register::BC(false)),
            0xA2 => self.and_r8(Register::DE(true)),
            0xA3 => self.and_r8(Register::DE(false)),
            0xA4 => self.and_r8(Register::HL(true)),
            0xA5 => self.and_r8(Register::HL(false)),
            0xA6 => self.and_hl(breakpoints, dbg_mode),
            0xA7 => self.and_r8(Register::AF),
            0xA8 => self.xor_r8(Register::BC(true)),
            0xA9 => self.xor_r8(Register::BC(false)),
            0xAA => self.xor_r8(Register::DE(true)),
            0xAB => self.xor_r8(Register::DE(false)),
            0xAC => self.xor_r8(Register::HL(true)),
            0xAD => self.xor_r8(Register::HL(false)),
            0xAE => self.xor_hl(breakpoints, dbg_mode),
            0xAF => self.xor_r8(Register::AF),

            0xB0 => self.or_r8(Register::BC(true)),
            0xB1 => self.or_r8(Register::BC(false)),
            0xB2 => self.or_r8(Register::DE(true)),
            0xB3 => self.or_r8(Register::DE(false)),
            0xB4 => self.or_r8(Register::HL(true)),
            0xB5 => self.or_r8(Register::HL(false)),
            0xB6 => self.or_hl(breakpoints, dbg_mode),
            0xB7 => self.or_r8(Register::AF),
            0xB8 => self.cp_r8(Register::BC(true)),
            0xB9 => self.cp_r8(Register::BC(false)),
            0xBA => self.cp_r8(Register::DE(true)),
            0xBB => self.cp_r8(Register::DE(false)),
            0xBC => self.cp_r8(Register::HL(true)),
            0xBD => self.cp_r8(Register::HL(false)),
            0xBE => self.cp_hl(breakpoints, dbg_mode),
            0xBF => self.cp_r8(Register::AF),

            0xC0 => self.conditional_ret(breakpoints, dbg_mode, Condition::Zero(false)),
            0xC1 => self.pop_rp(breakpoints, dbg_mode, Register::BC(false)),
            0xC2 => self.conditional_jump(breakpoints, dbg_mode, Condition::Zero(false)),
            0xC3 => self.jump(breakpoints, dbg_mode),
            0xC4 => self.conditional_call(breakpoints, dbg_mode, Condition::Zero(false)),
            0xC5 => self.push_rp(breakpoints, dbg_mode, Register::BC(false)),
            0xC6 => self.add_u8(breakpoints, dbg_mode),
            0xC7 => self.rst(0x00, breakpoints, dbg_mode),
            0xC8 => self.conditional_ret(breakpoints, dbg_mode, Condition::Zero(true)),
            0xC9 => self.ret(breakpoints, dbg_mode),
            0xCA => self.conditional_jump(breakpoints, dbg_mode, Condition::Zero(true)),
            0xCB => self.execute_instruction_prefixed(breakpoints, dbg_mode),
            0xCC => self.conditional_call(breakpoints, dbg_mode, Condition::Zero(true)),
            0xCD => self.call(breakpoints, dbg_mode),
            0xCE => self.adc_u8(breakpoints, dbg_mode),
            0xCF => self.rst(0x08, breakpoints, dbg_mode),

            0xD0 => self.conditional_ret(breakpoints, dbg_mode, Condition::Carry(false)),
            0xD1 => self.pop_rp(breakpoints, dbg_mode, Register::DE(false)),
            0xD2 => self.conditional_jump(breakpoints, dbg_mode, Condition::Carry(false)),
            // 0xD3 => illegal opcode
            0xD4 => self.conditional_call(breakpoints, dbg_mode, Condition::Carry(false)),
            0xD5 => self.push_rp(breakpoints, dbg_mode, Register::DE(false)),
            0xD6 => self.sub_u8(breakpoints, dbg_mode),
            0xD7 => self.rst(0x10, breakpoints, dbg_mode),
            0xD8 => self.conditional_ret(breakpoints, dbg_mode, Condition::Carry(true)),
            0xD9 => self.reti(breakpoints, dbg_mode),
            0xDA => self.conditional_jump(breakpoints, dbg_mode, Condition::Carry(true)),
            // 0xDB => illegal opcode
            0xDC => self.conditional_call(breakpoints, dbg_mode, Condition::Carry(true)),
            // 0xDD => illegal opcode
            0xDE => self.sbc_u8(breakpoints, dbg_mode),
            0xDF => self.rst(0x18, breakpoints, dbg_mode),

            0xE0 => self.store_a_to_io_u8(breakpoints, dbg_mode),
            0xE1 => self.pop_rp(breakpoints, dbg_mode, Register::HL(false)),
            0xE2 => self.store_a_to_io_c(breakpoints, dbg_mode),
            // 0xE3 => illegal opcode
            // 0xE4 => illegal opcode
            0xE5 => self.push_rp(breakpoints, dbg_mode, Register::HL(false)),
            0xE6 => self.and_u8(breakpoints, dbg_mode),
            0xE7 => self.rst(0x20, breakpoints, dbg_mode),
            0xE8 => self.add_i8_to_sp(breakpoints, dbg_mode),
            0xE9 => self.jump_hl(),
            0xEA => self.store_a_to_u16(breakpoints, dbg_mode),
            // 0xEB => illegal opcode
            // 0xEC => illegal opcode
            // 0xED => illegal opcode
            0xEE => self.xor_u8(breakpoints, dbg_mode),
            0xEF => self.rst(0x28, breakpoints, dbg_mode),

            0xF0 => self.load_a_from_io_u8(breakpoints, dbg_mode),
            0xF1 => self.pop_rp(breakpoints, dbg_mode, Register::AF),
            0xF2 => self.load_a_from_io_c(breakpoints, dbg_mode),
            0xF3 => self.di(),
            // 0xF4 => illegal opcode
            0xF5 => self.push_rp(breakpoints, dbg_mode, Register::AF),
            0xF6 => self.or_u8(breakpoints, dbg_mode),
            0xF7 => self.rst(0x30, breakpoints, dbg_mode),
            0xF8 => self.load_sp_i8_to_hl(breakpoints, dbg_mode),
            0xF9 => self.load_hl_to_sp(),
            0xFA => self.load_a_from_u16(breakpoints, dbg_mode),
            0xFB => self.ei(),
            // 0xFC => illegal opcode
            // 0xFD => illegal opcode
            0xFE => self.cp_u8(breakpoints, dbg_mode),
            0xFF => self.rst(0x38, breakpoints, dbg_mode),

            _ => *dbg_mode = EmulatorMode::UnknownInstruction(false, opcode)
        }
    }

    fn execute_instruction_prefixed(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (bp_hit, opcode) = self.read_u8(self.pc + 1, breakpoints, dbg_mode);

        if bp_hit && *dbg_mode != EmulatorMode::Stepping {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        match opcode {
            0x00 => self.rlc_r8(Register::BC(true)),
            0x01 => self.rlc_r8(Register::BC(false)),
            0x02 => self.rlc_r8(Register::DE(true)),
            0x03 => self.rlc_r8(Register::DE(false)),
            0x04 => self.rlc_r8(Register::HL(true)),
            0x05 => self.rlc_r8(Register::HL(false)),
            0x06 => self.rlc_hl(breakpoints, dbg_mode),
            0x07 => self.rlc_r8(Register::AF),
            0x08 => self.rrc_r8(Register::BC(true)),
            0x09 => self.rrc_r8(Register::BC(false)),
            0x0A => self.rrc_r8(Register::DE(true)),
            0x0B => self.rrc_r8(Register::DE(false)),
            0x0C => self.rrc_r8(Register::HL(true)),
            0x0D => self.rrc_r8(Register::HL(false)),
            0x0E => self.rrc_hl(breakpoints, dbg_mode),
            0x0F => self.rrc_r8(Register::AF),

            0x10 => self.rl_r8(Register::BC(true)),
            0x11 => self.rl_r8(Register::BC(false)),
            0x12 => self.rl_r8(Register::DE(true)),
            0x13 => self.rl_r8(Register::DE(false)),
            0x14 => self.rl_r8(Register::HL(true)),
            0x15 => self.rl_r8(Register::HL(false)),
            0x16 => self.rl_hl(breakpoints, dbg_mode),
            0x17 => self.rl_r8(Register::AF),
            0x18 => self.rr_r8(Register::BC(true)),
            0x19 => self.rr_r8(Register::BC(false)),
            0x1A => self.rr_r8(Register::DE(true)),
            0x1B => self.rr_r8(Register::DE(false)),
            0x1C => self.rr_r8(Register::HL(true)),
            0x1D => self.rr_r8(Register::HL(false)),
            0x1E => self.rr_hl(breakpoints, dbg_mode),
            0x1F => self.rr_r8(Register::AF),

            0x20 => self.sla_r8(Register::BC(true)),
            0x21 => self.sla_r8(Register::BC(false)),
            0x22 => self.sla_r8(Register::DE(true)),
            0x23 => self.sla_r8(Register::DE(false)),
            0x24 => self.sla_r8(Register::HL(true)),
            0x25 => self.sla_r8(Register::HL(false)),
            0x26 => self.sla_hl(breakpoints, dbg_mode),
            0x27 => self.sla_r8(Register::AF),
            0x28 => self.sra_r8(Register::BC(true)),
            0x29 => self.sra_r8(Register::BC(false)),
            0x2A => self.sra_r8(Register::DE(true)),
            0x2B => self.sra_r8(Register::DE(false)),
            0x2C => self.sra_r8(Register::HL(true)),
            0x2D => self.sra_r8(Register::HL(false)),
            0x2E => self.sra_hl(breakpoints, dbg_mode),
            0x2F => self.sra_r8(Register::AF),

            0x30 => self.swap_r8(Register::BC(true)),
            0x31 => self.swap_r8(Register::BC(false)),
            0x32 => self.swap_r8(Register::DE(true)),
            0x33 => self.swap_r8(Register::DE(false)),
            0x34 => self.swap_r8(Register::HL(true)),
            0x35 => self.swap_r8(Register::HL(false)),
            0x36 => self.swap_hl(breakpoints, dbg_mode),
            0x37 => self.swap_r8(Register::AF),
            0x38 => self.srl_r8(Register::BC(true)),
            0x39 => self.srl_r8(Register::BC(false)),
            0x3A => self.srl_r8(Register::DE(true)),
            0x3B => self.srl_r8(Register::DE(false)),
            0x3C => self.srl_r8(Register::HL(true)),
            0x3D => self.srl_r8(Register::HL(false)),
            0x3E => self.srl_hl(breakpoints, dbg_mode),
            0x3F => self.srl_r8(Register::AF),

            0x40 => self.bit_r8(Register::BC(true), 0),
            0x41 => self.bit_r8(Register::BC(false), 0),
            0x42 => self.bit_r8(Register::DE(true), 0),
            0x43 => self.bit_r8(Register::DE(false), 0),
            0x44 => self.bit_r8(Register::HL(true), 0),
            0x45 => self.bit_r8(Register::HL(false), 0),
            0x46 => self.bit_hl(breakpoints, dbg_mode, 0),
            0x47 => self.bit_r8(Register::AF, 0),
            0x48 => self.bit_r8(Register::BC(true), 1),
            0x49 => self.bit_r8(Register::BC(false), 1),
            0x4A => self.bit_r8(Register::DE(true), 1),
            0x4B => self.bit_r8(Register::DE(false), 1),
            0x4C => self.bit_r8(Register::HL(true), 1),
            0x4D => self.bit_r8(Register::HL(false), 1),
            0x4E => self.bit_hl(breakpoints, dbg_mode, 1),
            0x4F => self.bit_r8(Register::AF, 1),

            0x50 => self.bit_r8(Register::BC(true), 2),
            0x51 => self.bit_r8(Register::BC(false), 2),
            0x52 => self.bit_r8(Register::DE(true), 2),
            0x53 => self.bit_r8(Register::DE(false), 2),
            0x54 => self.bit_r8(Register::HL(true), 2),
            0x55 => self.bit_r8(Register::HL(false), 2),
            0x56 => self.bit_hl(breakpoints, dbg_mode, 2),
            0x57 => self.bit_r8(Register::AF, 2),
            0x58 => self.bit_r8(Register::BC(true), 3),
            0x59 => self.bit_r8(Register::BC(false), 3),
            0x5A => self.bit_r8(Register::DE(true), 3),
            0x5B => self.bit_r8(Register::DE(false), 3),
            0x5C => self.bit_r8(Register::HL(true), 3),
            0x5D => self.bit_r8(Register::HL(false), 3),
            0x5E => self.bit_hl(breakpoints, dbg_mode, 3),
            0x5F => self.bit_r8(Register::AF, 3),

            0x60 => self.bit_r8(Register::BC(true), 4),
            0x61 => self.bit_r8(Register::BC(false), 4),
            0x62 => self.bit_r8(Register::DE(true), 4),
            0x63 => self.bit_r8(Register::DE(false), 4),
            0x64 => self.bit_r8(Register::HL(true), 4),
            0x65 => self.bit_r8(Register::HL(false), 4),
            0x66 => self.bit_hl(breakpoints, dbg_mode, 4),
            0x67 => self.bit_r8(Register::AF, 4),
            0x68 => self.bit_r8(Register::BC(true), 5),
            0x69 => self.bit_r8(Register::BC(false), 5),
            0x6A => self.bit_r8(Register::DE(true), 5),
            0x6B => self.bit_r8(Register::DE(false), 5),
            0x6C => self.bit_r8(Register::HL(true), 5),
            0x6D => self.bit_r8(Register::HL(false), 5),
            0x6E => self.bit_hl(breakpoints, dbg_mode, 5),
            0x6F => self.bit_r8(Register::AF, 5),

            0x70 => self.bit_r8(Register::BC(true), 6),
            0x71 => self.bit_r8(Register::BC(false), 6),
            0x72 => self.bit_r8(Register::DE(true), 6),
            0x73 => self.bit_r8(Register::DE(false), 6),
            0x74 => self.bit_r8(Register::HL(true), 6),
            0x75 => self.bit_r8(Register::HL(false), 6),
            0x76 => self.bit_hl(breakpoints, dbg_mode, 6),
            0x77 => self.bit_r8(Register::AF, 6),
            0x78 => self.bit_r8(Register::BC(true), 7),
            0x79 => self.bit_r8(Register::BC(false), 7),
            0x7A => self.bit_r8(Register::DE(true), 7),
            0x7B => self.bit_r8(Register::DE(false), 7),
            0x7C => self.bit_r8(Register::HL(true), 7),
            0x7D => self.bit_r8(Register::HL(false), 7),
            0x7E => self.bit_hl(breakpoints, dbg_mode, 7),
            0x7F => self.bit_r8(Register::AF, 7),

            0x80 => self.res_r8(Register::BC(true), 0),
            0x81 => self.res_r8(Register::BC(false), 0),
            0x82 => self.res_r8(Register::DE(true), 0),
            0x83 => self.res_r8(Register::DE(false), 0),
            0x84 => self.res_r8(Register::HL(true), 0),
            0x85 => self.res_r8(Register::HL(false), 0),
            0x86 => self.res_hl(breakpoints, dbg_mode, 0),
            0x87 => self.res_r8(Register::AF, 0),
            0x88 => self.res_r8(Register::BC(true), 1),
            0x89 => self.res_r8(Register::BC(false), 1),
            0x8A => self.res_r8(Register::DE(true), 1),
            0x8B => self.res_r8(Register::DE(false), 1),
            0x8C => self.res_r8(Register::HL(true), 1),
            0x8D => self.res_r8(Register::HL(false), 1),
            0x8E => self.res_hl(breakpoints, dbg_mode, 1),
            0x8F => self.res_r8(Register::AF, 1),

            0x90 => self.res_r8(Register::BC(true), 2),
            0x91 => self.res_r8(Register::BC(false), 2),
            0x92 => self.res_r8(Register::DE(true), 2),
            0x93 => self.res_r8(Register::DE(false), 2),
            0x94 => self.res_r8(Register::HL(true), 2),
            0x95 => self.res_r8(Register::HL(false), 2),
            0x96 => self.res_hl(breakpoints, dbg_mode, 2),
            0x97 => self.res_r8(Register::AF, 2),
            0x98 => self.res_r8(Register::BC(true), 3),
            0x99 => self.res_r8(Register::BC(false), 3),
            0x9A => self.res_r8(Register::DE(true), 3),
            0x9B => self.res_r8(Register::DE(false), 3),
            0x9C => self.res_r8(Register::HL(true), 3),
            0x9D => self.res_r8(Register::HL(false), 3),
            0x9E => self.res_hl(breakpoints, dbg_mode, 3),
            0x9F => self.res_r8(Register::AF, 3),

            0xA0 => self.res_r8(Register::BC(true), 4),
            0xA1 => self.res_r8(Register::BC(false), 4),
            0xA2 => self.res_r8(Register::DE(true), 4),
            0xA3 => self.res_r8(Register::DE(false), 4),
            0xA4 => self.res_r8(Register::HL(true), 4),
            0xA5 => self.res_r8(Register::HL(false), 4),
            0xA6 => self.res_hl(breakpoints, dbg_mode, 4),
            0xA7 => self.res_r8(Register::AF, 4),
            0xA8 => self.res_r8(Register::BC(true), 5),
            0xA9 => self.res_r8(Register::BC(false), 5),
            0xAA => self.res_r8(Register::DE(true), 5),
            0xAB => self.res_r8(Register::DE(false), 5),
            0xAC => self.res_r8(Register::HL(true), 5),
            0xAD => self.res_r8(Register::HL(false), 5),
            0xAE => self.res_hl(breakpoints, dbg_mode, 5),
            0xAF => self.res_r8(Register::AF, 5),

            0xB0 => self.res_r8(Register::BC(true), 6),
            0xB1 => self.res_r8(Register::BC(false), 6),
            0xB2 => self.res_r8(Register::DE(true), 6),
            0xB3 => self.res_r8(Register::DE(false), 6),
            0xB4 => self.res_r8(Register::HL(true), 6),
            0xB5 => self.res_r8(Register::HL(false), 6),
            0xB6 => self.res_hl(breakpoints, dbg_mode, 6),
            0xB7 => self.res_r8(Register::AF, 6),
            0xB8 => self.res_r8(Register::BC(true), 7),
            0xB9 => self.res_r8(Register::BC(false), 7),
            0xBA => self.res_r8(Register::DE(true), 7),
            0xBB => self.res_r8(Register::DE(false), 7),
            0xBC => self.res_r8(Register::HL(true), 7),
            0xBD => self.res_r8(Register::HL(false), 7),
            0xBE => self.res_hl(breakpoints, dbg_mode, 7),
            0xBF => self.res_r8(Register::AF, 7),

            0xC0 => self.set(Register::BC(true), 0),
            0xC1 => self.set(Register::BC(false), 0),
            0xC2 => self.set(Register::DE(true), 0),
            0xC3 => self.set(Register::DE(false), 0),
            0xC4 => self.set(Register::HL(true), 0),
            0xC5 => self.set(Register::HL(false), 0),
            0xC6 => self.set_hl(breakpoints, dbg_mode, 0),
            0xC7 => self.set(Register::AF, 0),
            0xC8 => self.set(Register::BC(true), 1),
            0xC9 => self.set(Register::BC(false), 1),
            0xCA => self.set(Register::DE(true), 1),
            0xCB => self.set(Register::DE(false), 1),
            0xCC => self.set(Register::HL(true), 1),
            0xCD => self.set(Register::HL(false), 1),
            0xCE => self.set_hl(breakpoints, dbg_mode, 1),
            0xCF => self.set(Register::AF, 1),

            0xD0 => self.set(Register::BC(true), 2),
            0xD1 => self.set(Register::BC(false), 2),
            0xD2 => self.set(Register::DE(true), 2),
            0xD3 => self.set(Register::DE(false), 2),
            0xD4 => self.set(Register::HL(true), 2),
            0xD5 => self.set(Register::HL(false), 2),
            0xD6 => self.set_hl(breakpoints, dbg_mode, 2),
            0xD7 => self.set(Register::AF, 2),
            0xD8 => self.set(Register::BC(true), 3),
            0xD9 => self.set(Register::BC(false), 3),
            0xDA => self.set(Register::DE(true), 3),
            0xDB => self.set(Register::DE(false), 3),
            0xDC => self.set(Register::HL(true), 3),
            0xDD => self.set(Register::HL(false), 3),
            0xDE => self.set_hl(breakpoints, dbg_mode, 3),
            0xDF => self.set(Register::AF, 3),

            0xE0 => self.set(Register::BC(true), 4),
            0xE1 => self.set(Register::BC(false), 4),
            0xE2 => self.set(Register::DE(true), 4),
            0xE3 => self.set(Register::DE(false), 4),
            0xE4 => self.set(Register::HL(true), 4),
            0xE5 => self.set(Register::HL(false), 4),
            0xE6 => self.set_hl(breakpoints, dbg_mode, 4),
            0xE7 => self.set(Register::AF, 4),
            0xE8 => self.set(Register::BC(true), 5),
            0xE9 => self.set(Register::BC(false), 5),
            0xEA => self.set(Register::DE(true), 5),
            0xEB => self.set(Register::DE(false), 5),
            0xEC => self.set(Register::HL(true), 5),
            0xED => self.set(Register::HL(false), 5),
            0xEE => self.set_hl(breakpoints, dbg_mode, 5),
            0xEF => self.set(Register::AF, 5),

            0xF0 => self.set(Register::BC(true), 6),
            0xF1 => self.set(Register::BC(false), 6),
            0xF2 => self.set(Register::DE(true), 6),
            0xF3 => self.set(Register::DE(false), 6),
            0xF4 => self.set(Register::HL(true), 6),
            0xF5 => self.set(Register::HL(false), 6),
            0xF6 => self.set_hl(breakpoints, dbg_mode, 6),
            0xF7 => self.set(Register::AF, 6),
            0xF8 => self.set(Register::BC(true), 7),
            0xF9 => self.set(Register::BC(false), 7),
            0xFA => self.set(Register::DE(true), 7),
            0xFB => self.set(Register::DE(false), 7),
            0xFC => self.set(Register::HL(true), 7),
            0xFD => self.set(Register::HL(false), 7),
            0xFE => self.set_hl(breakpoints, dbg_mode, 7),
            0xFF => self.set(Register::AF, 7)
        }
    }

    fn nop(&mut self) {
        self.pc += 1;
        self.cycles += 4;
    }

    fn load_u8_to_r8(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode, reg: Register) {
        let (bp_hit, value) = self.read_u8(self.pc + 1, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.set_r8(reg, value);

        self.pc += 2;
        self.cycles += 8;
    }

    fn load_hl_to_r8(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode, reg: Register) {
        let (bp_hit, value) = self.read_u8(self.hl, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.set_r8(reg, value);

        self.pc += 1;
        self.cycles += 8;
    }

    fn load_a_from_rp(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode, reg: Register) {
        let address = self.get_rp(&reg);
        let (bp_hit, value) = self.read_u8(address, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.set_r8(Register::AF, value);

        self.pc += 1;
        self.cycles += 8;
    }

    fn load_a_from_hl_and_inc(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (bp_hit, value) = self.read_u8(self.hl, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.set_r8(Register::AF, value);
        self.set_rp(Register::HL(true), self.hl.wrapping_add(1));

        self.pc += 1;
        self.cycles += 8;
    }

    fn load_a_from_hl_and_dec(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (bp_hit, value) = self.read_u8(self.hl, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.set_r8(Register::AF, value);
        self.set_rp(Register::HL(true), self.hl.wrapping_sub(1));

        self.pc += 1;
        self.cycles += 8;
    }

    fn load_a_from_io_c(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (bp_hit, value) = self.read_u8(0xFF00 + self.get_r8(&Register::BC(false)) as u16, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.set_r8(Register::AF, value);

        self.pc += 1;
        self.cycles += 8;
    }

    fn load_a_from_io_u8(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (bp_hit, value) = self.read_u8(self.pc + 1, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        let (bp_hit, value) = self.read_u8(0xFF00 + value as u16, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.set_r8(Register::AF, value);

        self.pc += 2;
        self.cycles += 12;
    }

    fn load_a_from_u16(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (bp_hit, address) = self.read_u16(self.pc + 1, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        let (bp_hit, value) = self.read_u8(address, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.set_r8(Register::AF, value);

        self.pc += 3;
        self.cycles += 16;
    }

    fn load_u16_to_rp(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode, reg: Register) {
        let (bp_hit, value) = self.read_u16(self.pc + 1, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.set_rp(reg, value);

        self.pc += 3;
        self.cycles += 12;
    }

    fn load_hl_to_sp(&mut self) {
        self.sp = self.hl;
        
        self.pc += 1;
        self.cycles += 8;
    }

    fn load_r8_to_r8(&mut self, target: Register, source: Register) {
        self.set_r8(target, self.get_r8(&source));
        
        self.pc += 1;
        self.cycles += 4;
    }

    fn load_sp_i8_to_hl(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (bp_hit, value) = self.read_u8(self.pc + 1, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        let sp = self.sp;
        let value = (value as i8) as u16;
        let result = sp.wrapping_add(value);

        self.hl = result;

        self.set_flag(Flag::Zero(false));
        self.set_flag(Flag::Negative(false));
        self.set_flag(Flag::HalfCarry((sp & 0x0F) + (value & 0x0F) > 0x0F));
        self.set_flag(Flag::Carry((sp & 0x00FF) + (value & 0x00FF) > 0xFF));

        self.pc += 2;
        self.cycles += 12;
    }

    fn store_r8_to_hl(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode, reg: Register) {
        let value = self.get_r8(&reg);
        
        if self.write(self.hl, value, breakpoints, dbg_mode) {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }
        
        self.pc += 1;
        self.cycles += 8;
    }

    fn store_a_to_hl_and_inc(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let value = self.get_r8(&Register::AF);
        
        if self.write(self.hl, value, breakpoints, dbg_mode) {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }
        
        self.set_rp(Register::HL(false), self.hl.wrapping_add(1));

        self.pc += 1;
        self.cycles += 8;
    }

    fn store_a_to_hl_and_dec(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let value = self.get_r8(&Register::AF);
        
        if self.write(self.hl, value, breakpoints, dbg_mode) {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }
        
        self.set_rp(Register::HL(false), self.hl.wrapping_sub(1));

        self.pc += 1;
        self.cycles += 8;
    }

    fn store_a_to_rp(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode, reg: Register) {
        let value = self.get_r8(&Register::AF);

        if self.write(self.get_rp(&reg), value, breakpoints, dbg_mode) {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.pc += 1;
        self.cycles += 8;
    }

    fn store_a_to_io_c(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let value = self.get_r8(&Register::AF);

        if self.write(0xFF00 + self.get_r8(&Register::BC(false)) as u16, value, breakpoints, dbg_mode) {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }
        
        self.pc += 1;
        self.cycles += 8;
    }

    fn store_a_to_io_u8(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (bp_hit, offset) = self.read_u8(self.pc + 1, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        let value = self.get_r8(&Register::AF);

        if self.write(0xFF00 + offset as u16, value, breakpoints, dbg_mode) {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }
        
        self.pc += 2;
        self.cycles += 12;
    }

    fn store_a_to_u16(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (bp_hit, address) = self.read_u16(self.pc + 1, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        if self.write(address, self.get_r8(&Register::AF), breakpoints, dbg_mode) {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.pc += 3;
        self.cycles += 16;
    }

    fn store_sp_to_u16(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (bp_hit, address) = self.read_u16(self.pc + 1, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        let sp = self.sp.to_le_bytes();

        if self.write(address, sp[0], breakpoints, dbg_mode) {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        if self.write(address + 1, sp[1], breakpoints, dbg_mode) {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.pc += 3;
        self.cycles += 20;
    }

    fn store_u8_to_hl(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (bp_hit, value) = self.read_u8(self.pc + 1, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        if self.write(self.hl, value, breakpoints, dbg_mode) {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.pc += 2;
        self.cycles += 12;
    }

    fn add_i8_to_sp(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (bp_hit, value) = self.read_u8(self.pc + 1, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        let sp = self.sp;
        let value = (value as i8) as u16;
        let result = sp.wrapping_add(value);

        self.sp = result;

        self.set_flag(Flag::Zero(false));
        self.set_flag(Flag::Negative(false));
        self.set_flag(Flag::HalfCarry((sp & 0x0F) + (value & 0x0F) > 0x0F));
        self.set_flag(Flag::Carry((sp & 0x00FF) + (value & 0x00FF) > 0xFF));

        self.pc += 2;
        self.cycles += 16;
    }

    fn halt(&mut self) {
        self.halted = true;

        self.pc += 1;
        self.cycles += 4;
    }

    fn daa(&mut self) {
        let a = self.get_r8(&Register::AF);
        let flag_c = self.get_flag(Flag::Carry(false));
        let flag_n = self.get_flag(Flag::Negative(false));
        let flag_h = self.get_flag(Flag::HalfCarry(false));
        
        let mut daa_correction = 0;

        if flag_h || (!flag_n && (a & 0x0F) > 9) {
            daa_correction = 0x06;
        }

        if flag_c || (!flag_n && a > 0x99) {
            daa_correction |= 0x60;
            self.set_flag(Flag::Carry(true));
        }

        let result = if flag_n {a.wrapping_sub(daa_correction)} else {a.wrapping_add(daa_correction)};

        self.set_r8(Register::AF, result);

        self.set_flag(Flag::Zero(result == 0));
        self.set_flag(Flag::HalfCarry(false));

        self.pc += 1;
        self.cycles += 4;
    }

    fn scf(&mut self) {
        self.set_flag(Flag::Negative(false));
        self.set_flag(Flag::HalfCarry(false));
        self.set_flag(Flag::Carry(true));

        self.pc += 1;
        self.cycles += 4;
    }

    fn ccf(&mut self) {
        let carry = !self.get_flag(Flag::Carry(false));

        self.set_flag(Flag::Negative(false));
        self.set_flag(Flag::HalfCarry(false));
        self.set_flag(Flag::Carry(carry));

        self.pc += 1;
        self.cycles += 4;
    }

    fn cpl(&mut self) {
        let result = !self.get_r8(&Register::AF);
        
        self.set_r8(Register::AF, result);
        
        self.set_flag(Flag::Negative(true));
        self.set_flag(Flag::HalfCarry(true));

        self.pc += 1;
        self.cycles += 4;
    }

    fn pop_rp(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode, reg: Register) {
        let (bp_hit, value) = self.stack_read(breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.set_rp(reg, value);

        self.pc += 1;
        self.cycles += 12;
    }

    fn push_rp(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode, reg: Register) {
        let value = self.get_rp(&reg);

        if self.stack_write(value, breakpoints, dbg_mode) {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.pc += 1;
        self.cycles += 16;
    }

    fn inc_rp(&mut self, reg: Register) {
        let value = self.get_rp(&reg);

        self.set_rp(reg, value.wrapping_add(1));
        
        self.pc += 1;
        self.cycles += 8;
    }

    fn dec_rp(&mut self, reg: Register) {
        let value = self.get_rp(&reg);

        self.set_rp(reg, value.wrapping_sub(1));
        
        self.pc += 1;
        self.cycles += 8;
    }

    fn inc(&mut self, value: u8) -> u8 {
        let result = value.wrapping_add(1);

        self.set_flag(Flag::Zero(result == 0));
        self.set_flag(Flag::Negative(false));
        self.set_flag(Flag::HalfCarry((value & 0x0F) + 1 > 0x0F));

        result
    }

    fn inc_r8(&mut self, reg: Register) {
        let result = self.inc(self.get_r8(&reg));
        self.set_r8(reg, result);
        
        self.pc += 1;
        self.cycles += 4;
    }

    fn inc_hl(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (bp_hit, value) = self.read_u8(self.hl, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        let result = self.inc(value);

        if self.write(self.hl, result, breakpoints, dbg_mode) {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.pc += 1;
        self.cycles += 12;
    }

    fn dec(&mut self, value: u8) -> u8 {
        let result = value.wrapping_sub(1);

        self.set_flag(Flag::Zero(result == 0));
        self.set_flag(Flag::Negative(true));
        self.set_flag(Flag::HalfCarry((value & 0x0F) == 0));

        result
    }

    fn dec_r8(&mut self, reg: Register) {
        let result = self.dec(self.get_r8(&reg));
        self.set_r8(reg, result);

        self.pc += 1;
        self.cycles += 4;
    }

    fn dec_hl(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (bp_hit, value) = self.read_u8(self.hl, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        let result = self.dec(value);

        if self.write(self.hl, result, breakpoints, dbg_mode) {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.pc += 1;
        self.cycles += 12;
    }

    fn add_hl_rp(&mut self, reg: Register) {
        let hl = self.hl;
        let value = self.get_rp(&reg);
        let (result, carry) = hl.overflowing_add(value);

        self.hl = result;

        self.set_flag(Flag::Negative(false));
        self.set_flag(Flag::HalfCarry((hl & 0x0FFF) + (value & 0x0FFF) > 0x0FFF));
        self.set_flag(Flag::Carry(carry));
        
        self.pc += 1;
        self.cycles += 8;
    }

    fn add(&mut self, value: u8) {
        let a = self.get_r8(&Register::AF);
        let result = a as u16 + value as u16;

        self.set_r8(Register::AF, result as u8);

        self.set_flag(Flag::Zero(result as u8 == 0));
        self.set_flag(Flag::Negative(false));
        self.set_flag(Flag::HalfCarry((a & 0x0F) + (value & 0x0F) > 0x0F));
        self.set_flag(Flag::Carry(result > 0xFF));
    }

    fn add_r8(&mut self, reg: Register) {
        let value = self.get_r8(&reg);
        self.add(value);

        self.pc += 1;
        self.cycles += 4;
    }

    fn add_u8(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (bp_hit, value) = self.read_u8(self.pc + 1, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.add(value);

        self.pc += 2;
        self.cycles += 8;
    }

    fn add_hl(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (bp_hit, value) = self.read_u8(self.hl, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.add(value);

        self.pc += 1;
        self.cycles += 8;
    }

    fn adc(&mut self, value: u8) {
        let a = self.get_r8(&Register::AF);
        let carry = if self.get_flag(Flag::Carry(false)) {1} else {0};
        let result = a as u16 + value as u16 + carry as u16;

        self.set_r8(Register::AF, result as u8);

        self.set_flag(Flag::Zero(result as u8 == 0));
        self.set_flag(Flag::Negative(false));
        self.set_flag(Flag::HalfCarry((a & 0x0F) + (value & 0x0F) + carry > 0x0F));
        self.set_flag(Flag::Carry(result > 0xFF));
    }

    fn adc_r8(&mut self, reg: Register) {
        let value = self.get_r8(&reg);
        self.adc(value);

        self.pc += 1;
        self.cycles += 4;
    }

    fn adc_u8(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (bp_hit, value) = self.read_u8(self.pc + 1, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.adc(value);

        self.pc += 2;
        self.cycles += 8;
    }

    fn adc_hl(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (bp_hit, value) = self.read_u8(self.hl, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.adc(value);

        self.pc += 1;
        self.cycles += 8;
    }

    fn sub(&mut self, value: u8) {
        let a = self.get_r8(&Register::AF);
        let result = a.wrapping_sub(value);

        self.set_r8(Register::AF, result);

        self.set_flag(Flag::Zero(result == 0));
        self.set_flag(Flag::Negative(true));
        self.set_flag(Flag::HalfCarry((a & 0x0F) < (value & 0x0F)));
        self.set_flag(Flag::Carry(value > a));
    }

    fn sub_r8(&mut self, reg: Register) {
        let value = self.get_r8(&reg);
        self.sub(value);

        self.pc += 1;
        self.cycles += 4;
    }

    fn sub_u8(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (bp_hit, value) = self.read_u8(self.pc + 1, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.sub(value);

        self.pc += 2;
        self.cycles += 8;
    }

    fn sub_hl(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (bp_hit, value) = self.read_u8(self.hl, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.sub(value);

        self.pc += 1;
        self.cycles += 8;
    }

    fn sbc(&mut self, value: u8) {
        let a = self.get_r8(&Register::AF);
        let carry = if self.get_flag(Flag::Carry(false)) {1} else {0};
        let result = a as i16 - value as i16 - carry as i16;

        self.set_r8(Register::AF, result as u8);

        self.set_flag(Flag::Zero(result as u8 == 0));
        self.set_flag(Flag::Negative(true));
        self.set_flag(Flag::HalfCarry(((a & 0x0F) as i16 - (value & 0x0F) as i16 - carry as i16) < 0));
        self.set_flag(Flag::Carry(result < 0));
    }

    fn sbc_r8(&mut self, reg: Register) {
        let value = self.get_r8(&reg);
        self.sbc(value);

        self.pc += 1;
        self.cycles += 4;
    }

    fn sbc_u8(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (bp_hit, value) = self.read_u8(self.pc + 1, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.sbc(value);

        self.pc += 2;
        self.cycles += 8;
    }

    fn sbc_hl(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (bp_hit, value) = self.read_u8(self.hl, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.sbc(value);

        self.pc += 1;
        self.cycles += 8;
    }

    fn and(&mut self, value: u8) {
        let a = self.get_r8(&Register::AF);
        let result = a & value;

        self.set_r8(Register::AF, result);

        self.set_flag(Flag::Zero(result == 0));
        self.set_flag(Flag::Negative(false));
        self.set_flag(Flag::HalfCarry(true));
        self.set_flag(Flag::Carry(false));
    }

    fn and_r8(&mut self, reg: Register) {
        let value = self.get_r8(&reg);
        self.and(value);

        self.pc += 1;
        self.cycles += 4;
    }

    fn and_u8(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (bp_hit, value) = self.read_u8(self.pc + 1, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.and(value);

        self.pc += 2;
        self.cycles += 8;
    }

    fn and_hl(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (bp_hit, value) = self.read_u8(self.hl, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.and(value);

        self.pc += 1;
        self.cycles += 8;
    }

    fn xor(&mut self, value: u8) {
        let a = self.get_r8(&Register::AF);
        let result = a ^ value;

        self.set_r8(Register::AF, result);

        self.set_flag(Flag::Zero(result == 0));
        self.set_flag(Flag::Negative(false));
        self.set_flag(Flag::HalfCarry(false));
        self.set_flag(Flag::Carry(false));
    }

    fn xor_r8(&mut self, reg: Register) {
        let value = self.get_r8(&reg);
        self.xor(value);

        self.pc += 1;
        self.cycles += 4;
    }

    fn xor_u8(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (bp_hit, value) = self.read_u8(self.pc + 1, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.xor(value);

        self.pc += 2;
        self.cycles += 8;
    }

    fn xor_hl(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (bp_hit, value) = self.read_u8(self.hl, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.xor(value);

        self.pc += 1;
        self.cycles += 8;
    }

    fn or(&mut self, value: u8) {
        let a = self.get_r8(&Register::AF);
        let result = a | value;

        self.set_r8(Register::AF, result);

        self.set_flag(Flag::Zero(result == 0));
        self.set_flag(Flag::Negative(false));
        self.set_flag(Flag::HalfCarry(false));
        self.set_flag(Flag::Carry(false));
    }

    fn or_r8(&mut self, reg: Register) {
        let value = self.get_r8(&reg);
        self.or(value);

        self.pc += 1;
        self.cycles += 4;
    }

    fn or_u8(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (bp_hit, value) = self.read_u8(self.pc + 1, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.or(value);

        self.pc += 2;
        self.cycles += 8;
    }

    fn or_hl(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (bp_hit, value) = self.read_u8(self.hl, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.or(value);

        self.pc += 1;
        self.cycles += 8;
    }

    fn cp(&mut self, value: u8) {
        let a = self.get_r8(&Register::AF);

        self.set_flag(Flag::Zero(a == value));
        self.set_flag(Flag::Negative(true));
        self.set_flag(Flag::HalfCarry((a & 0x0F) < (value & 0x0F)));
        self.set_flag(Flag::Carry(a < value));
    }

    fn cp_r8(&mut self, reg: Register) {
        let value = self.get_r8(&reg);
        self.cp(value);

        self.pc += 1;
        self.cycles += 4;
    }

    fn cp_u8(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (bp_hit, value) = self.read_u8(self.pc + 1, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.cp(value);

        self.pc += 2;
        self.cycles += 8;
    }

    fn cp_hl(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (bp_hit, value) = self.read_u8(self.hl, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.cp(value);

        self.pc += 1;
        self.cycles += 8;
    }

    fn call(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (bp_hit, address) = self.read_u16(self.pc + 1, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        if self.stack_write(self.pc + 3, breakpoints, dbg_mode) {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        if let Ok(mut lock) = self.callstack.write() {
            lock.push(format!("${:04X}: CALL {:04X}", self.pc, address));
        }

        self.pc = address;
        self.cycles += 24;
    }

    fn conditional_call(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode, condition: Condition) {
        if self.check_condition(condition) {
            let (bp_hit, address) = self.read_u16(self.pc + 1, breakpoints, dbg_mode);

            if bp_hit {
                *dbg_mode = EmulatorMode::BreakpointHit;
                return;
            }

            if self.stack_write(self.pc + 3, breakpoints, dbg_mode) {
                *dbg_mode = EmulatorMode::BreakpointHit;
                return;
            }

            if let Ok(mut lock) = self.callstack.write() {
                lock.push(format!("${:04X}: CALL {}, {:04X}", self.pc, condition, address));
            }

            self.pc = address;
            self.cycles += 24;
        }
        else {
            self.pc += 3;
            self.cycles += 12;
        }
    }

    fn ret(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (bp_hit, address) = self.stack_read(breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        if let Ok(mut lock) = self.callstack.write() {
            lock.pop();
        }

        self.pc = address;
        self.cycles += 12;
    }

    fn conditional_ret(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode, condition: Condition) {
        if self.check_condition(condition) {
            let (bp_hit, address) = self.stack_read(breakpoints, dbg_mode);

            if bp_hit {
                *dbg_mode = EmulatorMode::BreakpointHit;
                return;
            }

            if let Ok(mut lock) = self.callstack.write() {
                lock.pop();
            }

            self.pc = address;
            self.cycles += 20;
        }
        else {
            self.pc += 1;
            self.cycles += 8;
        }
    }

    fn reti(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (bp_hit, address) = self.stack_read(breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        if let Ok(mut lock) = self.callstack.write() {
            lock.pop();
        }

        self.interrupt_handler.enable_interrupts(false);

        self.pc = address;
        self.cycles += 16;
    }

    fn jump(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (bp_hit, address) = self.read_u16(self.pc + 1, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.pc = address;
        self.cycles += 16;
    }

    fn conditional_jump(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode, condition: Condition) {
        if self.check_condition(condition) {
            let (bp_hit, address) = self.read_u16(self.pc + 1, breakpoints, dbg_mode);

            if bp_hit {
                *dbg_mode = EmulatorMode::BreakpointHit;
                return;
            }

            self.pc = address;
            self.cycles += 16;
        }
        else {
            self.pc += 3;
            self.cycles += 12;
        }
    }

    fn jump_relative(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (bp_hit, offset) = self.read_u8(self.pc + 1, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        let offset = offset as i8;
        let target = self.pc.wrapping_add(offset as u16) + 2;

        self.pc = target;
        self.cycles += 12;
    }

    fn conditional_jump_relative(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode, condition: Condition) {
        if self.check_condition(condition) {
            let (bp_hit, offset) = self.read_u8(self.pc + 1, breakpoints, dbg_mode);

            if bp_hit {
                *dbg_mode = EmulatorMode::BreakpointHit;
                return;
            }

            let offset = offset as i8;
            let target = self.pc.wrapping_add(offset as u16) + 2;

            self.pc = target;
            self.cycles += 12;
        }
        else {
            self.pc += 2;
            self.cycles += 8;
        }
    }

    fn jump_hl(&mut self) {
        self.pc = self.hl;
        self.cycles += 4;
    }

    fn rst(&mut self, address: u16, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        if self.stack_write(self.pc + 1, breakpoints, dbg_mode) {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        if let Ok(mut lock) = self.callstack.write() {
            lock.push(format!("${:04X}: RST {:04X}", self.pc, address));
        }

        self.pc = address;
        self.cycles += 16;
    }

    fn di(&mut self) {
        self.interrupt_handler.disable_interrupts();

        self.pc += 1;
        self.cycles += 4;
    }

    fn ei(&mut self) {
        self.interrupt_handler.enable_interrupts(true);
        
        self.pc += 1;
        self.cycles += 4;
    }

    fn rla(&mut self) {
        let result = self.rl(self.get_r8(&Register::AF));
        self.set_r8(Register::AF, result);
        // RL sets the Zero flag according to the result. RLA is always 0.
        self.set_flag(Flag::Zero(false));

        self.pc += 1;
        self.cycles += 4;
    }

    fn rra(&mut self) {
        let result = self.rr(self.get_r8(&Register::AF));
        self.set_r8(Register::AF, result);
        // RR sets the Zero flag according to the result. RRA is always 0.
        self.set_flag(Flag::Zero(false));

        self.pc += 1;
        self.cycles += 4;
    }

    fn rlca(&mut self) {
        let result = self.rlc(self.get_r8(&Register::AF));
        self.set_r8(Register::AF, result);
        // RLC sets the Zero flag according to the result. RLCA is always 0.
        self.set_flag(Flag::Zero(false));

        self.pc += 1;
        self.cycles += 4;
    }

    fn rrca(&mut self) {
        let result = self.rrc(self.get_r8(&Register::AF));
        self.set_r8(Register::AF, result);
        // RRC sets the Zero flag according to the result. RRCA is always 0.
        self.set_flag(Flag::Zero(false));

        self.pc += 1;
        self.cycles += 4;
    }

    fn rlc(&mut self, value: u8) -> u8 {
        let new_carry = value & 0x80 != 0;
        let result = value.rotate_left(1);

        self.set_flag(Flag::Zero(result == 0));
        self.set_flag(Flag::Negative(false));
        self.set_flag(Flag::HalfCarry(false));
        self.set_flag(Flag::Carry(new_carry));

        result
    }

    fn rlc_r8(&mut self, reg: Register) {
        let result = self.rlc(self.get_r8(&reg));
        self.set_r8(reg, result);

        self.pc += 2;
        self.cycles += 8;
    }

    fn rlc_hl(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (bp_hit, value) = self.read_u8(self.hl, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        let result = self.rlc(value);

        if self.write(self.hl, result, breakpoints, dbg_mode) {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.pc += 2;
        self.cycles += 16;
    }

    fn rrc(&mut self, value: u8) -> u8 {
        let new_carry = value & 1 != 0;
        let result = value.rotate_right(1);

        self.set_flag(Flag::Zero(result == 0));
        self.set_flag(Flag::Negative(false));
        self.set_flag(Flag::HalfCarry(false));
        self.set_flag(Flag::Carry(new_carry));

        result
    }

    fn rrc_r8(&mut self, reg: Register) {
        let result = self.rrc(self.get_r8(&reg));
        self.set_r8(reg, result);

        self.pc += 2;
        self.cycles += 8;
    }

    fn rrc_hl(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (bp_hit, value) = self.read_u8(self.hl, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        let result = self.rrc(value);

        if self.write(self.hl, result, breakpoints, dbg_mode) {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.pc += 2;
        self.cycles += 16;
    }

    fn rl(&mut self, value: u8) -> u8 {
        let new_carry = (value & 0x80) != 0;
        let current_carry = if self.get_flag(Flag::Carry(false)) {1} else {0};
        let result = (value << 1) | current_carry;

        self.set_flag(Flag::Zero(result == 0));
        self.set_flag(Flag::Negative(false));
        self.set_flag(Flag::HalfCarry(false));
        self.set_flag(Flag::Carry(new_carry));

        result
    }

    fn rl_r8(&mut self, reg: Register) {
        let result = self.rl(self.get_r8(&reg));
        self.set_r8(reg, result);

        self.pc += 2;
        self.cycles += 8;
    }

    fn rl_hl(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (bp_hit, value) = self.read_u8(self.hl, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        let result = self.rl(value);

        if self.write(self.hl, result, breakpoints, dbg_mode) {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.pc += 2;
        self.cycles += 16;
    }

    fn rr(&mut self, value: u8) -> u8 {
        let new_carry = (value & 1) != 0;
        let current_carry = if self.get_flag(Flag::Carry(false)) {1} else {0};
        let result = (value >> 1) | (current_carry << 7);

        self.set_flag(Flag::Zero(result == 0));
        self.set_flag(Flag::Negative(false));
        self.set_flag(Flag::HalfCarry(false));
        self.set_flag(Flag::Carry(new_carry));

        result
    }

    fn rr_r8(&mut self, reg: Register) {
        let result = self.rr(self.get_r8(&reg));
        self.set_r8(reg, result);

        self.pc += 2;
        self.cycles += 8;
    }

    fn rr_hl(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (bp_hit, value) = self.read_u8(self.hl, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }
        
        let result = self.rr(value);

        if self.write(self.hl, result, breakpoints, dbg_mode) {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.pc += 2;
        self.cycles += 16;
    }

    fn sla(&mut self, value: u8) -> u8 {
        let new_carry = (value & 0x80) != 0;
        let result = value << 1;

        self.set_flag(Flag::Zero(result == 0));
        self.set_flag(Flag::Negative(false));
        self.set_flag(Flag::HalfCarry(false));
        self.set_flag(Flag::Carry(new_carry));

        result
    }

    fn sla_r8(&mut self, reg: Register) {
        let result = self.sla(self.get_r8(&reg));
        self.set_r8(reg, result);

        self.pc += 2;
        self.cycles += 8;
    }

    fn sla_hl(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (bp_hit, value) = self.read_u8(self.hl, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }
        
        let result = self.sla(value);

        if self.write(self.hl, result, breakpoints, dbg_mode) {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.pc += 2;
        self.cycles += 16;
    }

    fn sra(&mut self, value: u8) -> u8 {
        let msb = value & 0x80;
        let new_carry = (value & 1) == 1;
        let result = (value >> 1) | msb;

        self.set_flag(Flag::Zero(result == 0));
        self.set_flag(Flag::Negative(false));
        self.set_flag(Flag::HalfCarry(false));
        self.set_flag(Flag::Carry(new_carry));

        result
    }

    fn sra_r8(&mut self, reg: Register) {
        let result = self.sra(self.get_r8(&reg));
        self.set_r8(reg, result);
        
        self.pc += 2;
        self.cycles += 8;
    }

    fn sra_hl(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (bp_hit, value) = self.read_u8(self.hl, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }
        
        let result = self.sra(value);

        if self.write(self.hl, result, breakpoints, dbg_mode) {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.pc += 2;
        self.cycles += 16;
    }

    fn swap(&mut self, value: u8) -> u8 {
        let (hi, low) = (((value & 0xF0) >> 4), (value & 0x0F));
        let result = (low << 4) | hi;

        self.set_flag(Flag::Zero(result == 0));
        self.set_flag(Flag::Negative(false));
        self.set_flag(Flag::HalfCarry(false));
        self.set_flag(Flag::Carry(false));

        result
    }

    fn swap_r8(&mut self, reg: Register) {
        let result = self.swap(self.get_r8(&reg));
        self.set_r8(reg, result);
        
        self.pc += 2;
        self.cycles += 8;
    }

    fn swap_hl(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (bp_hit, value) = self.read_u8(self.hl, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        let result = self.swap(value);

        if self.write(self.hl, result, breakpoints, dbg_mode) {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.pc += 2;
        self.cycles += 16;
    }

    fn srl(&mut self, value: u8) -> u8 {
        let new_carry = (value & 1) != 0;
        let result = value >> 1;

        self.set_flag(Flag::Zero(result == 0));
        self.set_flag(Flag::Negative(false));
        self.set_flag(Flag::HalfCarry(false));
        self.set_flag(Flag::Carry(new_carry));

        result
    }

    fn srl_r8(&mut self, reg: Register) {
        let result = self.srl(self.get_r8(&reg));
        self.set_r8(reg, result);

        self.pc += 2;
        self.cycles += 8;
    }

    fn srl_hl(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode) {
        let (bp_hit, value) = self.read_u8(self.hl, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        let result = self.srl(value);

        if self.write(self.hl, result, breakpoints, dbg_mode) {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.pc += 2;
        self.cycles += 16;
    }

    fn bit(&mut self, value: u8, bit: u8) {
        self.set_flag(Flag::Zero((value >> bit) & 1 == 0));
        self.set_flag(Flag::Negative(false));
        self.set_flag(Flag::HalfCarry(true));
    }

    fn bit_r8(&mut self, reg: Register, bit: u8) {
        self.bit(self.get_r8(&reg), bit);

        self.pc += 2;
        self.cycles += 8;
    }

    fn bit_hl(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode, bit: u8) {
        let (bp_hit, value) = self.read_u8(self.hl, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.bit(value, bit);

        self.pc += 2;
        self.cycles += 12;
    }

    fn res_r8(&mut self, reg: Register, bit: u8) {
        let value = self.get_r8(&reg);
        let result = value & !(1 << bit);

        self.set_r8(reg, result);

        self.pc += 2;
        self.cycles += 8;
    }

    fn res_hl(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode, bit: u8) {
        let (bp_hit, value) = self.read_u8(self.hl, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        let result = value & !(1 << bit);

        if self.write(self.hl, result, breakpoints, dbg_mode) {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.pc += 2;
        self.cycles += 16;
    }

    fn set(&mut self, reg: Register, bit: u8) {
        let value = self.get_r8(&reg);
        let result = value | (1 << bit);

        self.set_r8(reg, result);

        self.pc += 2;
        self.cycles += 8;
    }

    fn set_hl(&mut self, breakpoints: &[Breakpoint], dbg_mode: &mut EmulatorMode, bit: u8) {
        let (bp_hit, value) = self.read_u8(self.hl, breakpoints, dbg_mode);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        let result = value | (1 << bit);

        if self.write(self.hl, result, breakpoints, dbg_mode) {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.pc += 2;
        self.cycles += 16;
    }
}
