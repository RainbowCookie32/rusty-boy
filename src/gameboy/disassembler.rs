use std::sync::{Arc, RwLock};

use super::memory::GameboyMemory;

pub fn get_instruction_data(address: u16, gb_mem: &Arc<RwLock<GameboyMemory>>) -> (u16, String) {
    let (opcode_value, imm_1, imm_2) = {
        if let Ok(lock) = gb_mem.read() {
            // FIXME: This will overflow when getting close to $FFFF.
            (lock.read(address), lock.read(address + 1), lock.read(address + 2))
        }
        else {
            (0, 0, 0)
        }
    };

    match opcode_value {
        0x00 => (1, String::from("NOP")),
        0x01 => {
            let args = [imm_1, imm_2];
            let dis = format!("LD BC, ${:04X}", u16::from_le_bytes(args));

            (3, dis)
        }
        0x02 => (1, String::from("LD (BC), A")),
        0x03 => (1, String::from("INC BC")),
        0x04 => (1, String::from("INC B")),
        0x05 => (1, String::from("DEC B")),
        0x06 => {
            let value = imm_1;
            let dis = format!("LD B, ${:02X}", value);

            (2, dis)
        }
        0x08 => {
            let args = [imm_1, imm_2];
            let dis = format!("LD (${:04X}), SP", u16::from_le_bytes(args));

            (3, dis)
        }
        0x09 => (1, String::from("ADD HL, BC")),
        0x0A => (1, String::from("LD A, (BC)")),
        0x0B => (1, String::from("DEC BC")),
        0x0C => (1, String::from("INC C")),
        0x0D => (1, String::from("DEC C")),
        0x0E => {
            let value = imm_1;
            let dis = format!("LD C, ${:02X}", value);

            (2, dis)
        }

        0x10 => (2, format!("??? (${:02X})", opcode_value)),
        0x11 => {
            let args = [imm_1, imm_2];
            let dis = format!("LD DE, ${:04X}", u16::from_le_bytes(args));

            (3, dis)
        }
        0x12 => (1, String::from("LD (DE), A")),
        0x13 => (1, String::from("INC DE")),
        0x14 => (1, String::from("INC D")),
        0x15 => (1, String::from("DEC D")),
        0x16 => {
            let value = imm_1;
            let dis = format!("LD D, ${:02X}", value);

            (2, dis)
        }
        0x17 => (1, String::from("RLA")),
        0x18 => {
            let offset = imm_1 as i8;
            let target = address.wrapping_add(offset as u16) + 2;
            let dis = format!("JR ${:04X}", target);

            (2, dis)
        }
        0x19 => (1, String::from("ADD HL, DE")),
        0x1A => (1, String::from("LD A, (DE)")),
        0x1B => (1, String::from("DEC DE")),
        0x1C => (1, String::from("INC E")),
        0x1D => (1, String::from("DEC E")),
        0x1E => {
            let value = imm_1;
            let dis = format!("LD E, ${:02X}", value);

            (2, dis)
        }
        0x1F => (1, String::from("RRA")),

        0x20 => {
            let offset = imm_1 as i8;
            let dis = format!("JR NZ, ${:04X}", address.wrapping_add(offset as u16) + 2);

            (2, dis)
        }
        0x21 => {
            let args = [imm_1, imm_2];
            let dis = format!("LD HL, ${:04X}", u16::from_le_bytes(args));

            (3, dis)
        }
        0x22 => (1, String::from("LD [HL+], A")),
        0x23 => (1, String::from("INC HL")),
        0x24 => (1, String::from("INC H")),
        0x25 => (1, String::from("DEC H")),
        0x26 => {
            let value = imm_1;
            let dis = format!("LD H, ${:02X}", value);

            (2, dis)
        }
        0x28 => {
            let offset = imm_1 as i8;
            let dis = format!("JR Z, ${:04X}", address.wrapping_add(offset as u16) + 2);

            (2, dis)
        }
        0x29 => (1, String::from("ADD HL, HL")),
        0x2A => (1, String::from("LD A, (HL+)")),
        0x2B => (1, String::from("DEC HL")),
        0x2C => (1, String::from("INC L")),
        0x2D => (1, String::from("DEC L")),
        0x2E => {
            let value = imm_1;
            let dis = format!("LD L, ${:02X}", value);

            (2, dis)
        }
        0x2F => (1, String::from("CPL")),

        0x30 => {
            let offset = imm_1 as i8;
            let dis = format!("JR NC, ${:04X}", address.wrapping_add(offset as u16) + 2);

            (2, dis)
        }
        0x31 => {
            let args = [imm_1, imm_2];
            let dis = format!("LD SP, ${:04X}", u16::from_le_bytes(args));

            (3, dis)
        }
        0x32 => (1, String::from("LD [HL-], A")),
        0x33 => (1, String::from("INC SP")),
        0x34 => (1, String::from("INC (HL)")),
        0x35 => (1, String::from("DEC (HL)")),
        0x36 => {
            let value = imm_1;
            let dis = format!("LD [HL], ${:02X}", value);

            (2, dis)
        }
        0x37 => (1, String::from("SCF")),
        0x38 => {
            let offset = imm_1 as i8;
            let dis = format!("JR C, ${:04X}", address.wrapping_add(offset as u16) + 2);

            (2, dis)
        }
        0x39 => (1, String::from("ADD HL, SP")),
        0x3A => (1, String::from("LD A, (HL-)")),
        0x3B => (1, String::from("DEC SP")),
        0x3C => (1, String::from("INC A")),
        0x3D => (1, String::from("DEC A")),
        0x3E => {
            let value = imm_1;
            let dis = format!("LD A, ${:02X}", value);

            (2, dis)
        }
        0x3F => (1, String::from("CCF")),

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
        0x7E => (1, String::from("LD A, (HL)")),
        0x7F => (1, String::from("LD A, A")),

        0x80 => (1, String::from("ADD A, B")),
        0x81 => (1, String::from("ADD A, C")),
        0x82 => (1, String::from("ADD A, D")),
        0x83 => (1, String::from("ADD A, E")),
        0x84 => (1, String::from("ADD A, H")),
        0x85 => (1, String::from("ADD A, L")),
        0x86 => (1, String::from("ADD A, (HL)")),
        0x87 => (1, String::from("ADD A, A")),
        0x88 => (1, String::from("ADC A, B")),
        0x89 => (1, String::from("ADC A, C")),
        0x8A => (1, String::from("ADC A, D")),
        0x8B => (1, String::from("ADC A, E")),
        0x8C => (1, String::from("ADC A, H")),
        0x8D => (1, String::from("ADC A, L")),
        0x8E => (1, String::from("ADC A, (HL)")),
        0x8F => (1, String::from("ADC A, A")),

        0x90 => (1, String::from("SUB A, B")),
        0x91 => (1, String::from("SUB A, C")),
        0x92 => (1, String::from("SUB A, D")),
        0x93 => (1, String::from("SUB A, E")),
        0x94 => (1, String::from("SUB A, H")),
        0x95 => (1, String::from("SUB A, L")),
        0x96 => (1, String::from("SUB A, (HL)")),
        0x97 => (1, String::from("SUB A, A")),
        0x98 => (1, String::from("SBC A, B")),
        0x99 => (1, String::from("SBC A, C")),
        0x9A => (1, String::from("SBC A, D")),
        0x9B => (1, String::from("SBC A, E")),
        0x9C => (1, String::from("SBC A, H")),
        0x9D => (1, String::from("SBC A, L")),
        0x9E => (1, String::from("SBC A, (HL)")),
        0x9F => (1, String::from("SBC A, A")),

        0xA0 => (1, String::from("AND A, B")),
        0xA1 => (1, String::from("AND A, C")),
        0xA2 => (1, String::from("AND A, D")),
        0xA3 => (1, String::from("AND A, E")),
        0xA4 => (1, String::from("AND A, H")),
        0xA5 => (1, String::from("AND A, L")),
        0xA6 => (1, String::from("AND A, (HL)")),
        0xA7 => (1, String::from("AND A, A")),
        0xA8 => (1, String::from("XOR A, B")),
        0xA9 => (1, String::from("XOR A, C")),
        0xAA => (1, String::from("XOR A, D")),
        0xAB => (1, String::from("XOR A, E")),
        0xAC => (1, String::from("XOR A, H")),
        0xAD => (1, String::from("XOR A, L")),
        0xAE => (1, String::from("XOR A, (HL)")),
        0xAF => (1, String::from("XOR A, A")),

        0xB0 => (1, String::from("OR A, B")),
        0xB1 => (1, String::from("OR A, C")),
        0xB2 => (1, String::from("OR A, D")),
        0xB3 => (1, String::from("OR A, E")),
        0xB4 => (1, String::from("OR A, H")),
        0xB5 => (1, String::from("OR A, L")),
        0xB6 => (1, String::from("OR A, (HL)")),
        0xB7 => (1, String::from("OR A, A")),
        0xB8 => (1, String::from("CP A, B")),
        0xB9 => (1, String::from("CP A, C")),
        0xBA => (1, String::from("CP A, D")),
        0xBB => (1, String::from("CP A, E")),
        0xBC => (1, String::from("CP A, H")),
        0xBD => (1, String::from("CP A, L")),
        0xBE => (1, String::from("CP A, (HL)")),
        0xBF => (1, String::from("CP A, A")),

        0xC0 => (1, String::from("RET NZ")),
        0xC1 => (1, String::from("POP BC")),
        0xC2 => {
            let args = [imm_1, imm_2];
            let dis = format!("JP NZ, ${:04X}", u16::from_le_bytes(args));

            (3, dis)
        }
        0xC3 => {
            let args = [imm_1, imm_2];
            let dis = format!("JP ${:04X}", u16::from_le_bytes(args));

            (3, dis)
        }
        0xC4 => {
            let args = [imm_1, imm_2];
            let dis = format!("CALL NZ, ${:04X}", u16::from_le_bytes(args));

            (3, dis)
        }
        0xC5 => (1, String::from("PUSH BC")),
        0xC6 => {
            let value = imm_1;
            let dis = format!("ADD A, ${:02X}", value);

            (2, dis)
        }
        0xC7 => (1, String::from("RST $00")),
        0xC8 => (1, String::from("RET Z")),
        0xC9 => (1, String::from("RET")),
        0xCA => {
            let args = [imm_1, imm_2];
            let dis = format!("JP Z, ${:04X}", u16::from_le_bytes(args));

            (3, dis)
        }
        0xCB => get_instruction_data_prefixed(address, gb_mem),
        0xCC => {
            let args = [imm_1, imm_2];
            let dis = format!("CALL Z, ${:04X}", u16::from_le_bytes(args));

            (3, dis)
        }
        0xCD => {
            let args = [imm_1, imm_2];
            let dis = format!("CALL ${:04X}", u16::from_le_bytes(args));

            (3, dis)
        }
        0xCE => {
            let value = imm_1;
            let dis = format!("ADC A, ${:02X}", value);

            (2, dis)
        }
        0xCF => (1, String::from("RST $08")),

        0xD0 => (1, String::from("RET NC")),
        0xD1 => (1, String::from("POP DE")),
        0xD2 => {
            let args = [imm_1, imm_2];
            let dis = format!("JP NC, ${:04X}", u16::from_le_bytes(args));

            (3, dis)
        }
        0xD4 => {
            let args = [imm_1, imm_2];
            let dis = format!("CALL NC, ${:04X}", u16::from_le_bytes(args));

            (3, dis)
        }
        0xD5 => (1, String::from("PUSH DE")),
        0xD6 => {
            let value = imm_1;
            let dis = format!("SUB A, ${:02X}", value);

            (2, dis)
        }
        0xD7 => (1, String::from("RST $10")),
        0xD8 => (1, String::from("RET C")),
        0xD9 => (1, String::from("RETI")),
        0xDA => {
            let args = [imm_1, imm_2];
            let dis = format!("JP C, ${:04X}", u16::from_le_bytes(args));

            (3, dis)
        }
        0xDC => {
            let args = [imm_1, imm_2];
            let dis = format!("CALL C, ${:04X}", u16::from_le_bytes(args));

            (3, dis)
        }
        0xDE => {
            let value = imm_1;
            let dis = format!("SBC A, ${:02X}", value);

            (2, dis)
        }
        0xDF => (1, String::from("RST $18")),

        0xE0 => {
            let offset = imm_1;
            let address = 0xFF00 + offset as u16;
            let dis = format!("LD [${:04X}], A", address);

            (2, dis)
        }
        0xE1 => (1, String::from("POP HL")),
        0xE2 => (1, String::from("LD (FF00+C), A")),
        0xE5 => (1, String::from("PUSH HL")),
        0xE6 => {
            let value = imm_1;
            let dis = format!("AND A, ${:02X}", value);

            (2, dis)
        }
        0xE7 => (1, String::from("RST $20")),
        0xE8 => {
            let value = imm_1;
            let dis = format!("ADD SP, ${:02X}", value);

            (2, dis)
        }
        0xE9 => (1, String::from("JP HL")),
        0xEA => {
            let args = [imm_1, imm_2];
            let dis = format!("LD ${:04X}, A", u16::from_le_bytes(args));

            (3, dis)
        }
        0xEE => {
            let value = imm_1;
            let dis = format!("XOR A, ${:02X}", value);

            (2, dis)
        }
        0xEF => (1, String::from("RST $28")),

        0xF0 => {
            let value = imm_1;
            let dis = format!("LD A, ${:04X}", 0xFF00 + value as u16);

            (2, dis)
        }
        0xF1 => (1, String::from("POP AF")),
        0xF2 => (1, String::from("LD A, (FF00+C)")),
        0xF3 => (1, String::from("DI")),
        0xF5 => (1, String::from("PUSH AF")),
        0xF6 => {
            let value = imm_1;
            let dis = format!("OR A, ${:02X}", value);

            (2, dis)
        }
        0xF7 => (1, String::from("RST $30")),
        0xF8 => {
            let value = imm_1;
            let dis = format!("LD HL, SP+${:02X}", value);

            (2, dis)
        }
        0xF9 => (1, String::from("LD SP, HL")),
        0xFA => {
            let args = [imm_1, imm_2];
            let dis = format!("LD A, [${:04X}]", u16::from_le_bytes(args));

            (3, dis)
        }
        0xFB => (1, String::from("EI")),
        0xFE => {
            let dis = format!("CP A, {:02X}", imm_1);

            (2, dis)
        }
        0xFF => (1, String::from("RST $38")),

        _ => (1, format!("??? (${:02X})", opcode_value))
    }
}

