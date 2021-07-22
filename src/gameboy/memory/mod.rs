pub mod cart;

use std::sync::atomic::{AtomicU8, Ordering};

use cart::{CartHeader, GameboyCart};

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
    bootrom: Vec<GameboyByte>,
    cartridge: Box<dyn GameboyCart + Send + Sync>,

    vram: Vec<GameboyByte>,
    wram: Vec<GameboyByte>,

    oam: Vec<GameboyByte>,
    io: Vec<GameboyByte>,
    hram: Vec<GameboyByte>,

    ie: GameboyByte
}

impl GameboyMemory {
    pub fn init(bootrom_data: Vec<u8>, romfile_data: Vec<u8>) -> GameboyMemory {
        let cartridge = cart::create_cart(romfile_data);
        let bootrom = bootrom_data.into_iter().map(GameboyByte::from).collect();

        GameboyMemory {
            bootrom,
            cartridge,
            
            vram: vec![GameboyByte::from(0); 0x2000],
            wram: vec![GameboyByte::from(0); 0x2000],

            oam: vec![GameboyByte::from(0); 0x00A0],
            io: vec![GameboyByte::from(0); 0x0080],
            hram: vec![GameboyByte::from(0); 0x007F],

            ie: GameboyByte::from(0)
        }
    }

    pub fn cartridge(&self) -> &Box<dyn GameboyCart + Send + Sync> {
        &self.cartridge
    }

    pub fn header(&self) -> &CartHeader {
        &self.cartridge.get_header()
    }

    pub fn reset(&self) {
        for b in self.vram.iter() {
            b.set(0);
        }

        for b in self.wram.iter() {
            b.set(0);
        }

        for b in self.oam.iter() {
            b.set(0);
        }

        for b in self.io.iter() {
            b.set(0);
        }

        for b in self.hram.iter() {
            b.set(0);
        }

        self.ie.set(0);
    }

    pub fn read(&self, address: u16) -> u8 {
        if address <= 0x7FFF {
            let bootrom_enabled = self.read(0xFF50) == 0;

            if bootrom_enabled {
                if address >= self.bootrom.len() as u16 {
                    self.cartridge.read(address)
                }
                else {
                    self.bootrom[address as usize].get()
                }
            }
            else {
                self.cartridge.read(address)
            }
        }
        else if (0x8000..=0x9FFF).contains(&address) {
            self.vram[address as usize - 0x8000].get()
        }
        else if (0xA000..=0xBFFF).contains(&address) {
            self.cartridge.read(address)
        }
        else if (0xC000..=0xDFFF).contains(&address) {
            self.wram[address as usize - 0xC000].get()
        }
        else if (0xE000..=0xFDFF).contains(&address) {
            self.wram[address as usize - 0xE000].get()
        }
        else if (0xFE00..=0xFE9F).contains(&address) {
            self.oam[address as usize - 0xFE00].get()
        }
        else if (0xFEA0..=0xFEFF).contains(&address) {
            0
        }
        else if (0xFF00..=0xFF7F).contains(&address) {
            self.io[address as usize - 0xFF00].get()
        }
        else if (0xFF80..=0xFFFE).contains(&address) {
            self.hram[address as usize - 0xFF80].get()
        }
        else {
            self.ie.get()
        }
    }

    pub fn write(&self, address: u16, value: u8) {
        if address <= 0x7FFF {
            self.cartridge.write(address, value);
        }
        else if (0x8000..=0x9FFF).contains(&address) {
            self.vram[address as usize - 0x8000].set(value);
        }
        else if (0xA000..=0xBFFF).contains(&address) {
            self.cartridge.write(address, value);
        }
        else if (0xC000..=0xDFFF).contains(&address) {
            self.wram[address as usize - 0xC000].set(value);
        }
        else if (0xE000..=0xFDFF).contains(&address) {
            self.wram[address as usize - 0xE000].set(value);
        }
        else if (0xFE00..=0xFE9F).contains(&address) {
            self.oam[address as usize - 0xFE00].set(value);
        }
        else if (0xFEA0..=0xFEFF).contains(&address) {
            
        }
        else if (0xFF00..=0xFF7F).contains(&address) {
            self.io[address as usize - 0xFF00].set(value);
        }
        else if (0xFF80..=0xFFFE).contains(&address) {
            self.hram[address as usize - 0xFF80].set(value);
        }
        else {
            self.ie.set(value);
        }
    }

    pub fn dbg_write(&self, address: u16, value: u8) {
        if address <= 0x7FFF {
            let bootrom_enabled = self.read(0xFF50) == 0;

            if bootrom_enabled {
                if address >= self.bootrom.len() as u16 {
                    self.cartridge.dbg_write(address, value);
                }
                else {
                    self.bootrom[address as usize].set(value)
                }
            }
            else {
                self.cartridge.dbg_write(address, value);
            }
        }
        else if (0x8000..=0x9FFF).contains(&address) {
            self.vram[address as usize - 0x8000].set(value);
        }
        else if (0xA000..=0xBFFF).contains(&address) {
            self.cartridge.write(address, value);
        }
        else if (0xC000..=0xDFFF).contains(&address) {
            self.wram[address as usize - 0xC000].set(value);
        }
        else if (0xE000..=0xFDFF).contains(&address) {
            self.wram[address as usize - 0xE000].set(value);
        }
        else if (0xFE00..=0xFE9F).contains(&address) {
            self.oam[address as usize - 0xFE00].set(value);
        }
        else if (0xFEA0..=0xFEFF).contains(&address) {
            
        }
        else if (0xFF00..=0xFF7F).contains(&address) {
            self.io[address as usize - 0xFF00].set(value);
        }
        else if (0xFF80..=0xFFFE).contains(&address) {
            self.hram[address as usize - 0xFF80].set(value);
        }
        else {
            self.ie.set(value);
        }
    }
}
