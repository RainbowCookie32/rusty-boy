use std::sync::Arc;

use super::memory::GameboyMemory;

pub fn get_instruction_data(address: u16, gb_mem: &Arc<GameboyMemory>) -> (u16, String) {
    let opcode_value = gb_mem.read(address);

    match opcode_value {
        0x01 => {
            let args = [gb_mem.read(address + 1), gb_mem.read(address + 2)];
            let dis = format!("LD BC, ${:04X}", u16::from_le_bytes(args));

            (3, dis)
        },

        0x11 => {
            let args = [gb_mem.read(address + 1), gb_mem.read(address + 2)];
            let dis = format!("LD DE, ${:04X}", u16::from_le_bytes(args));

            (3, dis)
        },

        0x21 => {
            let args = [gb_mem.read(address + 1), gb_mem.read(address + 2)];
            let dis = format!("LD HL, ${:04X}", u16::from_le_bytes(args));

            (3, dis)
        },

        0x31 => {
            let args = [gb_mem.read(address + 1), gb_mem.read(address + 2)];
            let dis = format!("LD SP, ${:04X}", u16::from_le_bytes(args));

            (3, dis)
        },
        0x32 => (1, String::from("LD [HL-], A")),

        0xAF => (1, String::from("XOR A, A")),
        _ => (1, format!("??? (${:02X})", opcode_value))
    }
}