pub fn get_instruction_data_prefixed(address: u16, gb_mem: &Arc<RwLock<GameboyMemory>>) -> (u16, String) {
    let opcode_value = {
        if let Ok(lock) = gb_mem.read() {
            lock.read(address)
        }
        else {
            0
        }
    };

    match opcode_value {
        0x00 => (2, String::from("RLC B")),
        0x01 => (2, String::from("RLC C")),
        0x02 => (2, String::from("RLC D")),
        0x03 => (2, String::from("RLC E")),
        0x04 => (2, String::from("RLC H")),
        0x05 => (2, String::from("RLC L")),
        0x06 => (2, String::from("RLC (HL)")),
        0x07 => (2, String::from("RLC A")),
        0x08 => (2, String::from("RRC B")),
        0x09 => (2, String::from("RRC C")),
        0x0A => (2, String::from("RRC D")),
        0x0B => (2, String::from("RRC E")),
        0x0C => (2, String::from("RRC H")),
        0x0D => (2, String::from("RRC L")),
        0x0E => (2, String::from("RRC (HL)")),
        0x0F => (2, String::from("RRC A")),

        0x10 => (2, String::from("RL B")),
        0x11 => (2, String::from("RL C")),
        0x12 => (2, String::from("RL D")),
        0x13 => (2, String::from("RL E")),
        0x14 => (2, String::from("RL H")),
        0x15 => (2, String::from("RL L")),
        0x16 => (2, String::from("RL (HL)")),
        0x17 => (2, String::from("RL A")),
        0x18 => (2, String::from("RR B")),
        0x19 => (2, String::from("RR C")),
        0x1A => (2, String::from("RR D")),
        0x1B => (2, String::from("RR E")),
        0x1C => (2, String::from("RR H")),
        0x1D => (2, String::from("RR L")),
        0x1E => (2, String::from("RR (HL)")),
        0x1F => (2, String::from("RR A")),

        0x20 => (2, String::from("SLA B")),
        0x21 => (2, String::from("SLA C")),
        0x22 => (2, String::from("SLA D")),
        0x23 => (2, String::from("SLA E")),
        0x24 => (2, String::from("SLA H")),
        0x25 => (2, String::from("SLA L")),
        0x26 => (2, String::from("SLA (HL)")),
        0x27 => (2, String::from("SLA A")),
        0x28 => (2, String::from("SRA B")),
        0x29 => (2, String::from("SRA C")),
        0x2A => (2, String::from("SRA D")),
        0x2B => (2, String::from("SRA E")),
        0x2C => (2, String::from("SRA H")),
        0x2D => (2, String::from("SRA L")),
        0x2E => (2, String::from("SRA (HL)")),
        0x2F => (2, String::from("SRA A")),

        0x30 => (2, String::from("SWAP B")),
        0x31 => (2, String::from("SWAP C")),
        0x32 => (2, String::from("SWAP D")),
        0x33 => (2, String::from("SWAP E")),
        0x34 => (2, String::from("SWAP H")),
        0x35 => (2, String::from("SWAP L")),
        0x36 => (2, String::from("SWAP (HL)")),
        0x37 => (2, String::from("SWAP A")),
        0x38 => (2, String::from("SRL B")),
        0x39 => (2, String::from("SRL C")),
        0x3A => (2, String::from("SRL D")),
        0x3B => (2, String::from("SRL E")),
        0x3C => (2, String::from("SRL H")),
        0x3D => (2, String::from("SRL L")),
        0x3E => (2, String::from("SRL (HL)")),
        0x3F => (2, String::from("SRL A")),
        
        0x40 => (2, String::from("BIT 0, B")),
        0x41 => (2, String::from("BIT 0, C")),
        0x42 => (2, String::from("BIT 0, D")),
        0x43 => (2, String::from("BIT 0, E")),
        0x44 => (2, String::from("BIT 0, H")),
        0x45 => (2, String::from("BIT 0, L")),
        0x46 => (2, String::from("BIT 0, [HL]")),
        0x47 => (2, String::from("BIT 0, A")),
        0x48 => (2, String::from("BIT 1, B")),
        0x49 => (2, String::from("BIT 1, C")),
        0x4A => (2, String::from("BIT 1, D")),
        0x4B => (2, String::from("BIT 1, E")),
        0x4C => (2, String::from("BIT 1, H")),
        0x4D => (2, String::from("BIT 1, L")),
        0x4E => (2, String::from("BIT 1, [HL]")),
        0x4F => (2, String::from("BIT 1, A")),

        0x50 => (2, String::from("BIT 2, B")),
        0x51 => (2, String::from("BIT 2, C")),
        0x52 => (2, String::from("BIT 2, D")),
        0x53 => (2, String::from("BIT 2, E")),
        0x54 => (2, String::from("BIT 2, H")),
        0x55 => (2, String::from("BIT 2, L")),
        0x56 => (2, String::from("BIT 2, [HL]")),
        0x57 => (2, String::from("BIT 2, A")),
        0x58 => (2, String::from("BIT 3, B")),
        0x59 => (2, String::from("BIT 3, C")),
        0x5A => (2, String::from("BIT 3, D")),
        0x5B => (2, String::from("BIT 3, E")),
        0x5C => (2, String::from("BIT 3, H")),
        0x5D => (2, String::from("BIT 3, L")),
        0x5E => (2, String::from("BIT 3, [HL]")),
        0x5F => (2, String::from("BIT 3, A")),

        0x60 => (2, String::from("BIT 4, B")),
        0x61 => (2, String::from("BIT 4, C")),
        0x62 => (2, String::from("BIT 4, D")),
        0x63 => (2, String::from("BIT 4, E")),
        0x64 => (2, String::from("BIT 4, H")),
        0x65 => (2, String::from("BIT 4, L")),
        0x66 => (2, String::from("BIT 4, [HL]")),
        0x67 => (2, String::from("BIT 4, A")),
        0x68 => (2, String::from("BIT 5, B")),
        0x69 => (2, String::from("BIT 5, C")),
        0x6A => (2, String::from("BIT 5, D")),
        0x6B => (2, String::from("BIT 5, E")),
        0x6C => (2, String::from("BIT 5, H")),
        0x6D => (2, String::from("BIT 5, L")),
        0x6E => (2, String::from("BIT 5, [HL]")),
        0x6F => (2, String::from("BIT 5, A")),

        0x70 => (2, String::from("BIT 6, B")),
        0x71 => (2, String::from("BIT 6, C")),
        0x72 => (2, String::from("BIT 6, D")),
        0x73 => (2, String::from("BIT 6, E")),
        0x74 => (2, String::from("BIT 6, H")),
        0x75 => (2, String::from("BIT 6, L")),
        0x76 => (2, String::from("BIT 6, [HL]")),
        0x77 => (2, String::from("BIT 6, A")),
        0x78 => (2, String::from("BIT 7, B")),
        0x79 => (2, String::from("BIT 7, C")),
        0x7A => (2, String::from("BIT 7, D")),
        0x7B => (2, String::from("BIT 7, E")),
        0x7C => (2, String::from("BIT 7, H")),
        0x7D => (2, String::from("BIT 7, L")),
        0x7E => (2, String::from("BIT 7, [HL]")),
        0x7F => (2, String::from("BIT 7, A")),

        0x80 => (2, String::from("RES 0, B")),
        0x81 => (2, String::from("RES 0, C")),
        0x82 => (2, String::from("RES 0, D")),
        0x83 => (2, String::from("RES 0, E")),
        0x84 => (2, String::from("RES 0, H")),
        0x85 => (2, String::from("RES 0, L")),
        0x86 => (2, String::from("RES 0, [HL]")),
        0x87 => (2, String::from("RES 0, A")),
        0x88 => (2, String::from("RES 1, B")),
        0x89 => (2, String::from("RES 1, C")),
        0x8A => (2, String::from("RES 1, D")),
        0x8B => (2, String::from("RES 1, E")),
        0x8C => (2, String::from("RES 1, H")),
        0x8D => (2, String::from("RES 1, L")),
        0x8E => (2, String::from("RES 1, [HL]")),
        0x8F => (2, String::from("RES 1, A")),

        0x90 => (2, String::from("RES 2, B")),
        0x91 => (2, String::from("RES 2, C")),
        0x92 => (2, String::from("RES 2, D")),
        0x93 => (2, String::from("RES 2, E")),
        0x94 => (2, String::from("RES 2, H")),
        0x95 => (2, String::from("RES 2, L")),
        0x96 => (2, String::from("RES 2, [HL]")),
        0x97 => (2, String::from("RES 2, A")),
        0x98 => (2, String::from("RES 3, B")),
        0x99 => (2, String::from("RES 3, C")),
        0x9A => (2, String::from("RES 3, D")),
        0x9B => (2, String::from("RES 3, E")),
        0x9C => (2, String::from("RES 3, H")),
        0x9D => (2, String::from("RES 3, L")),
        0x9E => (2, String::from("RES 3, [HL]")),
        0x9F => (2, String::from("RES 3, A")),

        0xA0 => (2, String::from("RES 4, B")),
        0xA1 => (2, String::from("RES 4, C")),
        0xA2 => (2, String::from("RES 4, D")),
        0xA3 => (2, String::from("RES 4, E")),
        0xA4 => (2, String::from("RES 4, H")),
        0xA5 => (2, String::from("RES 4, L")),
        0xA6 => (2, String::from("RES 4, [HL]")),
        0xA7 => (2, String::from("RES 4, A")),
        0xA8 => (2, String::from("RES 5, B")),
        0xA9 => (2, String::from("RES 5, C")),
        0xAA => (2, String::from("RES 5, D")),
        0xAB => (2, String::from("RES 5, E")),
        0xAC => (2, String::from("RES 5, H")),
        0xAD => (2, String::from("RES 5, L")),
        0xAE => (2, String::from("RES 5, [HL]")),
        0xAF => (2, String::from("RES 5, A")),

        0xB0 => (2, String::from("RES 6, B")),
        0xB1 => (2, String::from("RES 6, C")),
        0xB2 => (2, String::from("RES 6, D")),
        0xB3 => (2, String::from("RES 6, E")),
        0xB4 => (2, String::from("RES 6, H")),
        0xB5 => (2, String::from("RES 6, L")),
        0xB6 => (2, String::from("RES 6, [HL]")),
        0xB7 => (2, String::from("RES 6, A")),
        0xB8 => (2, String::from("RES 7, B")),
        0xB9 => (2, String::from("RES 7, C")),
        0xBA => (2, String::from("RES 7, D")),
        0xBB => (2, String::from("RES 7, E")),
        0xBC => (2, String::from("RES 7, H")),
        0xBD => (2, String::from("RES 7, L")),
        0xBE => (2, String::from("RES 7, [HL]")),
        0xBF => (2, String::from("RES 7, A")),

        0xC0 => (2, String::from("SET 0, B")),
        0xC1 => (2, String::from("SET 0, C")),
        0xC2 => (2, String::from("SET 0, D")),
        0xC3 => (2, String::from("SET 0, E")),
        0xC4 => (2, String::from("SET 0, H")),
        0xC5 => (2, String::from("SET 0, L")),
        0xC7 => (2, String::from("SET 0, A")),
        0xC6 => (2, String::from("SET 0, [HL]")),
        0xC8 => (2, String::from("SET 1, B")),
        0xC9 => (2, String::from("SET 1, C")),
        0xCA => (2, String::from("SET 1, D")),
        0xCB => (2, String::from("SET 1, E")),
        0xCC => (2, String::from("SET 1, H")),
        0xCD => (2, String::from("SET 1, L")),
        0xCE => (2, String::from("SET 1, [HL]")),
        0xCF => (2, String::from("SET 1, A")),

        0xD0 => (2, String::from("SET 2, B")),
        0xD1 => (2, String::from("SET 2, C")),
        0xD2 => (2, String::from("SET 2, D")),
        0xD3 => (2, String::from("SET 2, E")),
        0xD4 => (2, String::from("SET 2, H")),
        0xD5 => (2, String::from("SET 2, L")),
        0xD6 => (2, String::from("SET 2, [HL]")),
        0xD7 => (2, String::from("SET 2, A")),
        0xD8 => (2, String::from("SET 3, B")),
        0xD9 => (2, String::from("SET 3, C")),
        0xDA => (2, String::from("SET 3, D")),
        0xDB => (2, String::from("SET 3, E")),
        0xDC => (2, String::from("SET 3, H")),
        0xDD => (2, String::from("SET 3, L")),
        0xDE => (2, String::from("SET 3, [HL]")),
        0xDF => (2, String::from("SET 3, A")),

        0xE0 => (2, String::from("SET 4, B")),
        0xE1 => (2, String::from("SET 4, C")),
        0xE2 => (2, String::from("SET 4, D")),
        0xE3 => (2, String::from("SET 4, E")),
        0xE4 => (2, String::from("SET 4, H")),
        0xE5 => (2, String::from("SET 4, L")),
        0xE6 => (2, String::from("SET 4, [HL]")),
        0xE7 => (2, String::from("SET 4, A")),
        0xE8 => (2, String::from("SET 5, B")),
        0xE9 => (2, String::from("SET 5, C")),
        0xEA => (2, String::from("SET 5, D")),
        0xEB => (2, String::from("SET 5, E")),
        0xEC => (2, String::from("SET 5, H")),
        0xED => (2, String::from("SET 5, L")),
        0xEE => (2, String::from("SET 5, [HL]")),
        0xEF => (2, String::from("SET 5, A")),

        0xF0 => (2, String::from("SET 6, B")),
        0xF1 => (2, String::from("SET 6, C")),
        0xF2 => (2, String::from("SET 6, D")),
        0xF3 => (2, String::from("SET 6, E")),
        0xF4 => (2, String::from("SET 6, H")),
        0xF5 => (2, String::from("SET 6, L")),
        0xF6 => (2, String::from("SET 6, [HL]")),
        0xF7 => (2, String::from("SET 6, A")),
        0xF8 => (2, String::from("SET 7, B")),
        0xF9 => (2, String::from("SET 7, C")),
        0xFA => (2, String::from("SET 7, D")),
        0xFB => (2, String::from("SET 7, E")),
        0xFC => (2, String::from("SET 7, H")),
        0xFD => (2, String::from("SET 7, L")),
        0xFE => (2, String::from("SET 7, [HL]")),
        0xFF => (2, String::from("SET 7, A"))
    }
}
