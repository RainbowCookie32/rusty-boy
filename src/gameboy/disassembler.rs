use std::sync::Arc;

use super::memory::GameboyMemory;

pub fn get_instruction_data(address: u16, gb_mem: &Arc<GameboyMemory>) -> (u16, String) {
    let opcode_value = gb_mem.read(address);

    match opcode_value {
        0x00 => (1, String::from("NOP")),
        0x01 => {
            let args = [gb_mem.read(address + 1), gb_mem.read(address + 2)];
            let dis = format!("LD BC, ${:04X}", u16::from_le_bytes(args));

            (3, dis)
        }
        0x06 => {
            let value = gb_mem.read(address + 1);
            let dis = format!("LD B, ${:04X}", value);

            (2, dis)
        }
        0x0E => {
            let value = gb_mem.read(address + 1);
            let dis = format!("LD C, ${:04X}", value);

            (2, dis)
        }

        0x11 => {
            let args = [gb_mem.read(address + 1), gb_mem.read(address + 2)];
            let dis = format!("LD DE, ${:04X}", u16::from_le_bytes(args));

            (3, dis)
        }
        0x16 => {
            let value = gb_mem.read(address + 1);
            let dis = format!("LD D, ${:04X}", value);

            (2, dis)
        }
        0x1E => {
            let value = gb_mem.read(address + 1);
            let dis = format!("LD E, ${:04X}", value);

            (2, dis)
        }

        0x20 => {
            let offset = gb_mem.read(address + 1) as i8;
            let dis = format!("JP NZ, ${:04X}", address.wrapping_add(offset as u16) + 2);

            (2, dis)
        }
        0x21 => {
            let args = [gb_mem.read(address + 1), gb_mem.read(address + 2)];
            let dis = format!("LD HL, ${:04X}", u16::from_le_bytes(args));

            (3, dis)
        }
        0x26 => {
            let value = gb_mem.read(address + 1);
            let dis = format!("LD H, ${:04X}", value);

            (2, dis)
        }
        0x2E => {
            let value = gb_mem.read(address + 1);
            let dis = format!("LD L, ${:04X}", value);

            (2, dis)
        }

        0x31 => {
            let args = [gb_mem.read(address + 1), gb_mem.read(address + 2)];
            let dis = format!("LD SP, ${:04X}", u16::from_le_bytes(args));

            (3, dis)
        }
        0x32 => (1, String::from("LD [HL-], A")),
        0x3E => {
            let value = gb_mem.read(address + 1);
            let dis = format!("LD A, ${:04X}", value);

            (2, dis)
        }

        0xA8 => (1, String::from("XOR A, B")),
        0xA9 => (1, String::from("XOR A, C")),
        0xAA => (1, String::from("XOR A, D")),
        0xAB => (1, String::from("XOR A, E")),
        0xAC => (1, String::from("XOR A, H")),
        0xAD => (1, String::from("XOR A, L")),
        0xAF => (1, String::from("XOR A, A")),

        0xCB => get_instruction_data_prefixed(address, gb_mem),

        0xE2 => (1, String::from("LD (FF00+C), A")),

        _ => (1, format!("??? (${:02X})", opcode_value))
    }
}

pub fn get_instruction_data_prefixed(address: u16, gb_mem: &Arc<GameboyMemory>) -> (u16, String) {
    let opcode_value = gb_mem.read(address + 1);

    match opcode_value {
        0x7C => (2, format!("BIT 7, H")),

        _ => (2, format!("??? ($CB ${:02X})", opcode_value))
    }
}
