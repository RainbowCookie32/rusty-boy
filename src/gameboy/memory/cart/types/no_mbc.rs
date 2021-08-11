use std::sync::Arc;

use crate::gameboy::memory::regions::*;
use crate::gameboy::memory::GameboyCart;
use crate::gameboy::memory::cart::CartHeader;

pub struct NoMBC {
    header: Arc<CartHeader>,
    rom_banks: Vec<Vec<u8>>
}

impl NoMBC {
    pub fn new(header: Arc<CartHeader>, data: Vec<u8>) -> NoMBC {
        let rom_banks = {
            let mut result = Vec::new();
            let chunks = data.chunks(16384);

            for chunk in chunks {
                result.push(chunk.to_vec());
            }

            result
        };

        NoMBC {
            header,
            rom_banks
        }
    }
}

impl GameboyCart for NoMBC {
    fn read(&self, address: u16) -> u8 {
        if CARTRIDGE_ROM_BANK0.contains(&address) {
            self.rom_banks[0][address as usize]
        }
        else if CARTRIDGE_ROM_BANKX.contains(&address) {
            self.rom_banks[1][address as usize - 0x4000]
        }
        else {
            0
        }
    }

    fn write(&mut self, _address: u16, _value: u8) {
        
    }

    fn dbg_write(&mut self, address: u16, value: u8) {
        if CARTRIDGE_ROM_BANK0.contains(&address) {
            self.rom_banks[0][address as usize] = value;
        }
        else if CARTRIDGE_ROM_BANKX.contains(&address) {
            self.rom_banks[1][address as usize - 0x4000] = value;
        }
    }

    fn reset(&mut self) {
        
    }

    fn get_header(&self) -> Arc<CartHeader> {
        self.header.clone()
    }

    fn is_ram_enabled(&self) -> bool {
        false
    }

    fn get_selected_rom_bank(&self) -> usize {
        1
    }

    fn get_selected_ram_bank(&self) -> usize {
        0
    }
}