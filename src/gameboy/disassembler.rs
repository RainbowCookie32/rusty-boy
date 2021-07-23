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
        0x02 => (1, String::from("LD (BC), A")),
        0x03 => (1, String::from("INC BC")),
        0x04 => (1, String::from("INC B")),
        0x05 => (1, String::from("DEC B")),
        0x06 => {
            let value = gb_mem.read(address + 1);
            let dis = format!("LD B, ${:04X}", value);

            (2, dis)
        }
        0x08 => (3, format!("??? (${:02X})", opcode_value)),
        0x0A => (1, String::from("LD A, (BC)")),
        0x0C => (1, String::from("INC C")),
        0x0D => (1, String::from("DEC C")),
        0x0E => {
            let value = gb_mem.read(address + 1);
            let dis = format!("LD C, ${:04X}", value);

            (2, dis)
        }

        0x10 => (2, format!("??? (${:02X})", opcode_value)),
        0x11 => {
            let args = [gb_mem.read(address + 1), gb_mem.read(address + 2)];
            let dis = format!("LD DE, ${:04X}", u16::from_le_bytes(args));

            (3, dis)
        }
        0x12 => (1, String::from("LD (DE), A")),
        0x13 => (1, String::from("INC DE")),
        0x14 => (1, String::from("INC D")),
        0x15 => (1, String::from("DEC D")),
        0x16 => {
            let value = gb_mem.read(address + 1);
            let dis = format!("LD D, ${:04X}", value);

            (2, dis)
        }
        0x17 => (1, String::from("RLA")),
        0x18 => {
            let args = [gb_mem.read(address + 1), gb_mem.read(address + 2)];
            let dis = format!("JR ${:04X}", u16::from_le_bytes(args));

            (2, dis)
        }
        0x1A => (1, String::from("LD A, (DE)")),
        0x1C => (1, String::from("INC E")),
        0x1D => (1, String::from("DEC E")),
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
        0x22 => (1, String::from("LD [HL+], A")),
        0x23 => (1, String::from("INC HL")),
        0x24 => (1, String::from("INC H")),
        0x25 => (1, String::from("DEC H")),
        0x26 => {
            let value = gb_mem.read(address + 1);
            let dis = format!("LD H, ${:04X}", value);

            (2, dis)
        }
        0x28 => {
            let offset = gb_mem.read(address + 1) as i8;
            let dis = format!("JP Z, ${:04X}", address.wrapping_add(offset as u16) + 2);

            (2, dis)
        }
        0x2A => (1, String::from("LD A, (HL+)")),
        0x2C => (1, String::from("INC L")),
        0x2D => (1, String::from("DEC L")),
        0x2E => {
            let value = gb_mem.read(address + 1);
            let dis = format!("LD L, ${:04X}", value);

            (2, dis)
        }

        0x30 => {
            let offset = gb_mem.read(address + 1) as i8;
            let dis = format!("JP NC, ${:04X}", address.wrapping_add(offset as u16) + 2);

            (2, dis)
        }
        0x31 => {
            let args = [gb_mem.read(address + 1), gb_mem.read(address + 2)];
            let dis = format!("LD SP, ${:04X}", u16::from_le_bytes(args));

            (3, dis)
        }
        0x32 => (1, String::from("LD [HL-], A")),
        0x33 => (1, String::from("INC SP")),
        0x36 => (2, format!("??? (${:02X})", opcode_value)),
        0x38 => {
            let offset = gb_mem.read(address + 1) as i8;
            let dis = format!("JP C, ${:04X}", address.wrapping_add(offset as u16) + 2);

            (2, dis)
        }
        0x3C => (1, String::from("INC A")),
        0x3D => (1, String::from("DEC A")),
        0x3E => {
            let value = gb_mem.read(address + 1);
            let dis = format!("LD A, ${:04X}", value);

            (2, dis)
        }

        0x40 => (1, String::from("LD B, B")),
        0x41 => (1, String::from("LD B, C")),
        0x42 => (1, String::from("LD B, D")),
        0x43 => (1, String::from("LD B, E")),
        0x44 => (1, String::from("LD B, H")),
        0x45 => (1, String::from("LD B, L")),
        0x47 => (1, String::from("LD B, A")),
        0x46 => (1, String::from("LD B, (HL)")),
        0x48 => (1, String::from("LD C, B")),
        0x49 => (1, String::from("LD C, C")),
        0x4A => (1, String::from("LD C, D")),
        0x4B => (1, String::from("LD C, E")),
        0x4C => (1, String::from("LD C, H")),
        0x4D => (1, String::from("LD C, L")),
        0x4E => (1, String::from("LD C, (HL)")),
        0x4F => (1, String::from("LD C, A")),

        0x50 => (1, String::from("LD D, B")),
        0x51 => (1, String::from("LD D, C")),
        0x52 => (1, String::from("LD D, D")),
        0x53 => (1, String::from("LD D, E")),
        0x54 => (1, String::from("LD D, H")),
        0x55 => (1, String::from("LD D, L")),
        0x56 => (1, String::from("LD D, (HL)")),
        0x57 => (1, String::from("LD D, A")),
        0x58 => (1, String::from("LD E, B")),
        0x59 => (1, String::from("LD E, C")),
        0x5A => (1, String::from("LD E, D")),
        0x5B => (1, String::from("LD E, E")),
        0x5C => (1, String::from("LD E, H")),
        0x5D => (1, String::from("LD E, L")),
        0x5E => (1, String::from("LD E, (HL)")),
        0x5F => (1, String::from("LD E, A")),

        0x60 => (1, String::from("LD H, B")),
        0x61 => (1, String::from("LD H, C")),
        0x62 => (1, String::from("LD H, D")),
        0x63 => (1, String::from("LD H, E")),
        0x64 => (1, String::from("LD H, H")),
        0x65 => (1, String::from("LD H, L")),
        0x66 => (1, String::from("LD H, (HL)")),
        0x67 => (1, String::from("LD H, A")),
        0x68 => (1, String::from("LD L, B")),
        0x69 => (1, String::from("LD L, C")),
        0x6A => (1, String::from("LD L, D")),
        0x6B => (1, String::from("LD L, E")),
        0x6C => (1, String::from("LD L, H")),
        0x6D => (1, String::from("LD L, L")),
        0x6E => (1, String::from("LD L, (HL)")),
        0x6F => (1, String::from("LD L, A")),

        0x70 => (1, String::from("LD (HL), B")),
        0x71 => (1, String::from("LD (HL), C")),
        0x72 => (1, String::from("LD (HL), D")),
        0x73 => (1, String::from("LD (HL), E")),
        0x74 => (1, String::from("LD (HL), H")),
        0x75 => (1, String::from("LD (HL), L")),
        0x77 => (1, String::from("LD (HL), A")),
        0x78 => (1, String::from("LD A, B")),
        0x79 => (1, String::from("LD A, C")),
        0x7A => (1, String::from("LD A, D")),
        0x7B => (1, String::from("LD A, E")),
        0x7C => (1, String::from("LD A, H")),
        0x7D => (1, String::from("LD A, L")),
        0x7F => (1, String::from("LD A, A")),

        0x80 => (1, String::from("ADD A, B")),
        0x81 => (1, String::from("ADD A, C")),
        0x82 => (1, String::from("ADD A, D")),
        0x83 => (1, String::from("ADD A, E")),
        0x84 => (1, String::from("ADD A, H")),
        0x85 => (1, String::from("ADD A, L")),
        0x87 => (1, String::from("ADD A, A")),

        0xA0 => (1, String::from("AND A, B")),
        0xA1 => (1, String::from("AND A, C")),
        0xA2 => (1, String::from("AND A, D")),
        0xA3 => (1, String::from("AND A, E")),
        0xA4 => (1, String::from("AND A, H")),
        0xA5 => (1, String::from("AND A, L")),
        0xA7 => (1, String::from("AND A, A")),
        0xA8 => (1, String::from("XOR A, B")),
        0xA9 => (1, String::from("XOR A, C")),
        0xAA => (1, String::from("XOR A, D")),
        0xAB => (1, String::from("XOR A, E")),
        0xAC => (1, String::from("XOR A, H")),
        0xAD => (1, String::from("XOR A, L")),
        0xAF => (1, String::from("XOR A, A")),

        0xB0 => (1, String::from("OR A, B")),
        0xB1 => (1, String::from("OR A, C")),
        0xB2 => (1, String::from("OR A, D")),
        0xB3 => (1, String::from("OR A, E")),
        0xB4 => (1, String::from("OR A, H")),
        0xB5 => (1, String::from("OR A, L")),
        0xB7 => (1, String::from("OR A, A")),

        0xC1 => (1, String::from("POP BC")),
        0xC2 => (3, format!("??? (${:02X})", opcode_value)),
        0xC3 => {
            let args = [gb_mem.read(address + 1), gb_mem.read(address + 2)];
            let dis = format!("JP ${:04X}", u16::from_le_bytes(args));

            (3, dis)
        }
        0xC4 => {
            let args = [gb_mem.read(address + 1), gb_mem.read(address + 2)];
            let dis = format!("CALL NZ, ${:04X}", u16::from_le_bytes(args));

            (3, dis)
        }
        0xC5 => (1, String::from("PUSH BC")),
        0xC6 => {
            let value = gb_mem.read(address + 1);
            let dis = format!("ADD A, ${:02X}", value);

            (2, dis)
        }
        0xC9 => (1, String::from("RET")),
        0xCA => (3, format!("??? (${:02X})", opcode_value)),
        0xCB => get_instruction_data_prefixed(address, gb_mem),
        0xCC => {
            let args = [gb_mem.read(address + 1), gb_mem.read(address + 2)];
            let dis = format!("CALL Z, ${:04X}", u16::from_le_bytes(args));

            (3, dis)
        }
        0xCD => {
            let args = [gb_mem.read(address + 1), gb_mem.read(address + 2)];
            let dis = format!("CALL ${:04X}", u16::from_le_bytes(args));

            (3, dis)
        }
        0xCE => (2, format!("??? (${:02X})", opcode_value)),

        0xD1 => (1, String::from("POP DE")),
        0xD5 => (1, String::from("PUSH DE")),
        0xD2 => (3, format!("??? (${:02X})", opcode_value)),
        0xD4 => {
            let args = [gb_mem.read(address + 1), gb_mem.read(address + 2)];
            let dis = format!("CALL NC, ${:04X}", u16::from_le_bytes(args));

            (3, dis)
        }
        0xD6 => (2, format!("??? (${:02X})", opcode_value)),
        0xDA => (3, format!("??? (${:02X})", opcode_value)),
        0xDC => {
            let args = [gb_mem.read(address + 1), gb_mem.read(address + 2)];
            let dis = format!("CALL C, ${:04X}", u16::from_le_bytes(args));

            (3, dis)
        }
        0xDE => (2, format!("??? (${:02X})", opcode_value)),

        0xE0 => {
            let offset = gb_mem.read(address + 1);
            let address = 0xFF00 + offset as u16;
            let dis = format!("LD [${:04X}], A", address);

            (2, dis)
        }
        0xE1 => (1, String::from("POP HL")),
        0xE2 => (1, String::from("LD (FF00+C), A")),
        0xE5 => (1, String::from("PUSH HL")),
        0xE6 => {
            let value = gb_mem.read(address + 1);
            let dis = format!("AND A, ${:02X}", value);

            (2, dis)
        }
        0xE8 => (2, format!("??? (${:02X})", opcode_value)),
        0xEA => {
            let args = [gb_mem.read(address + 1), gb_mem.read(address + 2)];
            let dis = format!("LD ${:04X}, A", u16::from_le_bytes(args));

            (3, dis)
        }
        0xEE => (2, format!("??? (${:02X})", opcode_value)),

        0xF0 => {
            let value = gb_mem.read(address + 1);
            let dis = format!("LA A, ${:04X}", 0xFF00 + value as u16);

            (2, dis)
        }
        0xF1 => (1, String::from("POP AF")),
        0xF3 => (1, String::from("DI")),
        0xF5 => (1, String::from("PUSH AF")),
        0xF6 => (2, format!("??? (${:02X})", opcode_value)),
        0xF8 => (2, format!("??? (${:02X})", opcode_value)),
        0xFA => {
            let args = [gb_mem.read(address + 1), gb_mem.read(address + 2)];
            let dis = format!("LD A, [${:04X}]", u16::from_le_bytes(args));

            (3, dis)
        }
        0xFB => (1, String::from("EI")),
        0xFE => {
            let value = gb_mem.read(address + 1);
            let dis = format!("CP A, {:02X}", value);

            (2, dis)
        }

        _ => (1, format!("??? (${:02X})", opcode_value))
    }
}

