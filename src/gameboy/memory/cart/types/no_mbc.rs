use super::GameboyCart;
use crate::gameboy::memory::GameboyByte;

pub struct NoMBC {
    rom_banks: Vec<Vec<GameboyByte>>
}

impl NoMBC {
    pub fn new(data: Vec<u8>) -> NoMBC {
        let rom_banks = {
            let mut result = Vec::new();
            let mut chunks = data.chunks(16384);

            while let Some(chunk) = chunks.next() {
                let chunk: Vec<GameboyByte> = chunk.into_iter().map(|b| GameboyByte::from(*b)).collect();
                result.push(chunk.to_vec());
            }

            result
        };

        NoMBC {
            rom_banks
        }
    }
}

impl GameboyCart for NoMBC {
    fn read(&self, address: u16) -> u8 {
        if address <= 0x3FFF {
            self.rom_banks[0][address as usize].get()
        }
        else if address >= 0x4000 && address <= 0x7FFF {
            self.rom_banks[1][address as usize - 0x4000].get()
        }
        else {
            0
        }
    }

    fn write(&self, _address: u16, _value: u8) {
        
    }

    fn dbg_write(&self, address: u16, value: u8) {
        if address <= 0x3FFF {
            self.rom_banks[0][address as usize].set(value)
        }
        else if address >= 0x4000 && address <= 0x7FFF {
            self.rom_banks[1][address as usize - 0x4000].set(value)
        }
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