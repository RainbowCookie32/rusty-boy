use std::sync::Arc;

use crate::gameboy::memory::GameboyMemory;

const VBLANK_BIT: u8 = 0x01;
const STAT_BIT: u8 = 0x02;
const TIMER_BIT: u8 = 0x04;
const SERIAL_BIT: u8 = 0x08;
const JOYPAD_BIT: u8 = 0x10;

pub struct InterruptHandler {
    ime: bool,

    ei_executed: bool,
    instructions_since_ei: u8,

    gb_mem: Arc<GameboyMemory>
}

impl InterruptHandler {
    pub fn init(gb_mem: Arc<GameboyMemory>) -> InterruptHandler {
        InterruptHandler {
            ime: false,

            ei_executed: false,
            instructions_since_ei: 0,

            gb_mem
        }
    }

    pub fn check_interrupts(&mut self) -> Option<u16> {
        if self.ei_executed {
            if self.instructions_since_ei > 0 {
                self.ime = true;
                self.ei_executed = false;
                self.instructions_since_ei = 0;
            }
            else {
                self.instructions_since_ei += 1;
            }
        }

        if self.ime {
            let if_value = self.gb_mem.read(0xFF0F);
            let ie_value = self.gb_mem.read(0xFFFF);

            if (if_value & VBLANK_BIT != 0) && (ie_value & VBLANK_BIT != 0) {
                let new_if = if_value & !VBLANK_BIT;

                self.ime = false;
                self.gb_mem.write(0xFF0F, new_if);

                Some(0x40)
            }
            else if (if_value & STAT_BIT != 0) && (ie_value & STAT_BIT != 0) {
                let new_if = if_value & !STAT_BIT;

                self.ime = false;
                self.gb_mem.write(0xFF0F, new_if);

                Some(0x48)
            }
            else if (if_value & TIMER_BIT != 0) && (ie_value & TIMER_BIT != 0) {
                let new_if = if_value & !TIMER_BIT;

                self.ime = false;
                self.gb_mem.write(0xFF0F, new_if);

                Some(0x50)
            }
            else if (if_value & SERIAL_BIT != 0) && (ie_value & SERIAL_BIT != 0) {
                let new_if = if_value & !SERIAL_BIT;

                self.ime = false;
                self.gb_mem.write(0xFF0F, new_if);

                Some(0x58)
            }
            else if (if_value & JOYPAD_BIT != 0) && (ie_value & JOYPAD_BIT != 0) {
                let new_if = if_value & !JOYPAD_BIT;

                self.ime = false;
                self.gb_mem.write(0xFF0F, new_if);

                Some(0x60)
            }
            else {
                None
            }
        }
        else {
            None
        }
    }

    pub fn enable_interrupts(&mut self, ei: bool) {
        if ei {
            self.ei_executed = true;
            self.instructions_since_ei = 0;
        }
        else {
            self.ime = true;
        }
    }

    pub fn disable_interrupts(&mut self) {
        self.ime = false;
        self.ei_executed = false;
        self.instructions_since_ei = 0;
    }
}
