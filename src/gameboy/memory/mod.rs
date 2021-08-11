pub mod dma;
pub mod cart;
pub mod regions;

use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicU8, Ordering};

use regions::*;
use cart::{CartHeader, GameboyCart};

use crate::gameboy::JoypadHandler;

pub struct GameboyByte {
    value: AtomicU8
}

impl GameboyByte {
    pub fn from(value: u8) -> GameboyByte {
        GameboyByte {
            value: AtomicU8::from(value)
        }
    }

    pub fn get(&self) -> u8 {
        self.value.load(Ordering::Relaxed)
    }

    pub fn set(&self, value: u8) {
        self.value.store(value, Ordering::Relaxed)
    }
}

impl Clone for GameboyByte {
    fn clone(&self) -> GameboyByte {
        GameboyByte {
            value: AtomicU8::from(self.value.load(Ordering::Relaxed))
        }
    }
}

pub struct GameboyMemory {
    bootrom: Vec<u8>,
    cartridge: Box<dyn GameboyCart + Send + Sync>,

    vram: Vec<u8>,
    wram: Vec<u8>,

    oam: Vec<u8>,
    io: Vec<u8>,
    hram: Vec<u8>,

    ie: u8,

    gb_joy: Arc<RwLock<JoypadHandler>>,
    serial_output: Arc<RwLock<Vec<u8>>>
}

impl GameboyMemory {
    pub fn init(bootrom: Vec<u8>, romfile_data: Vec<u8>, gb_joy: Arc<RwLock<JoypadHandler>>) -> GameboyMemory {
        let cartridge = cart::create_cart(romfile_data);

        GameboyMemory {
            bootrom,
            cartridge,
            
            vram: vec![0; 0x2000],
            wram: vec![0; 0x2000],

            oam: vec![0; 0x00A0],
            io: vec![0; 0x0080],
            hram: vec![0; 0x007F],

            ie: 0,

            gb_joy,
            serial_output: Arc::new(RwLock::new(Vec::new()))
        }
    }

    #[allow(clippy::borrowed_box)]
    pub fn cartridge(&self) -> &Box<dyn GameboyCart + Send + Sync> {
        &self.cartridge
    }

    pub fn header(&self) -> Arc<CartHeader> {
        self.cartridge.get_header()
    }

    pub fn gb_joy(&self) -> Arc<RwLock<JoypadHandler>> {
        self.gb_joy.clone()
    }

    pub fn serial_output(&self) -> Arc<RwLock<Vec<u8>>> {
        self.serial_output.clone()
    }

    pub fn reset(&mut self) {
        self.cartridge.reset();

        for b in self.vram.iter_mut() {
            *b = 0;
        }

        for b in self.wram.iter_mut() {
            *b = 0;
        }

        for b in self.oam.iter_mut() {
            *b = 0;
        }

        for b in self.io.iter_mut() {
            *b = 0;
        }

        for b in self.hram.iter_mut() {
            *b = 0;
        }

        self.ie = 0;

        if let Ok(mut lock) = self.serial_output.write() {
            lock.clear();
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        if CARTRIDGE_ROM.contains(&address) {
            let bootrom_enabled = self.read(0xFF50) == 0;

            if bootrom_enabled {
                if address >= self.bootrom.len() as u16 {
                    self.cartridge.read(address)
                }
                else {
                    self.bootrom[address as usize]
                }
            }
            else {
                self.cartridge.read(address)
            }
        }
        else if VRAM.contains(&address) {
            self.vram[address as usize - 0x8000]
        }
        else if CARTRIDGE_RAM.contains(&address) {
            self.cartridge.read(address)
        }
        else if WRAM.contains(&address) {
            self.wram[address as usize - 0xC000]
        }
        else if ECHO.contains(&address) {
            self.wram[address as usize - 0xE000]
        }
        else if OAM.contains(&address) {
            self.oam[address as usize - 0xFE00]
        }
        // Unused.
        else if (0xFEA0..=0xFEFF).contains(&address) {
            0
        }
        else if IO.contains(&address) {
            if address == 0xFF00 {
                if let Ok(lock) = self.gb_joy.read() {
                    return lock.get_buttons();
                }
            }

            self.io[address as usize - 0xFF00]
        }
        else if HRAM.contains(&address) {
            self.hram[address as usize - 0xFF80]
        }
        else {
            self.ie
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        if CARTRIDGE_ROM.contains(&address) {
            self.cartridge.write(address, value);
        }
        else if VRAM.contains(&address) {
            self.vram[address as usize - 0x8000] = value;
        }
        else if CARTRIDGE_RAM.contains(&address) {
            self.cartridge.write(address, value);
        }
        else if WRAM.contains(&address) {
            self.wram[address as usize - 0xC000] = value;
        }
        else if ECHO.contains(&address) {
            self.wram[address as usize - 0xE000] = value;
        }
        else if OAM.contains(&address) {
            self.oam[address as usize - 0xFE00] = value;
        }
        // Unused.
        else if (0xFEA0..=0xFEFF).contains(&address) {
            
        }
        else if IO.contains(&address) {
            if address == 0xFF00 {
                if let Ok(mut lock) = self.gb_joy.write() {
                    lock.set_value(value);
                    return;
                }
            }
            else if address == 0xFF01 {
                if let Ok(mut lock) = self.serial_output.write() {
                    lock.push(value);
                }
            }

            self.io[address as usize - 0xFF00] = value;
        }
        else if HRAM.contains(&address) {
            self.hram[address as usize - 0xFF80] = value;
        }
        else {
            self.ie = value;
        }
    }

    pub fn dbg_write(&mut self, address: u16, value: u8) {
        if CARTRIDGE_ROM.contains(&address) {
            let bootrom_enabled = self.read(0xFF50) == 0;

            if bootrom_enabled {
                if address >= self.bootrom.len() as u16 {
                    self.cartridge.dbg_write(address, value);
                }
                else {
                    self.bootrom[address as usize] = value;
                }
            }
            else {
                self.cartridge.dbg_write(address, value);
            }
        }
        else if VRAM.contains(&address) {
            self.vram[address as usize - 0x8000] = value;
        }
        else if CARTRIDGE_RAM.contains(&address) {
            self.cartridge.write(address, value);
        }
        else if WRAM.contains(&address) {
            self.wram[address as usize - 0xC000] = value;
        }
        else if ECHO.contains(&address) {
            self.wram[address as usize - 0xE000] = value;
        }
        else if OAM.contains(&address) {
            self.oam[address as usize - 0xFE00] = value;
        }
        // Unused.
        else if (0xFEA0..=0xFEFF).contains(&address) {
            
        }
        else if IO.contains(&address) {
            self.io[address as usize - 0xFF00] = value;
        }
        else if HRAM.contains(&address) {
            self.hram[address as usize - 0xFF80] = value;
        }
        else {
            self.ie = value;
        }
    }
}
