use super::*;

enum JumpCondition {
    Zero(bool),
    Carry(bool)
}

enum TargetRegister {
    AF(bool),
    BC(bool),
    DE(bool),
    HL(bool),
    SP
}

enum TargetFlag {
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

    fn get_flag(&self, flag: TargetFlag) -> bool {
        match flag {
            TargetFlag::Zero(_) => (self.af & 0x80) != 0,
            TargetFlag::Negative(_) => (self.af & 0x40) != 0,
            TargetFlag::HalfCarry(_) => (self.af & 0x20) != 0,
            TargetFlag::Carry(_) => (self.af & 0x10) != 0,
        }
    }

    fn set_flag(&mut self, flag: TargetFlag) {
        let mut flags = self.af & 0x00FF;

        match flag {
            TargetFlag::Zero(value) => {
                if value {
                    flags |= 1 << 7;
                }
                else {
                    flags &= !(1 << 7);
                }
            }
            TargetFlag::Negative(value) => {
                if value {
                    flags |= 1 << 6;
                }
                else {
                    flags &= !(1 << 6);
                }
            }
            TargetFlag::HalfCarry(value) => {
                if value {
                    flags |= 1 << 5;
                }
                else {
                    flags &= !(1 << 5);
                }
            }
            TargetFlag::Carry(value) => {
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

    fn get_register(&self, reg: &TargetRegister) -> u8 {
        match reg {
            TargetRegister::AF(high) => {
                if *high {
                    ((self.af & 0xFF00) >> 8) as u8
                }
                else {
                    (self.af & 0x00FF) as u8
                }
            }
            TargetRegister::BC(high) => {
                if *high {
                    ((self.bc & 0xFF00) >> 8) as u8
                }
                else {
                    (self.bc & 0x00FF) as u8
                }
            }
            TargetRegister::DE(high) => {
                if *high {
                    ((self.de & 0xFF00) >> 8) as u8
                }
                else {
                    (self.de & 0x00FF) as u8
                }
            }
            TargetRegister::HL(high) => {
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

    fn set_register(&mut self, reg: TargetRegister, value: u8) {
        match reg {
            TargetRegister::AF(high) => {
                if high {
                    self.af = (self.af & 0x00FF) | (value as u16) << 8;
                }
                else {
                    self.af = (self.af & 0xFF00) | ((value as u16) & 0xFFF0);
                }
            }
            TargetRegister::BC(high) => {
                if high {
                    self.bc = (self.bc & 0x00FF) | (value as u16) << 8;
                }
                else {
                    self.bc = (self.bc & 0xFF00) | value as u16;
                }
            }
            TargetRegister::DE(high) => {
                if high {
                    self.de = (self.de & 0x00FF) | (value as u16) << 8;
                }
                else {
                    self.de = (self.de & 0xFF00) | value as u16;
                }
            }
            TargetRegister::HL(high) => {
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

        let values = [self.memory.read(self.sp - 1), self.memory.read(self.sp - 2)];
        self.sp -= 2;

        (found_bp, u16::from_le_bytes(values))
    }

    fn stack_write(&mut self, value: u16, breakpoints: &Vec<Breakpoint>) -> bool {
        let bytes = value.to_le_bytes();

        self.sp -= 1;
        if self.write(self.sp, bytes[0], breakpoints) {
            return true;
        }

        self.sp -= 1;
        if self.write(self.sp, bytes[1], breakpoints) {
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
            0x01 => self.load_u16_to_register(breakpoints, dbg_mode, TargetRegister::BC(false)),
            0x04 => self.inc_register(TargetRegister::BC(true)),
            0x06 => self.load_u8_to_register(breakpoints, dbg_mode, TargetRegister::BC(true)),
            0x0A => self.load_a_from_register(breakpoints, dbg_mode, TargetRegister::BC(false)),
            0x0C => self.inc_register(TargetRegister::BC(false)),
            0x0E => self.load_u8_to_register(breakpoints, dbg_mode, TargetRegister::BC(false)),

            0x11 => self.load_u16_to_register(breakpoints, dbg_mode, TargetRegister::DE(false)),
            0x14 => self.inc_register(TargetRegister::DE(true)),
            0x16 => self.load_u8_to_register(breakpoints, dbg_mode, TargetRegister::DE(true)),
            0x1A => self.load_a_from_register(breakpoints, dbg_mode, TargetRegister::DE(false)),
            0x1C => self.inc_register(TargetRegister::DE(false)),
            0x1E => self.load_u8_to_register(breakpoints, dbg_mode, TargetRegister::DE(false)),

            0x20 => self.conditional_jump_relative(breakpoints, dbg_mode, JumpCondition::Zero(false)),
            0x21 => self.load_u16_to_register(breakpoints, dbg_mode, TargetRegister::HL(false)),
            0x24 => self.inc_register(TargetRegister::HL(true)),
            0x26 => self.load_u8_to_register(breakpoints, dbg_mode, TargetRegister::HL(true)),
            0x28 => self.conditional_jump_relative(breakpoints, dbg_mode, JumpCondition::Zero(true)),
            0x2C => self.inc_register(TargetRegister::HL(false)),
            0x2E => self.load_u8_to_register(breakpoints, dbg_mode, TargetRegister::HL(false)),

            0x30 => self.conditional_jump_relative(breakpoints, dbg_mode, JumpCondition::Carry(false)),
            0x31 => self.load_u16_to_register(breakpoints, dbg_mode, TargetRegister::SP),
            0x32 => self.store_to_hl_and_dec(breakpoints, dbg_mode),
            0x38 => self.conditional_jump_relative(breakpoints, dbg_mode, JumpCondition::Carry(true)),
            0x3C => self.inc_register(TargetRegister::AF(true)),
            0x3E => self.load_u8_to_register(breakpoints, dbg_mode, TargetRegister::AF(true)),

            0x40 => self.load_register_to_register(TargetRegister::BC(true), TargetRegister::BC(true)),
            0x41 => self.load_register_to_register(TargetRegister::BC(true), TargetRegister::BC(false)),
            0x42 => self.load_register_to_register(TargetRegister::BC(true), TargetRegister::DE(true)),
            0x43 => self.load_register_to_register(TargetRegister::BC(true), TargetRegister::DE(false)),
            0x44 => self.load_register_to_register(TargetRegister::BC(true), TargetRegister::HL(true)),
            0x45 => self.load_register_to_register(TargetRegister::BC(true), TargetRegister::HL(false)),
            0x47 => self.load_register_to_register(TargetRegister::BC(true), TargetRegister::AF(true)),
            0x48 => self.load_register_to_register(TargetRegister::BC(false), TargetRegister::BC(true)),
            0x49 => self.load_register_to_register(TargetRegister::BC(false), TargetRegister::BC(false)),
            0x4A => self.load_register_to_register(TargetRegister::BC(false), TargetRegister::DE(true)),
            0x4B => self.load_register_to_register(TargetRegister::BC(false), TargetRegister::DE(false)),
            0x4C => self.load_register_to_register(TargetRegister::BC(false), TargetRegister::HL(true)),
            0x4D => self.load_register_to_register(TargetRegister::BC(false), TargetRegister::HL(false)),
            0x4F => self.load_register_to_register(TargetRegister::BC(false), TargetRegister::AF(true)),

            0x50 => self.load_register_to_register(TargetRegister::DE(true), TargetRegister::BC(true)),
            0x51 => self.load_register_to_register(TargetRegister::DE(true), TargetRegister::BC(false)),
            0x52 => self.load_register_to_register(TargetRegister::DE(true), TargetRegister::DE(true)),
            0x53 => self.load_register_to_register(TargetRegister::DE(true), TargetRegister::DE(false)),
            0x54 => self.load_register_to_register(TargetRegister::DE(true), TargetRegister::HL(true)),
            0x55 => self.load_register_to_register(TargetRegister::DE(true), TargetRegister::HL(false)),
            0x57 => self.load_register_to_register(TargetRegister::DE(true), TargetRegister::AF(true)),
            0x58 => self.load_register_to_register(TargetRegister::DE(false), TargetRegister::BC(true)),
            0x59 => self.load_register_to_register(TargetRegister::DE(false), TargetRegister::BC(false)),
            0x5A => self.load_register_to_register(TargetRegister::DE(false), TargetRegister::DE(true)),
            0x5B => self.load_register_to_register(TargetRegister::DE(false), TargetRegister::DE(false)),
            0x5C => self.load_register_to_register(TargetRegister::DE(false), TargetRegister::HL(true)),
            0x5D => self.load_register_to_register(TargetRegister::DE(false), TargetRegister::HL(false)),
            0x5F => self.load_register_to_register(TargetRegister::DE(false), TargetRegister::AF(true)),

            0x60 => self.load_register_to_register(TargetRegister::HL(true), TargetRegister::BC(true)),
            0x61 => self.load_register_to_register(TargetRegister::HL(true), TargetRegister::BC(false)),
            0x62 => self.load_register_to_register(TargetRegister::HL(true), TargetRegister::DE(true)),
            0x63 => self.load_register_to_register(TargetRegister::HL(true), TargetRegister::DE(false)),
            0x64 => self.load_register_to_register(TargetRegister::HL(true), TargetRegister::HL(true)),
            0x65 => self.load_register_to_register(TargetRegister::HL(true), TargetRegister::HL(false)),
            0x67 => self.load_register_to_register(TargetRegister::HL(true), TargetRegister::AF(true)),
            0x68 => self.load_register_to_register(TargetRegister::HL(false), TargetRegister::BC(true)),
            0x69 => self.load_register_to_register(TargetRegister::HL(false), TargetRegister::BC(false)),
            0x6A => self.load_register_to_register(TargetRegister::HL(false), TargetRegister::DE(true)),
            0x6B => self.load_register_to_register(TargetRegister::HL(false), TargetRegister::DE(false)),
            0x6C => self.load_register_to_register(TargetRegister::HL(false), TargetRegister::HL(true)),
            0x6D => self.load_register_to_register(TargetRegister::HL(false), TargetRegister::HL(false)),
            0x6F => self.load_register_to_register(TargetRegister::HL(false), TargetRegister::AF(true)),

            0x70 => self.store_register_to_hl(breakpoints, dbg_mode, TargetRegister::BC(true)),
            0x71 => self.store_register_to_hl(breakpoints, dbg_mode, TargetRegister::BC(false)),
            0x72 => self.store_register_to_hl(breakpoints, dbg_mode, TargetRegister::DE(true)),
            0x73 => self.store_register_to_hl(breakpoints, dbg_mode, TargetRegister::DE(false)),
            0x74 => self.store_register_to_hl(breakpoints, dbg_mode, TargetRegister::HL(true)),
            0x75 => self.store_register_to_hl(breakpoints, dbg_mode, TargetRegister::HL(false)),
            0x77 => self.store_register_to_hl(breakpoints, dbg_mode, TargetRegister::AF(true)),
            0x78 => self.load_register_to_register(TargetRegister::AF(true), TargetRegister::BC(true)),
            0x79 => self.load_register_to_register(TargetRegister::AF(true), TargetRegister::BC(false)),
            0x7A => self.load_register_to_register(TargetRegister::AF(true), TargetRegister::DE(true)),
            0x7B => self.load_register_to_register(TargetRegister::AF(true), TargetRegister::DE(false)),
            0x7C => self.load_register_to_register(TargetRegister::AF(true), TargetRegister::HL(true)),
            0x7D => self.load_register_to_register(TargetRegister::AF(true), TargetRegister::HL(false)),
            0x7F => self.load_register_to_register(TargetRegister::AF(true), TargetRegister::AF(true)),

            0xA8 => self.xor_register(TargetRegister::BC(true)),
            0xA9 => self.xor_register(TargetRegister::BC(false)),
            0xAA => self.xor_register(TargetRegister::DE(true)),
            0xAB => self.xor_register(TargetRegister::DE(false)),
            0xAC => self.xor_register(TargetRegister::HL(true)),
            0xAD => self.xor_register(TargetRegister::HL(false)),
            0xAF => self.xor_register(TargetRegister::AF(true)),

            0xCB => self.execute_instruction_prefixed(breakpoints, dbg_mode),
            0xCD => self.call(breakpoints, dbg_mode),

            0xE0 => self.store_a_to_io_u8(breakpoints, dbg_mode),
            0xE2 => self.store_a_to_io_c(breakpoints, dbg_mode),

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
            0x40 => self.bit_register(TargetRegister::BC(true), 0),
            0x41 => self.bit_register(TargetRegister::BC(false), 0),
            0x42 => self.bit_register(TargetRegister::DE(true), 0),
            0x43 => self.bit_register(TargetRegister::DE(false), 0),
            0x44 => self.bit_register(TargetRegister::HL(true), 0),
            0x45 => self.bit_register(TargetRegister::HL(false), 0),
            0x47 => self.bit_register(TargetRegister::AF(true), 0),
            0x48 => self.bit_register(TargetRegister::BC(true), 1),
            0x49 => self.bit_register(TargetRegister::BC(false), 1),
            0x4A => self.bit_register(TargetRegister::DE(true), 1),
            0x4B => self.bit_register(TargetRegister::DE(false), 1),
            0x4C => self.bit_register(TargetRegister::HL(true), 1),
            0x4D => self.bit_register(TargetRegister::HL(false), 1),
            0x4F => self.bit_register(TargetRegister::AF(true), 1),

            0x50 => self.bit_register(TargetRegister::BC(true), 2),
            0x51 => self.bit_register(TargetRegister::BC(false), 2),
            0x52 => self.bit_register(TargetRegister::DE(true), 2),
            0x53 => self.bit_register(TargetRegister::DE(false), 2),
            0x54 => self.bit_register(TargetRegister::HL(true), 2),
            0x55 => self.bit_register(TargetRegister::HL(false), 2),
            0x57 => self.bit_register(TargetRegister::AF(true), 2),
            0x58 => self.bit_register(TargetRegister::BC(true), 3),
            0x59 => self.bit_register(TargetRegister::BC(false), 3),
            0x5A => self.bit_register(TargetRegister::DE(true), 3),
            0x5B => self.bit_register(TargetRegister::DE(false), 3),
            0x5C => self.bit_register(TargetRegister::HL(true), 3),
            0x5D => self.bit_register(TargetRegister::HL(false), 3),
            0x5F => self.bit_register(TargetRegister::AF(true), 3),

            0x60 => self.bit_register(TargetRegister::BC(true), 4),
            0x61 => self.bit_register(TargetRegister::BC(false), 4),
            0x62 => self.bit_register(TargetRegister::DE(true), 4),
            0x63 => self.bit_register(TargetRegister::DE(false), 4),
            0x64 => self.bit_register(TargetRegister::HL(true), 4),
            0x65 => self.bit_register(TargetRegister::HL(false), 4),
            0x67 => self.bit_register(TargetRegister::AF(true), 4),
            0x68 => self.bit_register(TargetRegister::BC(true), 5),
            0x69 => self.bit_register(TargetRegister::BC(false), 5),
            0x6A => self.bit_register(TargetRegister::DE(true), 5),
            0x6B => self.bit_register(TargetRegister::DE(false), 5),
            0x6C => self.bit_register(TargetRegister::HL(true), 5),
            0x6D => self.bit_register(TargetRegister::HL(false), 5),
            0x6F => self.bit_register(TargetRegister::AF(true), 5),

            0x70 => self.bit_register(TargetRegister::BC(true), 6),
            0x71 => self.bit_register(TargetRegister::BC(false), 6),
            0x72 => self.bit_register(TargetRegister::DE(true), 6),
            0x73 => self.bit_register(TargetRegister::DE(false), 6),
            0x74 => self.bit_register(TargetRegister::HL(true), 6),
            0x75 => self.bit_register(TargetRegister::HL(false), 6),
            0x77 => self.bit_register(TargetRegister::AF(true), 6),
            0x78 => self.bit_register(TargetRegister::BC(true), 7),
            0x79 => self.bit_register(TargetRegister::BC(false), 7),
            0x7A => self.bit_register(TargetRegister::DE(true), 7),
            0x7B => self.bit_register(TargetRegister::DE(false), 7),
            0x7C => self.bit_register(TargetRegister::HL(true), 7),
            0x7D => self.bit_register(TargetRegister::HL(false), 7),
            0x7F => self.bit_register(TargetRegister::AF(true), 7),

            _ => *dbg_mode = EmulatorMode::UnknownInstruction(true, opcode)
        }
    }

    fn nop(&mut self) {
        self.pc += 1;
        self.cycles += 4;
    }

    fn load_u8_to_register(&mut self, bp: &Vec<Breakpoint>, dbg: &mut EmulatorMode, reg: TargetRegister) {
        let (bp_hit, value) = self.read_u8(self.pc + 1, bp);

        if bp_hit {
            *dbg = EmulatorMode::BreakpointHit;
            return;
        }

        self.set_register(reg, value);

        self.pc += 2;
        self.cycles += 8;
    }

    fn load_u8_to_register_from_hl(&mut self, bp: &Vec<Breakpoint>, dbg: &mut EmulatorMode, reg: TargetRegister) {
        let (bp_hit, value) = self.read_u8(self.hl, bp);

        if bp_hit {
            *dbg = EmulatorMode::BreakpointHit;
            return;
        }

        self.set_register(reg, value);

        self.pc += 1;
        self.cycles += 8;
    }

    fn load_a_from_register(&mut self, bp: &Vec<Breakpoint>, dbg: &mut EmulatorMode, reg: TargetRegister) {
        let address = {
            match reg {
                TargetRegister::BC(_) => self.bc,
                TargetRegister::DE(_) => self.de,
                _ => unreachable!()
            }
        };
        let (bp_hit, value) = self.read_u8(address, bp);

        if bp_hit {
            *dbg = EmulatorMode::BreakpointHit;
            return;
        }

        self.set_register(TargetRegister::AF(true), value);

        self.pc += 1;
        self.cycles += 8;
    }

    fn load_u16_to_register(&mut self, bp: &Vec<Breakpoint>, dbg: &mut EmulatorMode, reg: TargetRegister) {
        let (bp_hit, value) = self.read_u16(self.pc + 1, bp);

        if bp_hit {
            *dbg = EmulatorMode::BreakpointHit;
            return;
        }

        match reg {
            TargetRegister::AF(_) => self.af = value,
            TargetRegister::BC(_) => self.bc = value,
            TargetRegister::DE(_) => self.de = value,
            TargetRegister::HL(_) => self.hl = value,
            TargetRegister::SP => self.sp = value
        }

        self.pc += 3;
        self.cycles += 12;
    }

    fn load_register_to_register(&mut self, target: TargetRegister, source: TargetRegister) {
        self.set_register(target, self.get_register(&source));
        
        self.pc += 1;
        self.cycles += 4;
    }

    fn store_register_to_hl(&mut self, bp: &Vec<Breakpoint>, dbg: &mut EmulatorMode, reg: TargetRegister) {
        let value = self.get_register(&reg);
        let address = self.hl;
        
        if self.write(address, value, bp) {
            *dbg = EmulatorMode::BreakpointHit;
            return;
        }
        
        self.pc += 1;
        self.cycles += 8;
    }

    fn store_to_hl_and_dec(&mut self, bp: &Vec<Breakpoint>, dbg: &mut EmulatorMode) {
        let value = self.get_register(&TargetRegister::AF(true));
        let address = self.hl;
        
        if self.write(address, value, bp) {
            *dbg = EmulatorMode::BreakpointHit;
            return;
        }
        
        self.hl = address.wrapping_sub(1);

        self.pc += 1;
        self.cycles += 8;
    }

    fn store_a_to_io_c(&mut self, bp: &Vec<Breakpoint>, dbg: &mut EmulatorMode) {
        let value = self.get_register(&TargetRegister::AF(true));
        let address = 0xFF00 + self.get_register(&TargetRegister::BC(false)) as u16;

        if self.write(address, value, bp) {
            *dbg = EmulatorMode::BreakpointHit;
            return;
        }
        
        self.pc += 1;
        self.cycles += 8;
    }

    fn store_a_to_io_u8(&mut self, bp: &Vec<Breakpoint>, dbg: &mut EmulatorMode) {
        let (bp_hit, offset) = self.read_u8(self.pc + 1, bp);

        if bp_hit {
            *dbg = EmulatorMode::BreakpointHit;
            return;
        }

        let address = 0xFF00 + offset as u16;
        let value = self.get_register(&TargetRegister::AF(true));

        if self.write(address, value, bp) {
            *dbg = EmulatorMode::BreakpointHit;
            return;
        }
        
        self.pc += 2;
        self.cycles += 12;
    }

    fn inc_register(&mut self, reg: TargetRegister) {
        let value = self.get_register(&reg);
        let result = value.wrapping_add(1);

        let zero = result == 0;
        let half_carry = (value & 0x0F) + 1 > 0x0F;

        self.set_register(reg, result);
        
        self.set_flag(TargetFlag::Zero(zero));
        self.set_flag(TargetFlag::Negative(false));
        self.set_flag(TargetFlag::HalfCarry(half_carry));

        self.pc += 1;
        self.cycles += 4;
    }

    fn xor_register(&mut self, reg: TargetRegister) {
        let value = self.get_register(&reg);
        let target = self.get_register(&TargetRegister::AF(true));

        let result = value ^ target;

        self.af = (self.af & 0x00FF) | result as u16;

        self.set_flag(TargetFlag::Zero(result == 0));
        self.set_flag(TargetFlag::Negative(false));
        self.set_flag(TargetFlag::HalfCarry(false));
        self.set_flag(TargetFlag::Carry(false));

        self.pc += 1;
        self.cycles += 4;
    }

    fn call(&mut self, bp: &Vec<Breakpoint>, dbg: &mut EmulatorMode) {
        let (bp_hit, address) = self.read_u16(self.pc + 1, bp);

        if bp_hit {
            *dbg = EmulatorMode::BreakpointHit;
            return;
        }

        if self.stack_write(self.pc + 3, bp) {
            *dbg = EmulatorMode::BreakpointHit;
            return;
        }

        self.pc = address;
        self.cycles += 24;
    }

    fn conditional_jump_relative(&mut self, bp: &Vec<Breakpoint>, dbg: &mut EmulatorMode, condition: JumpCondition) {
        let jump: bool;

        match condition {
            JumpCondition::Zero(set) => {
                let zf = self.get_flag(TargetFlag::Zero(false));

                if set {
                    jump = zf;
                }
                else {
                    jump = !zf;
                }
            }
            JumpCondition::Carry(set) => {
                let cf = self.get_flag(TargetFlag::Zero(false));

                if set {
                    jump = cf;
                }
                else {
                    jump = !cf;
                }
            }
        }

        if jump {
            let (bp_hit, offset) = self.read_u8(self.pc + 1, bp);

            if bp_hit {
                *dbg = EmulatorMode::BreakpointHit;
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

    fn bit_register(&mut self, reg: TargetRegister, bit: u8) {
        let value = self.get_register(&reg);
        let result = (value & (1 << bit)) == 0;

        self.set_flag(TargetFlag::Zero(result));
        self.set_flag(TargetFlag::Negative(false));
        self.set_flag(TargetFlag::HalfCarry(true));

        self.pc += 2;
        self.cycles += 8;
    }
}