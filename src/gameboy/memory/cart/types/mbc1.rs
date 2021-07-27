use std::sync::atomic::{AtomicBool, Ordering};

use crate::gameboy::memory::regions::*;
use crate::gameboy::memory::cart::CartHeader;
use crate::gameboy::memory::{GameboyCart, GameboyByte};

pub struct MBC1 {
    header: CartHeader,

    rom_banks: Vec<Vec<GameboyByte>>,
    ram_banks: Vec<Vec<GameboyByte>>,

    ram_enabled: AtomicBool,

    banking_mode: GameboyByte,
    selected_rom_bank: GameboyByte,
    selected_ram_bank: GameboyByte
}

impl MBC1 {
    pub fn new(header: CartHeader, data: Vec<u8>) -> MBC1 {
        let rom_banks = {
            let mut result = Vec::new();
            let chunks = data.chunks(16384);

            for chunk in chunks {
                let chunk: Vec<GameboyByte> = chunk.iter().map(|b| GameboyByte::from(*b)).collect();
                result.push(chunk.to_vec());
            }

            result
        };

        let ram_banks = vec![vec![GameboyByte::from(0); 8192]; header.ram_banks_count];

        MBC1 {
            header,

            rom_banks,
            ram_banks,

            ram_enabled: AtomicBool::new(false),

            banking_mode: GameboyByte::from(0),
            selected_rom_bank: GameboyByte::from(1),
            selected_ram_bank: GameboyByte::from(0)
        }
    }

    fn get_rom_bank(&self) -> usize {
        if self.rom_banks.len() > 32 {
            ((self.selected_ram_bank.get() << 5) | self.selected_rom_bank.get()) as usize
        }
        else {
            self.selected_rom_bank.get() as usize
        }
    }
}

impl GameboyCart for MBC1 {
    fn read(&self, address: u16) -> u8 {
        if CARTRIDGE_ROM_BANK0.contains(&address) {
            if self.banking_mode.get() == 1 {
                let bank = (self.selected_ram_bank.get() << 5) as usize;

                if let Some(bank) = self.rom_banks.get(bank) {
                    return bank[address as usize].get();
                }
            }
            else {
                return self.rom_banks[0][address as usize].get();
            }
        }
        else if CARTRIDGE_ROM_BANKX.contains(&address) {
            let bank = self.get_rom_bank();
            let address = (address - 0x4000) as usize;

            if let Some(bank) = self.rom_banks.get(bank) {
                return bank[address as usize].get();
            }
        }
        else if CARTRIDGE_RAM.contains(&address) && self.is_ram_enabled() {
            let address = (address - 0xA000) as usize;

            if self.banking_mode.get() == 0 {
                if let Some(bank) = self.ram_banks.get(0) {
                    return bank[address as usize].get();
                }
            }
            else {
                // MBC1 carts can have no 0, 1, or 4 banks of RAM.
                // The bank register is only used if the cart is the latter.
                let bank = if self.ram_banks.len() == 4 {self.selected_ram_bank.get() as usize} else {0};
            
                if let Some(bank) = self.ram_banks.get(bank) {
                    return bank[address as usize].get();
                }
            }
        }

        0xFF
    }

    fn write(&self, address: u16, value: u8) {
        if MBC1_ENABLE_RAM.contains(&address) {
            self.ram_enabled.store((value & 0x0F) == 0x0A, Ordering::Relaxed);
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

            self.selected_rom_bank.set(if value == 0 {1} else {value});
        }
        else if MBC1_RAM_BANK.contains(&address) {
            self.selected_ram_bank.set(value & 3);
        }
        else if MBC1_BANKING_MODE.contains(&address) {
            self.banking_mode.set(value & 1);
        }
        else if CARTRIDGE_RAM.contains(&address) && self.is_ram_enabled() {
            let address = (address - 0xA000) as usize;

            if self.banking_mode.get() == 0 {
                if let Some(bank) = self.ram_banks.get(0) {
                    bank[address as usize].set(value);
                }
            }
            else {
                // MBC1 carts can have no 0, 1, or 4 banks of RAM.
                // The bank register is only used if the cart is the latter.
                let bank = if self.ram_banks.len() == 4 {self.selected_ram_bank.get() as usize} else {0};
                
                if let Some(bank) = self.ram_banks.get(bank) {
                    bank[address as usize].set(value);
                }
            }
        }
    }

    // TODO: Get this to work properly with banking.
    fn dbg_write(&self, address: u16, value: u8) {
        if CARTRIDGE_ROM_BANK0.contains(&address) {
            self.rom_banks[0][address as usize].set(value)
        }
        else if CARTRIDGE_ROM_BANKX.contains(&address) {
            self.rom_banks[1][address as usize - 0x4000].set(value)
        }
    }

    fn get_header(&self) -> &CartHeader {
        &self.header
    }

    fn is_ram_enabled(&self) -> bool {
        self.ram_enabled.load(Ordering::Relaxed)
    }

    fn get_selected_rom_bank(&self) -> usize {
        self.get_rom_bank()
    }

    fn get_selected_ram_bank(&self) -> usize {
        self.selected_ram_bank.get() as usize
    }
}