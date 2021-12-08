use std::sync::{Arc, RwLock};

use imgui::*;

use crate::gameboy::memory::GameboyMemory;

pub struct MemoryWindow {
    gb_mem: Arc<RwLock<GameboyMemory>>,

    editing_byte: bool,
    target_byte_address: u16,
    target_byte_new_value: String
}

impl MemoryWindow {
    pub fn init(gb_mem: Arc<RwLock<GameboyMemory>>) -> MemoryWindow {
        MemoryWindow {
            gb_mem,

            editing_byte: false,
            target_byte_address: 0,
            target_byte_new_value: String::new()
        }
    }

    pub fn draw(&mut self, ui: &Ui, opened: &mut bool) {
        if !*opened {
            return;
        }

        Window::new("Memory Viewer").size([350.0, 170.0], Condition::FirstUseEver).opened(opened).build(ui, || {
            let style_padding = ui.push_style_var(StyleVar::FramePadding([0.0, 0.0]));
            let style_spacing = ui.push_style_var(StyleVar::ItemSpacing([5.0, 1.0]));

            let size = ui.calc_text_size("FF");
            let mut clipper = ListClipper::new(0xFFFF / 8).items_height(ui.text_line_height() / 2.0).begin(ui);
            clipper.step();

            for line in clipper.display_start()..clipper.display_end() {
                let mut values = Vec::with_capacity(8);
                let mut current_addr = line as u16 * 8;

                for _ in 0..8 {
                    values.push(
                        if let Ok(lock) = self.gb_mem.read() {
                            lock.read(current_addr)
                        }
                        else {
                            0
                        }
                    );

                    current_addr += 1;
                }

                ui.text(format!("{:04X} |", current_addr - 8));

                ui.same_line();

                for (idx, value) in values.iter().enumerate() {
                    let token = ui.push_id(&format!("value{}", idx));
                    let value_address = (current_addr - 8) + idx as u16;

                    if self.editing_byte && self.target_byte_address == value_address {
                        let mut flags = InputTextFlags::empty();

                        flags.set(InputTextFlags::CHARS_HEXADECIMAL, true);
                        flags.set(InputTextFlags::ENTER_RETURNS_TRUE, true);
                        flags.set(InputTextFlags::AUTO_SELECT_ALL, true);
                        flags.set(InputTextFlags::NO_HORIZONTAL_SCROLL, true);
                        flags.set(InputTextFlags::ALWAYS_OVERWRITE, true);
                        
                        ui.set_next_item_width(size[0]);

                        if ui.input_text("##data", &mut self.target_byte_new_value).flags(flags).build() {
                            if let Ok(value) = u8::from_str_radix(&self.target_byte_new_value.to_string(), 16) {
                                if let Ok(mut lock) = self.gb_mem.write() {
                                    lock.dbg_write(value_address, value);
                                }
                                
                            }

                            self.editing_byte = false;
                            self.target_byte_address = 0;
                            self.target_byte_new_value = String::new();
                        }
                    }
                    else if Selectable::new(&ImString::from(format!("{:02X}", value))).allow_double_click(true).size(size).build(ui) {
                        self.editing_byte = true;
                        self.target_byte_address = (current_addr - 8) + idx as u16;
                        self.target_byte_new_value = format!("{:02X}", value);
                    }

                    token.pop();
                    ui.same_line();
                }

                ui.text(" | ");
                ui.same_line();

                for (idx, value) in values.iter().enumerate() {
                    let value = if *value == 0 {'.'} else {*value as char};
                    let size = ui.calc_text_size("F");
                    if Selectable::new(&ImString::from(format!("{}", value))).allow_double_click(true).size(size).build(ui) {

                    }

                    if idx != values.len() - 1 {
                        ui.same_line();
                    }
                }
            }

            clipper.end();

            style_padding.pop();
            style_spacing.pop();
        });
    }
}
