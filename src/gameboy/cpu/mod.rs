use super::*;

enum JumpCondition {
    Zero(bool),
    Carry(bool)
}

enum Register {
    AF(bool),
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

    cycles: usize,

    memory: Arc<GameboyMemory>
}

impl GameboyCPU {
    pub fn init(memory: Arc<GameboyMemory>) -> GameboyCPU {
        GameboyCPU {
            af: 0,
            bc: 0,
            de: 0,
            hl: 0,

            sp: 0,
            pc: 0,

            cycles: 0,

            memory
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
            Register::AF(high) => {
                if *high {
                    ((self.af & 0xFF00) >> 8) as u8
                }
                else {
                    (self.af & 0x00FF) as u8
                }
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
            Register::AF(high) => {
                if high {
                    self.af = (self.af & 0x00FF) | (value as u16) << 8;
                }
                else {
                    self.af = (self.af & 0xFF00) | ((value as u16) & 0xFFF0);
                }
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
            Register::AF(_) => self.af,
            Register::BC(_) => self.bc,
            Register::DE(_) => self.de,
            Register::HL(_) => self.hl,
            Register::SP => self.sp
        }
    }

    fn set_rp(&mut self, reg: Register, value: u16) {
        match reg {
            Register::AF(_) => self.af = value & 0xFFF0,
            Register::BC(_) => self.bc = value,
            Register::DE(_) => self.de = value,
            Register::HL(_) => self.hl = value,
            Register::SP => self.sp = value
        }
    }

    pub fn get_all_registers(&self) -> (&u16, &u16, &u16, &u16, &u16, &u16) {
        (&self.af, &self.bc, &self.de, &self.hl, &self.sp, &self.pc)
    }

    fn read_u8(&self, address: u16, breakpoints: &Vec<Breakpoint>) -> (bool, u8) {
        let mut found_bp = false;
        let matching_bps: Vec<&Breakpoint> = breakpoints.iter().filter(|b| *b.address() == address).collect();

        for bp in matching_bps {
            if *bp.read() {
                found_bp = true;
                break;
            }
        }

        (found_bp, self.memory.read(address))
    }

    fn read_u16(&self, address: u16, breakpoints: &Vec<Breakpoint>) -> (bool, u16) {
        let mut found_bp = false;
        let matching_bps: Vec<&Breakpoint> = breakpoints.iter().filter(|b| *b.address() == address || *b.address() == address + 1).collect();

        for bp in matching_bps {
            if *bp.read() {
                found_bp = true;
                break;
            }
        }

        let values = [self.memory.read(address), self.memory.read(address + 1)];

        (found_bp, u16::from_le_bytes(values))
    }

    fn write(&self, address: u16, value: u8, breakpoints: &Vec<Breakpoint>) -> bool {
        let matching_bps: Vec<&Breakpoint> = breakpoints.iter().filter(|b| *b.address() == address).collect();

        for bp in matching_bps {
            if *bp.read() {
                return true;
            }
        }

        self.memory.write(address, value);
        false
    }

    fn stack_read(&mut self, breakpoints: &Vec<Breakpoint>) -> (bool, u16) {
        let mut found_bp = false;
        let matching_bps: Vec<&Breakpoint> = breakpoints.iter().filter(|b| *b.address() == self.sp - 1 || *b.address() == self.sp - 2).collect();

        for bp in matching_bps {
            if *bp.read() {
                found_bp = true;
                break;
            }
        }

        let values = [self.memory.read(self.sp), self.memory.read(self.sp + 1)];
        self.sp += 2;

        (found_bp, u16::from_le_bytes(values))
    }

    fn stack_write(&mut self, value: u16, breakpoints: &Vec<Breakpoint>) -> bool {
        let high = (value >> 8) as u8;
        let low = value as u8;

        self.sp -= 1;
        if self.write(self.sp, high, breakpoints) {
            return true;
        }

        self.sp -= 1;
        if self.write(self.sp, low, breakpoints) {
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
    }

    pub fn cpu_cycle(&mut self, breakpoints: &Vec<Breakpoint>, dbg_mode: &mut EmulatorMode) {
        for bp in breakpoints {
            if self.pc == *bp.address() && *bp.execute() {
                if *dbg_mode != EmulatorMode::Stepping {
                    *dbg_mode = EmulatorMode::BreakpointHit;
                    return;
                }
            }
        }

        self.execute_instruction(breakpoints, dbg_mode);
    }

    fn execute_instruction(&mut self, breakpoints: &Vec<Breakpoint>, dbg_mode: &mut EmulatorMode) {
        let (bp_hit, opcode) = self.read_u8(self.pc, breakpoints);

        if bp_hit && *dbg_mode != EmulatorMode::Stepping {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        match opcode {
            0x00 => self.nop(),
            0x01 => self.load_u16_to_rp(breakpoints, dbg_mode, Register::BC(false)),
            0x03 => self.inc_rp(Register::BC(false)),
            0x04 => self.inc_r8(Register::BC(true)),
            0x05 => self.dec_r8(Register::BC(true)),
            0x06 => self.load_u8_to_r8(breakpoints, dbg_mode, Register::BC(true)),
            0x0A => self.load_a_from_rp(breakpoints, dbg_mode, Register::BC(false)),
            0x0C => self.inc_r8(Register::BC(false)),
            0x0D => self.dec_r8(Register::BC(false)),
            0x0E => self.load_u8_to_r8(breakpoints, dbg_mode, Register::BC(false)),

            0x11 => self.load_u16_to_rp(breakpoints, dbg_mode, Register::DE(false)),
            0x13 => self.inc_rp(Register::DE(false)),
            0x14 => self.inc_r8(Register::DE(true)),
            0x15 => self.dec_r8(Register::DE(true)),
            0x16 => self.load_u8_to_r8(breakpoints, dbg_mode, Register::DE(true)),
            0x17 => self.rla(),
            0x18 => self.jump_relative(breakpoints, dbg_mode),
            0x1A => self.load_a_from_rp(breakpoints, dbg_mode, Register::DE(false)),
            0x1C => self.inc_r8(Register::DE(false)),
            0x1D => self.dec_r8(Register::DE(false)),
            0x1E => self.load_u8_to_r8(breakpoints, dbg_mode, Register::DE(false)),

            0x20 => self.conditional_jump_relative(breakpoints, dbg_mode, JumpCondition::Zero(false)),
            0x21 => self.load_u16_to_rp(breakpoints, dbg_mode, Register::HL(false)),
            0x22 => self.store_to_hl_and_inc(breakpoints, dbg_mode),
            0x23 => self.inc_rp(Register::HL(false)),
            0x24 => self.inc_r8(Register::HL(true)),
            0x25 => self.dec_r8(Register::HL(true)),
            0x26 => self.load_u8_to_r8(breakpoints, dbg_mode, Register::HL(true)),
            0x28 => self.conditional_jump_relative(breakpoints, dbg_mode, JumpCondition::Zero(true)),
            0x2C => self.inc_r8(Register::HL(false)),
            0x2D => self.dec_r8(Register::HL(false)),
            0x2E => self.load_u8_to_r8(breakpoints, dbg_mode, Register::HL(false)),

            0x30 => self.conditional_jump_relative(breakpoints, dbg_mode, JumpCondition::Carry(false)),
            0x31 => self.load_u16_to_rp(breakpoints, dbg_mode, Register::SP),
            0x32 => self.store_to_hl_and_dec(breakpoints, dbg_mode),
            0x33 => self.inc_rp(Register::SP),
            0x38 => self.conditional_jump_relative(breakpoints, dbg_mode, JumpCondition::Carry(true)),
            0x3C => self.inc_r8(Register::AF(true)),
            0x3D => self.dec_r8(Register::AF(true)),
            0x3E => self.load_u8_to_r8(breakpoints, dbg_mode, Register::AF(true)),

            0x40 => self.load_r8_to_r8(Register::BC(true), Register::BC(true)),
            0x41 => self.load_r8_to_r8(Register::BC(true), Register::BC(false)),
            0x42 => self.load_r8_to_r8(Register::BC(true), Register::DE(true)),
            0x43 => self.load_r8_to_r8(Register::BC(true), Register::DE(false)),
            0x44 => self.load_r8_to_r8(Register::BC(true), Register::HL(true)),
            0x45 => self.load_r8_to_r8(Register::BC(true), Register::HL(false)),
            0x47 => self.load_r8_to_r8(Register::BC(true), Register::AF(true)),
            0x48 => self.load_r8_to_r8(Register::BC(false), Register::BC(true)),
            0x49 => self.load_r8_to_r8(Register::BC(false), Register::BC(false)),
            0x4A => self.load_r8_to_r8(Register::BC(false), Register::DE(true)),
            0x4B => self.load_r8_to_r8(Register::BC(false), Register::DE(false)),
            0x4C => self.load_r8_to_r8(Register::BC(false), Register::HL(true)),
            0x4D => self.load_r8_to_r8(Register::BC(false), Register::HL(false)),
            0x4F => self.load_r8_to_r8(Register::BC(false), Register::AF(true)),

            0x50 => self.load_r8_to_r8(Register::DE(true), Register::BC(true)),
            0x51 => self.load_r8_to_r8(Register::DE(true), Register::BC(false)),
            0x52 => self.load_r8_to_r8(Register::DE(true), Register::DE(true)),
            0x53 => self.load_r8_to_r8(Register::DE(true), Register::DE(false)),
            0x54 => self.load_r8_to_r8(Register::DE(true), Register::HL(true)),
            0x55 => self.load_r8_to_r8(Register::DE(true), Register::HL(false)),
            0x57 => self.load_r8_to_r8(Register::DE(true), Register::AF(true)),
            0x58 => self.load_r8_to_r8(Register::DE(false), Register::BC(true)),
            0x59 => self.load_r8_to_r8(Register::DE(false), Register::BC(false)),
            0x5A => self.load_r8_to_r8(Register::DE(false), Register::DE(true)),
            0x5B => self.load_r8_to_r8(Register::DE(false), Register::DE(false)),
            0x5C => self.load_r8_to_r8(Register::DE(false), Register::HL(true)),
            0x5D => self.load_r8_to_r8(Register::DE(false), Register::HL(false)),
            0x5F => self.load_r8_to_r8(Register::DE(false), Register::AF(true)),

            0x60 => self.load_r8_to_r8(Register::HL(true), Register::BC(true)),
            0x61 => self.load_r8_to_r8(Register::HL(true), Register::BC(false)),
            0x62 => self.load_r8_to_r8(Register::HL(true), Register::DE(true)),
            0x63 => self.load_r8_to_r8(Register::HL(true), Register::DE(false)),
            0x64 => self.load_r8_to_r8(Register::HL(true), Register::HL(true)),
            0x65 => self.load_r8_to_r8(Register::HL(true), Register::HL(false)),
            0x67 => self.load_r8_to_r8(Register::HL(true), Register::AF(true)),
            0x68 => self.load_r8_to_r8(Register::HL(false), Register::BC(true)),
            0x69 => self.load_r8_to_r8(Register::HL(false), Register::BC(false)),
            0x6A => self.load_r8_to_r8(Register::HL(false), Register::DE(true)),
            0x6B => self.load_r8_to_r8(Register::HL(false), Register::DE(false)),
            0x6C => self.load_r8_to_r8(Register::HL(false), Register::HL(true)),
            0x6D => self.load_r8_to_r8(Register::HL(false), Register::HL(false)),
            0x6F => self.load_r8_to_r8(Register::HL(false), Register::AF(true)),

            0x70 => self.store_r8_to_hl(breakpoints, dbg_mode, Register::BC(true)),
            0x71 => self.store_r8_to_hl(breakpoints, dbg_mode, Register::BC(false)),
            0x72 => self.store_r8_to_hl(breakpoints, dbg_mode, Register::DE(true)),
            0x73 => self.store_r8_to_hl(breakpoints, dbg_mode, Register::DE(false)),
            0x74 => self.store_r8_to_hl(breakpoints, dbg_mode, Register::HL(true)),
            0x75 => self.store_r8_to_hl(breakpoints, dbg_mode, Register::HL(false)),
            0x77 => self.store_r8_to_hl(breakpoints, dbg_mode, Register::AF(true)),
            0x78 => self.load_r8_to_r8(Register::AF(true), Register::BC(true)),
            0x79 => self.load_r8_to_r8(Register::AF(true), Register::BC(false)),
            0x7A => self.load_r8_to_r8(Register::AF(true), Register::DE(true)),
            0x7B => self.load_r8_to_r8(Register::AF(true), Register::DE(false)),
            0x7C => self.load_r8_to_r8(Register::AF(true), Register::HL(true)),
            0x7D => self.load_r8_to_r8(Register::AF(true), Register::HL(false)),
            0x7F => self.load_r8_to_r8(Register::AF(true), Register::AF(true)),

            0xA8 => self.xor_r8(Register::BC(true)),
            0xA9 => self.xor_r8(Register::BC(false)),
            0xAA => self.xor_r8(Register::DE(true)),
            0xAB => self.xor_r8(Register::DE(false)),
            0xAC => self.xor_r8(Register::HL(true)),
            0xAD => self.xor_r8(Register::HL(false)),
            0xAF => self.xor_r8(Register::AF(true)),

            0xC1 => self.pop_rp(breakpoints, dbg_mode, Register::BC(false)),
            0xC5 => self.push_rp(breakpoints, dbg_mode, Register::BC(false)),
            0xC9 => self.ret(breakpoints, dbg_mode),
            0xCB => self.execute_instruction_prefixed(breakpoints, dbg_mode),
            0xCD => self.call(breakpoints, dbg_mode),

            0xD1 => self.pop_rp(breakpoints, dbg_mode, Register::DE(false)),
            0xD5 => self.push_rp(breakpoints, dbg_mode, Register::DE(false)),

            0xE0 => self.store_a_to_io_u8(breakpoints, dbg_mode),
            0xE1 => self.pop_rp(breakpoints, dbg_mode, Register::HL(false)),
            0xE2 => self.store_a_to_io_c(breakpoints, dbg_mode),
            0xE5 => self.push_rp(breakpoints, dbg_mode, Register::HL(false)),
            0xEA => self.store_a_to_u16(breakpoints, dbg_mode),

            0xF1 => self.pop_rp(breakpoints, dbg_mode, Register::AF(false)),
            0xF5 => self.push_rp(breakpoints, dbg_mode, Register::AF(false)),
            0xFE => self.cp_u8(breakpoints, dbg_mode),

            _ => *dbg_mode = EmulatorMode::UnknownInstruction(false, opcode)
        }
    }

    fn execute_instruction_prefixed(&mut self, breakpoints: &Vec<Breakpoint>, dbg_mode: &mut EmulatorMode) {
        let (bp_hit, opcode) = self.read_u8(self.pc + 1, breakpoints);

        if bp_hit && *dbg_mode != EmulatorMode::Stepping {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        match opcode {
            0x10 => self.rl(Register::BC(true)),
            0x11 => self.rl(Register::BC(false)),
            0x12 => self.rl(Register::DE(true)),
            0x13 => self.rl(Register::DE(false)),
            0x14 => self.rl(Register::HL(true)),
            0x15 => self.rl(Register::HL(false)),
            0x17 => self.rl(Register::AF(true)),

            0x40 => self.bit(Register::BC(true), 0),
            0x41 => self.bit(Register::BC(false), 0),
            0x42 => self.bit(Register::DE(true), 0),
            0x43 => self.bit(Register::DE(false), 0),
            0x44 => self.bit(Register::HL(true), 0),
            0x45 => self.bit(Register::HL(false), 0),
            0x47 => self.bit(Register::AF(true), 0),
            0x48 => self.bit(Register::BC(true), 1),
            0x49 => self.bit(Register::BC(false), 1),
            0x4A => self.bit(Register::DE(true), 1),
            0x4B => self.bit(Register::DE(false), 1),
            0x4C => self.bit(Register::HL(true), 1),
            0x4D => self.bit(Register::HL(false), 1),
            0x4F => self.bit(Register::AF(true), 1),

            0x50 => self.bit(Register::BC(true), 2),
            0x51 => self.bit(Register::BC(false), 2),
            0x52 => self.bit(Register::DE(true), 2),
            0x53 => self.bit(Register::DE(false), 2),
            0x54 => self.bit(Register::HL(true), 2),
            0x55 => self.bit(Register::HL(false), 2),
            0x57 => self.bit(Register::AF(true), 2),
            0x58 => self.bit(Register::BC(true), 3),
            0x59 => self.bit(Register::BC(false), 3),
            0x5A => self.bit(Register::DE(true), 3),
            0x5B => self.bit(Register::DE(false), 3),
            0x5C => self.bit(Register::HL(true), 3),
            0x5D => self.bit(Register::HL(false), 3),
            0x5F => self.bit(Register::AF(true), 3),

            0x60 => self.bit(Register::BC(true), 4),
            0x61 => self.bit(Register::BC(false), 4),
            0x62 => self.bit(Register::DE(true), 4),
            0x63 => self.bit(Register::DE(false), 4),
            0x64 => self.bit(Register::HL(true), 4),
            0x65 => self.bit(Register::HL(false), 4),
            0x67 => self.bit(Register::AF(true), 4),
            0x68 => self.bit(Register::BC(true), 5),
            0x69 => self.bit(Register::BC(false), 5),
            0x6A => self.bit(Register::DE(true), 5),
            0x6B => self.bit(Register::DE(false), 5),
            0x6C => self.bit(Register::HL(true), 5),
            0x6D => self.bit(Register::HL(false), 5),
            0x6F => self.bit(Register::AF(true), 5),

            0x70 => self.bit(Register::BC(true), 6),
            0x71 => self.bit(Register::BC(false), 6),
            0x72 => self.bit(Register::DE(true), 6),
            0x73 => self.bit(Register::DE(false), 6),
            0x74 => self.bit(Register::HL(true), 6),
            0x75 => self.bit(Register::HL(false), 6),
            0x77 => self.bit(Register::AF(true), 6),
            0x78 => self.bit(Register::BC(true), 7),
            0x79 => self.bit(Register::BC(false), 7),
            0x7A => self.bit(Register::DE(true), 7),
            0x7B => self.bit(Register::DE(false), 7),
            0x7C => self.bit(Register::HL(true), 7),
            0x7D => self.bit(Register::HL(false), 7),
            0x7F => self.bit(Register::AF(true), 7),

            _ => *dbg_mode = EmulatorMode::UnknownInstruction(true, opcode)
        }
    }

    fn nop(&mut self) {
        self.pc += 1;
        self.cycles += 4;
    }

    fn load_u8_to_r8(&mut self, breakpoints: &Vec<Breakpoint>, dbg_mode: &mut EmulatorMode, reg: Register) {
        let (bp_hit, value) = self.read_u8(self.pc + 1, breakpoints);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.set_r8(reg, value);

        self.pc += 2;
        self.cycles += 8;
    }

    fn load_u8_to_r8_from_hl(&mut self, breakpoints: &Vec<Breakpoint>, dbg_mode: &mut EmulatorMode, reg: Register) {
        let (bp_hit, value) = self.read_u8(self.get_rp(&Register::HL(false)), breakpoints);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.set_r8(reg, value);

        self.pc += 1;
        self.cycles += 8;
    }

    fn load_a_from_rp(&mut self, breakpoints: &Vec<Breakpoint>, dbg_mode: &mut EmulatorMode, reg: Register) {
        let address = self.get_rp(&reg);
        let (bp_hit, value) = self.read_u8(address, breakpoints);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.set_r8(Register::AF(true), value);

        self.pc += 1;
        self.cycles += 8;
    }

    fn load_u16_to_rp(&mut self, breakpoints: &Vec<Breakpoint>, dbg_mode: &mut EmulatorMode, reg: Register) {
        let (bp_hit, value) = self.read_u16(self.pc + 1, breakpoints);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.set_rp(reg, value);

        self.pc += 3;
        self.cycles += 12;
    }

    fn load_r8_to_r8(&mut self, target: Register, source: Register) {
        self.set_r8(target, self.get_r8(&source));
        
        self.pc += 1;
        self.cycles += 4;
    }

    fn store_r8_to_hl(&mut self, breakpoints: &Vec<Breakpoint>, dbg_mode: &mut EmulatorMode, reg: Register) {
        let value = self.get_r8(&reg);
        let address = self.get_rp(&Register::HL(false));
        
        if self.write(address, value, breakpoints) {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }
        
        self.pc += 1;
        self.cycles += 8;
    }

    fn store_to_hl_and_inc(&mut self, breakpoints: &Vec<Breakpoint>, dbg_mode: &mut EmulatorMode) {
        let value = self.get_r8(&Register::AF(true));
        let address = self.get_rp(&Register::HL(false));
        
        if self.write(address, value, breakpoints) {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }
        
        self.set_rp(Register::HL(false), address.wrapping_add(1));

        self.pc += 1;
        self.cycles += 8;
    }

    fn store_to_hl_and_dec(&mut self, breakpoints: &Vec<Breakpoint>, dbg_mode: &mut EmulatorMode) {
        let value = self.get_r8(&Register::AF(true));
        let address = self.get_rp(&Register::HL(false));
        
        if self.write(address, value, breakpoints) {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }
        
        self.set_rp(Register::HL(false), address.wrapping_sub(1));

        self.pc += 1;
        self.cycles += 8;
    }

    fn store_a_to_io_c(&mut self, breakpoints: &Vec<Breakpoint>, dbg_mode: &mut EmulatorMode) {
        let value = self.get_r8(&Register::AF(true));
        let address = 0xFF00 + self.get_r8(&Register::BC(false)) as u16;

        if self.write(address, value, breakpoints) {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }
        
        self.pc += 1;
        self.cycles += 8;
    }

    fn store_a_to_io_u8(&mut self, breakpoints: &Vec<Breakpoint>, dbg_mode: &mut EmulatorMode) {
        let (bp_hit, offset) = self.read_u8(self.pc + 1, breakpoints);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        let address = 0xFF00 + offset as u16;
        let value = self.get_r8(&Register::AF(true));

        if self.write(address, value, breakpoints) {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }
        
        self.pc += 2;
        self.cycles += 12;
    }

    fn store_a_to_u16(&mut self, breakpoints: &Vec<Breakpoint>, dbg_mode: &mut EmulatorMode) {
        let (bp_hit, address) = self.read_u16(self.pc + 1, breakpoints);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.write(address, self.get_r8(&Register::AF(true)), breakpoints);

        self.pc += 3;
        self.cycles += 16;
    }

    fn pop_rp(&mut self, breakpoints: &Vec<Breakpoint>, dbg_mode: &mut EmulatorMode, reg: Register) {
        let (bp_hit, value) = self.stack_read(breakpoints);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.set_rp(reg, value);

        self.pc += 1;
        self.cycles += 12;
    }

    fn push_rp(&mut self, breakpoints: &Vec<Breakpoint>, dbg_mode: &mut EmulatorMode, reg: Register) {
        let value = self.get_rp(&reg);

        if self.stack_write(value, breakpoints) {
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

    fn inc_r8(&mut self, reg: Register) {
        let value = self.get_r8(&reg);
        let result = value.wrapping_add(1);

        let zero = result == 0;
        let half_carry = (value & 0x0F) + 1 > 0x0F;

        self.set_r8(reg, result);
        
        self.set_flag(Flag::Zero(zero));
        self.set_flag(Flag::Negative(false));
        self.set_flag(Flag::HalfCarry(half_carry));

        self.pc += 1;
        self.cycles += 4;
    }

    fn dec_r8(&mut self, reg: Register) {
        let value = self.get_r8(&reg);
        let result = value.wrapping_sub(1);

        let zero = result == 0;
        let half_carry = (value & 0x0F) < 1;

        self.set_r8(reg, result);

        self.set_flag(Flag::Zero(zero));
        self.set_flag(Flag::Negative(true));
        self.set_flag(Flag::HalfCarry(half_carry));

        self.pc += 1;
        self.cycles += 4;
    }

    fn xor_r8(&mut self, reg: Register) {
        let value = self.get_r8(&reg);
        let target = self.get_r8(&Register::AF(true));

        let result = value ^ target;

        self.set_r8(Register::AF(true), result);

        self.set_flag(Flag::Zero(result == 0));
        self.set_flag(Flag::Negative(false));
        self.set_flag(Flag::HalfCarry(false));
        self.set_flag(Flag::Carry(false));

        self.pc += 1;
        self.cycles += 4;
    }

    fn cp_u8(&mut self, breakpoints: &Vec<Breakpoint>, dbg_mode: &mut EmulatorMode) {
        let (bp_hit, value) = self.read_u8(self.pc + 1, breakpoints);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        let reg = self.get_r8(&Register::AF(true));
        let result = reg.wrapping_sub(value);

        self.set_flag(Flag::Zero(result == 0));
        self.set_flag(Flag::Negative(true));
        self.set_flag(Flag::HalfCarry((reg & 0x0F) < (value & 0x0F)));
        self.set_flag(Flag::Carry(reg > value));

        self.pc += 2;
        self.cycles += 8;
    }

    fn call(&mut self, breakpoints: &Vec<Breakpoint>, dbg_mode: &mut EmulatorMode) {
        let (bp_hit, address) = self.read_u16(self.pc + 1, breakpoints);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        if self.stack_write(self.pc + 3, breakpoints) {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.pc = address;
        self.cycles += 24;
    }

    fn ret(&mut self, breakpoints: &Vec<Breakpoint>, dbg_mode: &mut EmulatorMode) {
        let (bp_hit, address) = self.stack_read(breakpoints);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        self.pc = address;
        self.cycles += 12;
    }

    fn jump_relative(&mut self, breakpoints: &Vec<Breakpoint>, dbg_mode: &mut EmulatorMode) {
        let (bp_hit, offset) = self.read_u8(self.pc + 1, breakpoints);

        if bp_hit {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        let offset = offset as i8;
        let target = self.pc.wrapping_add(offset as u16) + 2;

        self.pc = target;
        self.cycles += 12;
    }

    fn conditional_jump_relative(&mut self, breakpoints: &Vec<Breakpoint>, dbg_mode: &mut EmulatorMode, condition: JumpCondition) {
        let jump: bool;

        match condition {
            JumpCondition::Zero(set) => {
                let zf = self.get_flag(Flag::Zero(false));

                if set {
                    jump = zf;
                }
                else {
                    jump = !zf;
                }
            }
            JumpCondition::Carry(set) => {
                let cf = self.get_flag(Flag::Zero(false));

                if set {
                    jump = cf;
                }
                else {
                    jump = !cf;
                }
            }
        }

        if jump {
            let (bp_hit, offset) = self.read_u8(self.pc + 1, breakpoints);

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

    fn rla(&mut self) {
        let value = self.get_r8(&Register::AF(true));
        let top_bit = (value >> 7) == 1;
        let carry = self.get_flag(Flag::Carry(false));
        
        let mut result = value << 1;

        if carry {
            result |= 1;
        }

        self.set_r8(Register::AF(true), result);

        self.set_flag(Flag::Zero(false));
        self.set_flag(Flag::Negative(false));
        self.set_flag(Flag::HalfCarry(false));
        self.set_flag(Flag::Carry(top_bit));

        self.pc += 1;
        self.cycles += 4;
    }

    fn rl(&mut self, reg: Register) {
        let value = self.get_r8(&reg);
        let top_bit = (value >> 7) == 1;
        let carry = self.get_flag(Flag::Carry(false));
        
        let mut result = value << 1;

        if carry {
            result |= 1;
        }

        self.set_r8(reg, result);

        self.set_flag(Flag::Zero(result == 0));
        self.set_flag(Flag::Negative(false));
        self.set_flag(Flag::HalfCarry(false));
        self.set_flag(Flag::Carry(top_bit));

        self.pc += 2;
        self.cycles += 8;
    }

    fn bit(&mut self, reg: Register, bit: u8) {
        let value = self.get_r8(&reg);
        let result = (value & (1 << bit)) == 0;

        self.set_flag(Flag::Zero(result));
        self.set_flag(Flag::Negative(false));
        self.set_flag(Flag::HalfCarry(true));

        self.pc += 2;
        self.cycles += 8;
    }
}