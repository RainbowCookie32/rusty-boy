use std::sync::{Arc, RwLock};

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

    gb_mem: Arc<RwLock<GameboyMemory>>
}

impl InterruptHandler {
    pub fn init(gb_mem: Arc<RwLock<GameboyMemory>>) -> InterruptHandler {
        InterruptHandler {
            ime: false,

            ei_executed: false,
            instructions_since_ei: 0,

            gb_mem
        }
    }

    fn read(&self, address: u16) -> u8 {
        if let Ok(lock) = self.gb_mem.read() {
            lock.read(address)
        }
        else {
            0
        }
    }

    fn write(&self, address: u16, value: u8) {
        if let Ok(mut lock) = self.gb_mem.write() {
            lock.write(address, value)
        }
    }

    // Returns whether an int was requested or not, and an address
    // to jump to if the interrupt was enabled.
    pub fn check_interrupts(&mut self) -> (bool, Option<u16>) {
        let mut requested = false;

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
            let if_value = self.read(0xFF0F);
            let ie_value = self.read(0xFFFF);

            if if_value & VBLANK_BIT != 0 {
                requested = true;

                if ie_value & VBLANK_BIT != 0 {
                    let new_if = if_value & !VBLANK_BIT;

                    self.ime = false;
                    self.write(0xFF0F, new_if);

                    return (requested, Some(0x40));
                }
            }
            else if if_value & STAT_BIT != 0 {
                requested = true;

                if ie_value & STAT_BIT != 0 {
                    let new_if = if_value & !STAT_BIT;

                    self.ime = false;
                    self.write(0xFF0F, new_if);
    
                    return (requested, Some(0x48))
                }
            }
            else if if_value & TIMER_BIT != 0 {
                requested = true;

                if ie_value & TIMER_BIT != 0 {
                    let new_if = if_value & !TIMER_BIT;

                    self.ime = false;
                    self.write(0xFF0F, new_if);
    
                    return (requested, Some(0x50))
                }
            }
            else if if_value & SERIAL_BIT != 0 {
                requested = true;

                if ie_value & SERIAL_BIT != 0 {
                    let new_if = if_value & !SERIAL_BIT;

                    self.ime = false;
                    self.write(0xFF0F, new_if);
    
                    return (requested, Some(0x58));
                }
            }
            else if if_value & JOYPAD_BIT != 0 {
                requested = true;
                
                if ie_value & JOYPAD_BIT != 0 {
                    let new_if = if_value & !JOYPAD_BIT;

                    self.ime = false;
                    self.write(0xFF0F, new_if);
    
                    return (requested, Some(0x60));
                }
            }
        }
        
        (requested, None)
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
