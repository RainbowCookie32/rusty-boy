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
        0x04 => (1, String::from("INC B")),
        0x06 => {
            let value = gb_mem.read(address + 1);
            let dis = format!("LD B, ${:04X}", value);

            (2, dis)
        }
        0x0A => (1, String::from("LD A, (BC)")),
        0x0C => (1, String::from("INC C")),
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
        0x14 => (1, String::from("INC D")),
        0x16 => {
            let value = gb_mem.read(address + 1);
            let dis = format!("LD D, ${:04X}", value);

            (2, dis)
        }
        0x1A => (1, String::from("LD A, (DE)")),
        0x1C => (1, String::from("INC E")),
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
        0x24 => (1, String::from("INC H")),
        0x26 => {
            let value = gb_mem.read(address + 1);
            let dis = format!("LD H, ${:04X}", value);

            (2, dis)
        }
        0x2C => (1, String::from("INC L")),
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
        0x3C => (1, String::from("INC A")),
        0x3E => {
            let value = gb_mem.read(address + 1);
            let dis = format!("LD A, ${:04X}", value);

            (2, dis)
        }

        0x70 => (1, String::from("LD (HL), B")),
        0x71 => (1, String::from("LD (HL), C")),
        0x72 => (1, String::from("LD (HL), D")),
        0x73 => (1, String::from("LD (HL), E")),
        0x74 => (1, String::from("LD (HL), H")),
        0x75 => (1, String::from("LD (HL), L")),
        0x77 => (1, String::from("LD (HL), A")),

        0xA8 => (1, String::from("XOR A, B")),
        0xA9 => (1, String::from("XOR A, C")),
        0xAA => (1, String::from("XOR A, D")),
        0xAB => (1, String::from("XOR A, E")),
        0xAC => (1, String::from("XOR A, H")),
        0xAD => (1, String::from("XOR A, L")),
        0xAF => (1, String::from("XOR A, A")),

        0xCB => get_instruction_data_prefixed(address, gb_mem),
        0xCD => {
            let args = [gb_mem.read(address + 1), gb_mem.read(address + 2)];
            let dis = format!("CALL ${:04X}", u16::from_le_bytes(args));

            (3, dis)
        }

        0xE0 => {
            let offset = gb_mem.read(address + 1);
            let address = 0xFF00 + offset as u16;
            let dis = format!("LD [${:04X}], A", address);

            (2, dis)
        }
        0xE2 => (1, String::from("LD (FF00+C), A")),

        _ => (1, format!("??? (${:02X})", opcode_value))
    }
}

pub fn get_instruction_data_prefixed(address: u16, gb_mem: &Arc<GameboyMemory>) -> (u16, String) {
    let opcode_value = gb_mem.read(address + 1);

    match opcode_value {
        0x40 => (2, format!("BIT 0, B")),
        0x41 => (2, format!("BIT 0, C")),
        0x42 => (2, format!("BIT 0, D")),
        0x43 => (2, format!("BIT 0, E")),
        0x44 => (2, format!("BIT 0, H")),
        0x45 => (2, format!("BIT 0, L")),
        0x47 => (2, format!("BIT 0, A")),
        0x48 => (2, format!("BIT 1, B")),
        0x49 => (2, format!("BIT 1, C")),
        0x4A => (2, format!("BIT 1, D")),
        0x4B => (2, format!("BIT 1, E")),
        0x4C => (2, format!("BIT 1, H")),
        0x4D => (2, format!("BIT 1, L")),
        0x4F => (2, format!("BIT 1, A")),

        0x50 => (2, format!("BIT 2, B")),
        0x51 => (2, format!("BIT 2, C")),
        0x52 => (2, format!("BIT 2, D")),
        0x53 => (2, format!("BIT 2, E")),
        0x54 => (2, format!("BIT 2, H")),
        0x55 => (2, format!("BIT 2, L")),
        0x57 => (2, format!("BIT 2, A")),
        0x58 => (2, format!("BIT 3, B")),
        0x59 => (2, format!("BIT 3, C")),
        0x5A => (2, format!("BIT 3, D")),
        0x5B => (2, format!("BIT 3, E")),
        0x5C => (2, format!("BIT 3, H")),
        0x5D => (2, format!("BIT 3, L")),
        0x5F => (2, format!("BIT 3, A")),

        0x60 => (2, format!("BIT 4, B")),
        0x61 => (2, format!("BIT 4, C")),
        0x62 => (2, format!("BIT 4, D")),
        0x63 => (2, format!("BIT 4, E")),
        0x64 => (2, format!("BIT 4, H")),
        0x65 => (2, format!("BIT 4, L")),
        0x67 => (2, format!("BIT 4, A")),
        0x68 => (2, format!("BIT 5, B")),
        0x69 => (2, format!("BIT 5, C")),
        0x6A => (2, format!("BIT 5, D")),
        0x6B => (2, format!("BIT 5, E")),
        0x6C => (2, format!("BIT 5, H")),
        0x6D => (2, format!("BIT 5, L")),
        0x6F => (2, format!("BIT 5, A")),

        0x70 => (2, format!("BIT 6, B")),
        0x71 => (2, format!("BIT 6, C")),
        0x72 => (2, format!("BIT 6, D")),
        0x73 => (2, format!("BIT 6, E")),
        0x74 => (2, format!("BIT 6, H")),
        0x75 => (2, format!("BIT 6, L")),
        0x77 => (2, format!("BIT 6, A")),
        0x78 => (2, format!("BIT 7, B")),
        0x79 => (2, format!("BIT 7, C")),
        0x7A => (2, format!("BIT 7, D")),
        0x7B => (2, format!("BIT 7, E")),
        0x7C => (2, format!("BIT 7, H")),
        0x7D => (2, format!("BIT 7, L")),
        0x7F => (2, format!("BIT 7, A")),

        _ => (2, format!("??? ($CB ${:02X})", opcode_value))
    }
}
