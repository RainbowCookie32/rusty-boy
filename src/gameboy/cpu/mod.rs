use super::*;

pub struct GameboyCPU {
    af: u16,
    bc: u16,
    de: u16,
    hl: u16,

    sp: u16,
    pc: u16,

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

            memory
        }
    }

    pub fn get_all_registers(&self) -> (&u16, &u16, &u16, &u16, &u16, &u16) {
        (&self.af, &self.bc, &self.de, &self.hl, &self.sp, &self.pc)
    }

    fn read(&self, address: u16, breakpoints: &Vec<Breakpoint>) -> (bool, u8) {
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

    pub fn cpu_cycle(&mut self, breakpoints: &Vec<Breakpoint>, dbg_mode: &mut EmulatorMode) {
        for bp in breakpoints {
            if self.pc == *bp.address() && *bp.execute() {
                if *dbg_mode != EmulatorMode::Stepping {
                    *dbg_mode = EmulatorMode::BreakpointHit;
                    return;
                }
            }
        }

        let (bp_hit, opcode) = self.read(self.pc, breakpoints);

        if bp_hit && *dbg_mode != EmulatorMode::Stepping {
            *dbg_mode = EmulatorMode::BreakpointHit;
            return;
        }

        match opcode {
            _ => {}
        }
    }
}