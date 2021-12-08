use std::sync::Arc;

use crate::gameboy::memory::regions::*;
use crate::gameboy::memory::GameboyCart;
use crate::gameboy::memory::cart::CartHeader;

pub struct MBC5 {
    header: Arc<CartHeader>,

    rom_banks: Vec<Vec<u8>>,
    ram_banks: Vec<Vec<u8>>,

    romb0: u8,
    romb1: u8,
    
    ramb: u8,
    ram_enabled: bool
}

impl MBC5 {
    pub fn new(header: Arc<CartHeader>, data: Vec<u8>) -> MBC5 {
        let rom_banks = {
            let mut result = Vec::new();
            let chunks = data.chunks(16384);

            for chunk in chunks {
                result.push(chunk.to_vec());
            }

            result
        };

        let ram_banks = {
            if let Ok(data) = std::fs::read(format!("ram/{}.bin", header.title())) {
                let mut result = Vec::with_capacity(8192 * header.ram_banks_count());

                for chunk in data.chunks_exact(8192) {
                    result.push(chunk.to_vec());
                }

                result
            }
            else {
                vec![vec![0; 8192]; header.ram_banks_count]
            }
        };

        MBC5 {
            header,

            rom_banks,
            ram_banks,

            romb0: 0,
            romb1: 0,

            ramb: 0,
            ram_enabled: false
        }
    }

    fn save_ram(&self) {
        let mut data = Vec::with_capacity(8192 * self.ram_banks.len());

        for bank in self.ram_banks.iter() {
            for byte in bank {
                data.push(*byte);
            }
        }

        if let Err(error) = std::fs::create_dir("ram") {
            if error.kind() != std::io::ErrorKind::AlreadyExists {
                println!("Error creating RAM directory: {}", error.to_string());
            }
        }

        if let Err(error) = std::fs::write(format!("ram/{}.bin", self.header.title()), data) {
            println!("Error saving ram contents: {}", error.to_string());
        }
    }

    fn get_rom_bank(&self) -> usize {
        (((self.romb1 as u16) << 9) | self.romb0 as u16) as usize
    }
}

impl GameboyCart for MBC5 {
    fn read(&self, address: u16) -> u8 {
        if CARTRIDGE_ROM_BANK0.contains(&address) {
            self.rom_banks[0][address as usize]
        }
        else if CARTRIDGE_ROM_BANKX.contains(&address) {
            let address = (address - 0x4000) as usize;
            self.rom_banks[self.get_selected_rom_bank()][address]
        }
        else if CARTRIDGE_RAM.contains(&address) {
            let address = (address - 0xA000) as usize;
            self.ram_banks[self.get_selected_ram_bank()][address]
        }
        else {
            unreachable!()
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        if MBC5_RAMG.contains(&address) {
            self.ram_enabled = value == 0b00001010;

            if !self.ram_enabled {
                self.save_ram();
            }
        }
        else if MBC5_ROMB0.contains(&address) {
            self.romb0 = value;
        }
        else if MBC5_ROMB1.contains(&address) {
            self.romb1 = value & 1;
        }
        else if MBC5_RAMB.contains(&address) {
            self.ramb = value & 0b00001111;
        }
    }

    // TODO: Get this to work properly with banking.
    fn dbg_write(&mut self, address: u16, value: u8) {
        
    }

    fn reset(&mut self) {
        self.romb0 = 0;
        self.romb1 = 0;
        self.ram_enabled = false;
    }

    fn get_header(&self) -> Arc<CartHeader> {
        self.header.clone()
    }

    fn is_ram_enabled(&self) -> bool {
        self.ram_enabled
    }

    fn get_selected_rom_bank(&self) -> usize {
        self.get_rom_bank()
    }

    fn get_selected_ram_bank(&self) -> usize {
        self.ramb as usize
    }
}
