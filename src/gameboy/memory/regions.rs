use std::ops::RangeInclusive;

pub const CARTRIDGE_ROM: RangeInclusive<u16> = 0x0000..=0x7FFF;
pub const CARTRIDGE_RAM: RangeInclusive<u16> = 0xA000..=0xBFFF;
pub const CARTRIDGE_ROM_BANK0: RangeInclusive<u16> = 0x0000..=0x3FFF;
pub const CARTRIDGE_ROM_BANKX: RangeInclusive<u16> = 0x4000..=0x7FFF;

pub const MBC1_RAMG: RangeInclusive<u16> = 0x0000..=0x1FFF;
pub const MBC1_BANK1: RangeInclusive<u16> = 0x2000..=0x3FFF;
pub const MBC1_BANK2: RangeInclusive<u16> = 0x4000..=0x5FFF;
pub const MBC1_MODE: RangeInclusive<u16> = 0x6000..=0x7FFF;

pub const MBC5_RAMG: RangeInclusive<u16> = 0x0000..=0x1FFF;
pub const MBC5_ROMB0: RangeInclusive<u16> = 0x2000..=0x2FFF;
pub const MBC5_ROMB1: RangeInclusive<u16> = 0x3000..=0x3FFF;
pub const MBC5_RAMB: RangeInclusive<u16> = 0x4000..=0x5FFF;

pub const VRAM: RangeInclusive<u16> = 0x8000..=0x9FFF;
pub const WRAM: RangeInclusive<u16> = 0xC000..=0xDFFF;
pub const ECHO: RangeInclusive<u16> = 0xE000..=0xFDFF;
pub const OAM: RangeInclusive<u16> = 0xFE00..=0xFE9F;
pub const IO: RangeInclusive<u16> = 0xFF00..=0xFF7F;
pub const HRAM: RangeInclusive<u16> = 0xFF80..=0xFFFE;
