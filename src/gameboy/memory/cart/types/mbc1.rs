use std::sync::atomic::{AtomicBool, Ordering};

use crate::gameboy::memory::cart::CartHeader;
use crate::gameboy::memory::{GameboyCart, GameboyByte};

pub struct MBC1 {
    header: CartHeader,

    rom_banks: Vec<Vec<GameboyByte>>,
    ram_banks: Vec<Vec<GameboyByte>>,

    ram_enabled: AtomicBool,
    simple_banking_mode: AtomicBool,

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
            simple_banking_mode: AtomicBool::new(true),

            selected_rom_bank: GameboyByte::from(1),
            selected_ram_bank: GameboyByte::from(0)
        }
    }

    fn get_rom_bank(&self) -> usize {
        if self.rom_banks.len() > 32 {
            ((self.selected_ram_bank.get() << 5) + self.selected_rom_bank.get()) as usize
        }
        else {
            self.selected_rom_bank.get() as usize
        }
    }
}

impl GameboyCart for MBC1 {
    fn read(&self, address: u16) -> u8 {
        if address <= 0x3FFF {
            if !self.simple_banking_mode.load(Ordering::Relaxed) && self.rom_banks.len() > 32 {
                let bank = (self.selected_ram_bank.get() << 5) as usize;

                self.rom_banks[bank][address as usize].get()
            }
            else {
                self.rom_banks[0][address as usize].get()
            }
        }
        else if (0x4000..=0x7FFF).contains(&address) {
            let bank = self.get_rom_bank();
            let address = (address - 0x4000) as usize;

            self.rom_banks[bank][address].get()
        }
        else if (0xA000..=0xBFFF).contains(&address) {
            let bank = self.selected_ram_bank.get() as usize;
            let address = (address - 0xA000) as usize;
            
            self.ram_banks[bank][address].get()
        }
        else {
            0
        }
    }

    fn write(&self, address: u16, value: u8) {
        if address <= 0x1FFF {
            self.ram_enabled.store((value & 0x0A) == 0x0A, Ordering::Relaxed);
        }
        else if (0x2000..=0x3FFF).contains(&address) {
            self.selected_rom_bank.set(value & 0x1F);
        }
        else if (0x4000..=0x5FFF).contains(&address) {
            self.selected_ram_bank.set(value & 3);
        }
        else if (0x6000..=0x7FFF).contains(&address) {
            self.simple_banking_mode.store(value == 0, Ordering::Relaxed);
        }
        else if (0xA000..=0xBFFF).contains(&address) {
            let bank = self.selected_ram_bank.get() as usize;
            let address = (address - 0xA000) as usize;
            
            self.ram_banks[bank][address].set(value)
        }
    }

    // TODO: Get this to work properly with banking.
    fn dbg_write(&self, address: u16, value: u8) {
        if address <= 0x3FFF {
            self.rom_banks[0][address as usize].set(value)
        }
        else if (0x4000..=0x7FFF).contains(&address) {
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