pub fn get_instruction_data_prefixed(address: u16, gb_mem: &Arc<GameboyMemory>) -> (u16, String) {
    let opcode_value = gb_mem.read(address + 1);

    match opcode_value {
        0x10 => (2, String::from("RL B")),
        0x11 => (2, String::from("RL C")),
        0x12 => (2, String::from("RL D")),
        0x13 => (2, String::from("RL E")),
        0x14 => (2, String::from("RL H")),
        0x15 => (2, String::from("RL L")),
        0x17 => (2, String::from("RL A")),
        
        0x40 => (2, String::from("BIT 0, B")),
        0x41 => (2, String::from("BIT 0, C")),
        0x42 => (2, String::from("BIT 0, D")),
        0x43 => (2, String::from("BIT 0, E")),
        0x44 => (2, String::from("BIT 0, H")),
        0x45 => (2, String::from("BIT 0, L")),
        0x47 => (2, String::from("BIT 0, A")),
        0x48 => (2, String::from("BIT 1, B")),
        0x49 => (2, String::from("BIT 1, C")),
        0x4A => (2, String::from("BIT 1, D")),
        0x4B => (2, String::from("BIT 1, E")),
        0x4C => (2, String::from("BIT 1, H")),
        0x4D => (2, String::from("BIT 1, L")),
        0x4F => (2, String::from("BIT 1, A")),

        0x50 => (2, String::from("BIT 2, B")),
        0x51 => (2, String::from("BIT 2, C")),
        0x52 => (2, String::from("BIT 2, D")),
        0x53 => (2, String::from("BIT 2, E")),
        0x54 => (2, String::from("BIT 2, H")),
        0x55 => (2, String::from("BIT 2, L")),
        0x57 => (2, String::from("BIT 2, A")),
        0x58 => (2, String::from("BIT 3, B")),
        0x59 => (2, String::from("BIT 3, C")),
        0x5A => (2, String::from("BIT 3, D")),
        0x5B => (2, String::from("BIT 3, E")),
        0x5C => (2, String::from("BIT 3, H")),
        0x5D => (2, String::from("BIT 3, L")),
        0x5F => (2, String::from("BIT 3, A")),

        0x60 => (2, String::from("BIT 4, B")),
        0x61 => (2, String::from("BIT 4, C")),
        0x62 => (2, String::from("BIT 4, D")),
        0x63 => (2, String::from("BIT 4, E")),
        0x64 => (2, String::from("BIT 4, H")),
        0x65 => (2, String::from("BIT 4, L")),
        0x67 => (2, String::from("BIT 4, A")),
        0x68 => (2, String::from("BIT 5, B")),
        0x69 => (2, String::from("BIT 5, C")),
        0x6A => (2, String::from("BIT 5, D")),
        0x6B => (2, String::from("BIT 5, E")),
        0x6C => (2, String::from("BIT 5, H")),
        0x6D => (2, String::from("BIT 5, L")),
        0x6F => (2, String::from("BIT 5, A")),

        0x70 => (2, String::from("BIT 6, B")),
        0x71 => (2, String::from("BIT 6, C")),
        0x72 => (2, String::from("BIT 6, D")),
        0x73 => (2, String::from("BIT 6, E")),
        0x74 => (2, String::from("BIT 6, H")),
        0x75 => (2, String::from("BIT 6, L")),
        0x77 => (2, String::from("BIT 6, A")),
        0x78 => (2, String::from("BIT 7, B")),
        0x79 => (2, String::from("BIT 7, C")),
        0x7A => (2, String::from("BIT 7, D")),
        0x7B => (2, String::from("BIT 7, E")),
        0x7C => (2, String::from("BIT 7, H")),
        0x7D => (2, String::from("BIT 7, L")),
        0x7F => (2, String::from("BIT 7, A")),

        _ => (2, format!("??? ($CB ${:02X})", opcode_value))
    }
}
