use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::gameboy::memory::regions::*;
use crate::gameboy::memory::cart::CartHeader;
use crate::gameboy::memory::{GameboyCart, GameboyByte};

pub struct MBC1 {
    header: Arc<CartHeader>,

    rom_banks: Vec<Vec<u8>>,
    ram_banks: Vec<Vec<u8>>,

    ram_enabled: bool,

    banking_mode: u8,
    selected_rom_bank: u8,
    selected_ram_bank: u8
}

impl MBC1 {
    pub fn new(header: Arc<CartHeader>, data: Vec<u8>) -> MBC1 {
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

        MBC1 {
            header,

            rom_banks,
            ram_banks,

            ram_enabled: false,

            banking_mode: 0,
            selected_rom_bank: 1,
            selected_ram_bank: 0
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
        ((self.selected_ram_bank << 5) | self.selected_rom_bank) as usize
    }
}

impl GameboyCart for MBC1 {
    fn read(&self, address: u16) -> u8 {
        if CARTRIDGE_ROM_BANK0.contains(&address) {
            if self.banking_mode == 1 {
                let bank = (self.selected_ram_bank << 5) as usize;

                if let Some(bank) = self.rom_banks.get(bank) {
                    return bank[address as usize];
                }
            }

            return self.rom_banks[0][address as usize];
        }
        else if CARTRIDGE_ROM_BANKX.contains(&address) {
            let bank = self.get_rom_bank();
            let address = (address - 0x4000) as usize;

            if let Some(bank) = self.rom_banks.get(bank) {
                return bank[address as usize];
            }

            return self.rom_banks[1][address as usize];
        }
        else if CARTRIDGE_RAM.contains(&address) && self.is_ram_enabled() {
            let address = (address - 0xA000) as usize;

            if self.banking_mode == 0 {
                if let Some(bank) = self.ram_banks.get(0) {
                    return bank[address as usize];
                }
            }
            else {
                // MBC1 carts can have 0, 1, or 4 banks of RAM.
                // The bank register is only used if the cart is the latter.
                let bank = if self.ram_banks.len() == 4 {self.selected_ram_bank as usize} else {0};
            
                if let Some(bank) = self.ram_banks.get(bank) {
                    return bank[address as usize];
                }
            }
        }

        0xFF
    }

    fn write(&mut self, address: u16, value: u8) {
        if MBC1_ENABLE_RAM.contains(&address) {
            let enable_ram = (value & 0x0F) == 0x0A;

            if !enable_ram {
                self.save_ram();
            }

            self.ram_enabled = enable_ram;
        }
        else if MBC1_ROM_BANK.contains(&address) {
            // Mask the bank value to fit the amount of banks on the cart.
            let value = match self.rom_banks.len() {
                2 => value & 1,
                4 => value & 3,
                8 => value & 7,
                16 => value & 0x0F,
                _ => value & 0x1F
            };

            self.selected_rom_bank = if value == 0 {1} else {value};
        }
        else if MBC1_RAM_BANK.contains(&address) {
            self.selected_ram_bank = value & 3;
        }
        else if MBC1_BANKING_MODE.contains(&address) {
            self.banking_mode = value & 1;
        }
        else if CARTRIDGE_RAM.contains(&address) && self.is_ram_enabled() {
            let address = (address - 0xA000) as usize;

            if self.banking_mode == 0 {
                if let Some(bank) = self.ram_banks.get_mut(0) {
                    bank[address as usize] = value;
                }
            }
            else {
                // MBC1 carts can have no 0, 1, or 4 banks of RAM.
                // The bank register is only used if the cart is the latter.
                let bank = if self.ram_banks.len() == 4 {self.selected_ram_bank as usize} else {0};
                
                if let Some(bank) = self.ram_banks.get_mut(bank) {
                    bank[address as usize] = value;
                }
            }
        }
    }

    // TODO: Get this to work properly with banking.
    fn dbg_write(&mut self, address: u16, value: u8) {
        if CARTRIDGE_ROM_BANK0.contains(&address) {
            self.rom_banks[0][address as usize] = value
        }
        else if CARTRIDGE_ROM_BANKX.contains(&address) {
            self.rom_banks[1][address as usize - 0x4000] = value
        }
    }

    fn reset(&mut self) {
        self.banking_mode = 0;
        self.selected_rom_bank = 1;
        self.selected_ram_bank = 0;
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
        self.selected_ram_bank as usize
    }
}