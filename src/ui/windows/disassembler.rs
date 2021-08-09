use std::sync::{Arc, RwLock};

use imgui::*;

use crate::gameboy::disassembler;
use crate::gameboy::memory::regions::*;
use crate::gameboy::memory::GameboyMemory;
use crate::gameboy::{Breakpoint, EmulatorMode, Gameboy};

pub struct DisassemblerWindow {
    gb: Arc<RwLock<Gameboy>>,
    gb_mem: Arc<GameboyMemory>,

    adjusted_cursor: bool
}

impl DisassemblerWindow {
    pub fn init(gb: Arc<RwLock<Gameboy>>, gb_mem: Arc<GameboyMemory>) -> DisassemblerWindow {
        DisassemblerWindow {
            gb,
            gb_mem,

            adjusted_cursor: true
        }
    }

    pub fn draw(&mut self, ui: &Ui, adjust: bool) {
        let pc = {
            if let Ok(lock) = self.gb.read() {
                let (_, _, _, _, _, pc) = lock.ui_get_cpu_registers();
                *pc
            }
            else {
                0
            }
        };

        Window::new(im_str!("Disassembler")).size([300.0, 325.0], Condition::FirstUseEver).build(ui, || {
            let mut clipper = ListClipper::new(0xFFFF).items_height(ui.text_line_height() / 2.0).begin(ui);
            clipper.step();

            let mut skipped_lines = 0;
            let mut last_instruction_len = 0;

            for line in clipper.display_start()..clipper.display_end() {
                if skipped_lines == last_instruction_len {
                    let current_addr = line as u16;
                    let (len, dis) = disassembler::get_instruction_data(current_addr, &self.gb_mem);

                    let line_p = if pc == current_addr {"> "} else {""};
                    let address_p = {
                        if CARTRIDGE_ROM_BANK0.contains(&current_addr) {
                            String::from("ROM00")
                        }
                        else if CARTRIDGE_ROM_BANKX.contains(&current_addr) {
                            format!("ROM{:02}", self.gb_mem.cartridge().get_selected_rom_bank())
                        }
                        else if VRAM.contains(&current_addr) {
                            String::from("VRAM")
                        }
                        else if CARTRIDGE_RAM.contains(&current_addr) {
                            String::from("CRAM")
                        }
                        else if WRAM.contains(&current_addr) {
                            String::from("WRAM")
                        }
                        else if ECHO.contains(&current_addr) {
                            String::from("ECHO")
                        }
                        else if OAM.contains(&current_addr) {
                            String::from("OAM")
                        }
                        else if (0xFEA0..=0xFEFF).contains(&current_addr) {
                            String::from("UNK")
                        }
                        else if IO.contains(&current_addr) {
                            String::from("IO")
                        }
                        else if HRAM.contains(&current_addr) {
                            String::from("HRAM")
                        }
                        else {
                            String::from("IE")
                        }
                    };
                    let line_str = format!("{}{}: {:04X} - {}", line_p, address_p, current_addr, dis);

                    skipped_lines = 1;
                    last_instruction_len = len;

                    let mut bp_idx = 0;
                    let mut address_is_bp = false;

                    if let Ok(lock) = self.gb.read() {
                        for (idx, bp) in lock.dbg_breakpoint_list.iter().enumerate() {
                            if current_addr == *bp.address() && *bp.execute() {
                                bp_idx = idx;
                                address_is_bp = true;

                                break;
                            }
                        }
                    }

                    let text = ImString::from(line_str);
                    let widget = Selectable::new(&text).allow_double_click(true);

                    let entry = || if widget.build(ui) && ui.is_mouse_double_clicked(MouseButton::Left) {
                        if let Ok(mut lock) = self.gb.write() {
                            if address_is_bp {
                                lock.dbg_breakpoint_list.remove(bp_idx);
                            }
                            else {
                                lock.dbg_breakpoint_list.push(
                                    Breakpoint::new(false, false, true, current_addr)
                                );
                            }
                        }
                    };

                    if address_is_bp {
                        let token = ui.push_style_color(StyleColor::Text, [1.0, 0.0, 0.0, 1.0]);

                        (entry)();

                        token.pop(ui);
                    }
                    else if pc == current_addr {
                        let token = ui.push_style_color(StyleColor::Text, [0.0, 1.0, 0.0, 1.0]);

                        (entry)();

                        token.pop(ui);
                    }
                    else {
                        (entry)();
                    }
                }
                else {
                    skipped_lines += 1;
                }
            }

            clipper.end();

            if adjust {
                if let Ok(lock) = self.gb.read() {
                    match lock.dbg_mode {
                        EmulatorMode::Paused | EmulatorMode::BreakpointHit | EmulatorMode::UnknownInstruction(..) => {
                            if !self.adjusted_cursor {
                                let target = ui.cursor_start_pos()[1] + pc as f32 * (ui.text_line_height() / 2.0);
    
                                self.adjusted_cursor = true;
                                ui.set_scroll_from_pos_y(target);
                            }
                        }
                        _ => {}
                    }
                }
            }
            else {
                self.adjusted_cursor = false;
            }
        });
    }
}
