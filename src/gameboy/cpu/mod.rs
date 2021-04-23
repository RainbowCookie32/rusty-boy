use super::*;

enum TargetRegister {
    AF,
    BC,
    DE,
    HL,
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
        let flags = self.af & 0x00FF;

        match flag {
            TargetFlag::Zero => (flags >> 7) & 1 != 0,
            TargetFlag::Negative => (flags >> 6) & 1 != 0,
            TargetFlag::HalfCarry => (flags >> 5) & 1 != 0,
            TargetFlag::Carry => (flags >> 4) & 1 != 0,
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

    fn get_register(&self, reg: TargetRegister, high: bool) -> u8 {
        match reg {
            TargetRegister::AF => {
                if high {
                    ((self.af & 0xFF00) >> 8) as u8
                }
                else {
                    (self.af & 0x00FF) as u8
                }
            }
            TargetRegister::BC => {
                if high {
                    ((self.bc & 0xFF00) >> 8) as u8
                }
                else {
                    (self.bc & 0x00FF) as u8
                }
            }
            TargetRegister::DE => {
                if high {
                    ((self.de & 0xFF00) >> 8) as u8
                }
                else {
                    (self.de & 0x00FF) as u8
                }
            }
            TargetRegister::HL => {
                if high {
                    ((self.hl & 0xFF00) >> 8) as u8
                }
                else {
                    (self.hl & 0x00FF) as u8
                }
            }
            TargetRegister::SP => {
                if high {
                    ((self.sp & 0xFF00) >> 8) as u8
                }
                else {
                    (self.sp & 0x00FF) as u8
                }
            }
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
            0x01 => self.load_u16_to_register(breakpoints, dbg_mode, TargetRegister::BC),

            0x11 => self.load_u16_to_register(breakpoints, dbg_mode, TargetRegister::DE),

            0x21 => self.load_u16_to_register(breakpoints, dbg_mode, TargetRegister::HL),

            0x31 => self.load_u16_to_register(breakpoints, dbg_mode, TargetRegister::SP),
            0x32 => self.store_to_hl_and_dec(breakpoints, dbg_mode),

            0xAF => self.xor_register(TargetRegister::AF, true),

            0xCB => self.execute_instruction_prefixed(breakpoints, dbg_mode),

            _ => *dbg_mode = EmulatorMode::UnknownInstruction(true, opcode)
        }
    }

    fn execute_instruction_prefixed(&mut self, breakpoints: &Vec<Breakpoint>, dbg_mode: &mut EmulatorMode) {
        let (bp_hit, opcode) = self.read_u8(self.pc + 1, breakpoints);

        if bp_hit && *dbg_mode != EmulatorMode::Stepping {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        match opcode {
            0x7C => self.bit_register(TargetRegister::HL, 7, true),

            _ => *dbg_mode = EmulatorMode::UnknownInstruction(false, opcode)
        }
    }

    fn load_u16_to_register(&mut self, bp: &Vec<Breakpoint>, dbg: &mut EmulatorMode, reg: TargetRegister) {
        let (bp_hit, value) = self.read_u16(self.pc + 1, bp);

        if bp_hit {
            *dbg = EmulatorMode::BreakpointHit;
            return;
        }

        match reg {
            TargetRegister::AF => self.af = value,
            TargetRegister::BC => self.bc = value,
            TargetRegister::DE => self.de = value,
            TargetRegister::HL => self.hl = value,
            TargetRegister::SP => self.sp = value
        }

        self.pc += 3;
        self.cycles += 12;
    }

    fn store_to_hl_and_dec(&mut self, bp: &Vec<Breakpoint>, dbg: &mut EmulatorMode) {
        let value = self.get_register(TargetRegister::AF, true);
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

    fn xor_register(&mut self, reg: TargetRegister, high: bool) {
        let value = self.get_register(reg, high);
        let target = self.get_register(TargetRegister::AF, true);

        let result = value ^ target;

        self.af = (self.af & 0x00FF) | result as u16;

        self.set_flag(TargetFlag::Zero, result == 0);
        self.set_flag(TargetFlag::Negative, false);
        self.set_flag(TargetFlag::HalfCarry, false);
        self.set_flag(TargetFlag::Carry, false);

        self.pc += 1;
        self.cycles += 4;
    }

    fn bit_register(&mut self, reg: TargetRegister, bit: u8, high: bool) {
        let value = self.get_register(reg, high);
        let result = (value & (1 << bit)) == 0;

        self.set_flag(TargetFlag::Zero, result);
        self.set_flag(TargetFlag::Negative, false);
        self.set_flag(TargetFlag::HalfCarry, true);

        self.pc += 2;
        self.cycles += 8;
    }
}