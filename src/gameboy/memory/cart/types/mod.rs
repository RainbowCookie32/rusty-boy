pub mod mbc1;
pub mod no_mbc;

use std::fmt;

pub enum CartridgeType {
    MBC1,
    MBC2,
    MBC3,
    MBC5,
    MBC6,
    NoController
}

impl fmt::Display for CartridgeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CartridgeType::MBC1 => write!(f, "MBC1"),
            CartridgeType::MBC2 => write!(f, "MBC2"),
            CartridgeType::MBC3 => write!(f, "MBC3"),
            CartridgeType::MBC5 => write!(f, "MBC5"),
            CartridgeType::MBC6 => write!(f, "MBC6"),
            CartridgeType::NoController => write!(f, "No Memory Controller")
        }
    }
}
