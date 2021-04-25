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
    Zero,
    Negative,
    HalfCarry,
    Carry
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
            TargetFlag::Zero => (self.af & 0x80) != 0,
            TargetFlag::Negative => (self.af & 0x40) != 0,
            TargetFlag::HalfCarry => (self.af & 0x20) != 0,
            TargetFlag::Carry => (self.af & 0x10) != 0,
        }
    }

    fn set_flag(&mut self, flag: TargetFlag, value: bool) {
        let mut flags = self.af & 0x00FF;

        match flag {
            TargetFlag::Zero => {
                if value {
                    flags |= 1 << 7;
                }
                else {
                    flags &= !(1 << 7);
                }
            }
            TargetFlag::Negative => {
                if value {
                    flags |= 1 << 6;
                }
                else {
                    flags &= !(1 << 6);
                }
            }
            TargetFlag::HalfCarry => {
                if value {
                    flags |= 1 << 5;
                }
                else {
                    flags &= !(1 << 5);
                }
            }
            TargetFlag::Carry => {
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
            0x0C => self.inc_register(TargetRegister::BC(false)),
            0x0E => self.load_u8_to_register(breakpoints, dbg_mode, TargetRegister::BC(false)),

            0x11 => self.load_u16_to_register(breakpoints, dbg_mode, TargetRegister::DE(false)),
            0x14 => self.inc_register(TargetRegister::DE(true)),
            0x16 => self.load_u8_to_register(breakpoints, dbg_mode, TargetRegister::DE(true)),
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

            0x70 => self.store_register_to_hl(breakpoints, dbg_mode, TargetRegister::BC(true)),
            0x71 => self.store_register_to_hl(breakpoints, dbg_mode, TargetRegister::BC(false)),
            0x72 => self.store_register_to_hl(breakpoints, dbg_mode, TargetRegister::DE(true)),
            0x73 => self.store_register_to_hl(breakpoints, dbg_mode, TargetRegister::DE(false)),
            0x74 => self.store_register_to_hl(breakpoints, dbg_mode, TargetRegister::HL(true)),
            0x75 => self.store_register_to_hl(breakpoints, dbg_mode, TargetRegister::HL(false)),
            0x77 => self.store_register_to_hl(breakpoints, dbg_mode, TargetRegister::AF(true)),

            0xA8 => self.xor_register(TargetRegister::BC(true)),
            0xA9 => self.xor_register(TargetRegister::BC(false)),
            0xAA => self.xor_register(TargetRegister::DE(true)),
            0xAB => self.xor_register(TargetRegister::DE(false)),
            0xAC => self.xor_register(TargetRegister::HL(true)),
            0xAD => self.xor_register(TargetRegister::HL(false)),
            0xAF => self.xor_register(TargetRegister::AF(true)),

            0xCB => self.execute_instruction_prefixed(breakpoints, dbg_mode),

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
            0x7C => self.bit_register(TargetRegister::HL(true), 7),

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

    fn store_register_to_hl(&mut self, bp: &Vec<Breakpoint>, dbg: &mut EmulatorMode, reg: TargetRegister) {
        let value = self.get_register(&reg);
        let address = self.hl;
        
        if self.write(address, value, bp) {
            *dbg = EmulatorMode::BreakpointHit;
            return;
        }
        else {
            self.pc += 1;
            self.cycles += 8;
        }
    }

    fn store_to_hl_and_dec(&mut self, bp: &Vec<Breakpoint>, dbg: &mut EmulatorMode) {
        let value = self.get_register(&TargetRegister::AF(true));
        let address = self.hl;
        
        if self.write(address, value, bp) {
            *dbg = EmulatorMode::BreakpointHit;
            return;
        }
        else {
            self.hl = address.wrapping_sub(1);

            self.pc += 1;
            self.cycles += 8;
        }
    }

    fn store_a_to_io_c(&mut self, bp: &Vec<Breakpoint>, dbg: &mut EmulatorMode) {
        let value = self.get_register(&TargetRegister::AF(true));
        let address = 0xFF00 + self.get_register(&TargetRegister::BC(false)) as u16;

        if self.write(address, value, bp) {
            *dbg = EmulatorMode::BreakpointHit;
            return;
        }
        else {
            self.pc += 1;
            self.cycles += 8;
        }
    }

    fn inc_register(&mut self, reg: TargetRegister) {
        let value = self.get_register(&reg);
        let result = value.wrapping_add(1);

        let zero = result == 0;
        let half_carry = (value & 0x0F) + 1 > 0x0F;

        self.set_register(reg, result);
        
        self.set_flag(TargetFlag::Zero, zero);
        self.set_flag(TargetFlag::Negative, false);
        self.set_flag(TargetFlag::HalfCarry, half_carry);

        self.pc += 1;
        self.cycles += 4;
    }

    fn xor_register(&mut self, reg: TargetRegister) {
        let value = self.get_register(&reg);
        let target = self.get_register(&TargetRegister::AF(true));

        let result = value ^ target;

        self.af = (self.af & 0x00FF) | result as u16;

        self.set_flag(TargetFlag::Zero, result == 0);
        self.set_flag(TargetFlag::Negative, false);
        self.set_flag(TargetFlag::HalfCarry, false);
        self.set_flag(TargetFlag::Carry, false);

        self.pc += 1;
        self.cycles += 4;
    }

    fn conditional_jump_relative(&mut self, bp: &Vec<Breakpoint>, dbg: &mut EmulatorMode, condition: JumpCondition) {
        let jump: bool;

        match condition {
            JumpCondition::Zero(set) => {
                let zf = self.get_flag(TargetFlag::Zero);

                if set {
                    jump = zf;
                }
                else {
                    jump = !zf;
                }
            }
            JumpCondition::Carry(set) => {
                let cf = self.get_flag(TargetFlag::Zero);

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

        self.set_flag(TargetFlag::Zero, result);
        self.set_flag(TargetFlag::Negative, false);
        self.set_flag(TargetFlag::HalfCarry, true);

        self.pc += 2;
        self.cycles += 8;
    }
}