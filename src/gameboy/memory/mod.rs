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

    header: CartHeader,
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
        let (header, cartridge) = cart::create_cart(romfile_data);
        let bootrom = bootrom_data.into_iter().map(|b| GameboyByte::from(b)).collect();

        GameboyMemory {
            bootrom,

            header,
            cartridge,
            
            vram: vec![GameboyByte::from(0); 0x2000],
            wram: vec![GameboyByte::from(0); 0x2000],

            oam: vec![GameboyByte::from(0); 0x00A0],
            io: vec![GameboyByte::from(0); 0x0080],
            hram: vec![GameboyByte::from(0); 0x007F],

            ie: GameboyByte::from(0)
        }
    }

    pub fn header(&self) -> &CartHeader {
        &self.header
    }

    pub fn read(&self, address: u16) -> u8 {
        if address <= 0x07FF {
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
        else if address >= 0x8000 && address <= 0x9FFF {
            self.vram[address as usize - 0x8000].get()
        }
        else if address >= 0xA000 && address <= 0xBFFF {
            self.cartridge.read(address)
        }
        else if address >= 0xC000 && address <= 0xDFFF {
            self.wram[address as usize - 0xC000].get()
        }
        else if address >= 0xE000 && address <= 0xFDFF {
            self.wram[address as usize - 0xE000].get()
        }
        else if address >= 0xFE00 && address <= 0xFE9F {
            self.oam[address as usize - 0xFE00].get()
        }
        else if address >= 0xFEA0 && address <= 0xFEFF {
            0
        }
        else if address >= 0xFF00 && address <= 0xFF7F {
            self.io[address as usize - 0xFF00].get()
        }
        else if address >= 0xFF80 && address <= 0xFFFE {
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
        else if address >= 0x8000 && address <= 0x9FFF {
            self.vram[address as usize - 0x8000].set(value);
        }
        else if address >= 0xA000 && address <= 0xBFFF {
            self.cartridge.write(address, value);
        }
        else if address >= 0xC000 && address <= 0xDFFF {
            self.wram[address as usize - 0xC000].set(value);
        }
        else if address >= 0xE000 && address <= 0xFDFF {
            self.wram[address as usize - 0xE000].set(value);
        }
        else if address >= 0xFE00 && address <= 0xFE9F {
            self.oam[address as usize - 0xFE00].set(value);
        }
        else if address >= 0xFEA0 && address <= 0xFEFF {
            
        }
        else if address >= 0xFF00 && address <= 0xFF7F {
            self.io[address as usize - 0xFF00].set(value);
        }
        else if address >= 0xFF80 && address <= 0xFFFE {
            self.hram[address as usize - 0xFF80].set(value);
        }
        else {
            self.ie.set(value);
        }
    }
}
