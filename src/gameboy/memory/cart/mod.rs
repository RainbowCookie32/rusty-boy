mod types;

use types::*;

pub struct CartHeader {
    title: String,
    cart_type: CartridgeType,

    rom_size: String,
    rom_banks_count: usize,

    ram_size: String,
    ram_banks_count: usize
}

impl CartHeader {
    pub fn new(data: &Vec<u8>) -> CartHeader {
        let title = {
            let data = data[0x0134..0x0143].to_vec().clone();
            let data_clean: Vec<u8> = data.into_iter().filter(|b| *b > 0).collect();
            
            String::from_utf8_lossy(&data_clean).to_string()
        };

        let cart_type = match data[0x0147] {
            0x00 | 0x08 | 0x09 => CartridgeType::NoController,
            0x01 | 0x02 | 0x03 => CartridgeType::MBC1,
            0x05 | 0x06 => CartridgeType::MBC2,
            0x0F | 0x10 | 0x11 | 0x12 | 0x13 => CartridgeType::MBC3,
            0x19 | 0x1A | 0x1B | 0x1C | 0x1D | 0x1E => CartridgeType::MBC5,
            0x20 => CartridgeType::MBC6,
            _ => unimplemented!("Unknown or invalid cart type")
        };

        let (rom_size, rom_banks_count) = match data[0x0148] {
            0x00 => (String::from("32 KByte"), 2),
            0x01 => (String::from("64 KByte"), 4),
            0x02 => (String::from("128 KByte"), 8),
            0x03 => (String::from("256 KByte"), 16),
            0x04 => (String::from("512 KByte"), 32),
            0x05 => (String::from("1 MByte"), 64),
            0x06 => (String::from("2 MByte"), 128),
            0x07 => (String::from("4 MByte"), 256),
            0x08 => (String::from("8 MByte"), 512),
            _ => unimplemented!("Unknown or invalid ROM size")
        };

        let (ram_size, ram_banks_count) = match data[0x0149] {
            0x00 => (String::from("0 KByte"), 0),
            0x01 => (String::from("0 KByte"), 0),
            0x02 => (String::from("8 KByte"), 1),
            0x03 => (String::from("32 KByte"), 4),
            0x04 => (String::from("128 KByte"), 16),
            0x05 => (String::from("64 KByte"), 8),
            _ => unimplemented!("Unknown or invalid RAM size")
        };

        CartHeader {
            title,
            cart_type,

            rom_size,
            rom_banks_count,

            ram_size,
            ram_banks_count
        }
    }

    /// Get a reference to the cart header's title.
    pub fn title(&self) -> &String {
        &self.title
    }

    /// Get a reference to the cart header's cart type.
    pub fn cart_type(&self) -> &CartridgeType {
        &self.cart_type
    }

    /// Get a reference to the cart header's rom size.
    pub fn rom_size(&self) -> &String {
        &self.rom_size
    }

    /// Get a reference to the cart header's rom banks count.
    pub fn rom_banks_count(&self) -> &usize {
        &self.rom_banks_count
    }

    /// Get a reference to the cart header's ram size.
    pub fn ram_size(&self) -> &String {
        &self.ram_size
    }

    /// Get a reference to the cart header's ram banks count.
    pub fn ram_banks_count(&self) -> &usize {
        &self.ram_banks_count
    }
}

pub trait GameboyCart {
    fn read(&self, address: u16) -> u8;
    fn write(&self, address: u16, value: u8);
    fn dbg_write(&self, address: u16, value: u8);
}

pub fn create_cart(data: Vec<u8>) -> (CartHeader, Box<dyn GameboyCart + Send + Sync>) {
    let header = CartHeader::new(&data);

    match header.cart_type {
        CartridgeType::MBC1 => todo!(),
        CartridgeType::MBC2 => todo!(),
        CartridgeType::MBC3 => todo!(),
        CartridgeType::MBC5 => todo!(),
        CartridgeType::MBC6 => todo!(),
        CartridgeType::NoController => (header, Box::new(no_mbc::NoMBC::new(data)))
    }
}
