use std::sync::Arc;

use imgui::*;

use crate::gameboy::memory::GameboyMemory;

pub struct MemoryWindow {
    gb_mem: Arc<GameboyMemory>,

    editing_byte: bool,
    target_byte_address: u16,
    target_byte_new_value: ImString
}

impl MemoryWindow {
    pub fn init(gb_mem: Arc<GameboyMemory>) -> MemoryWindow {
        MemoryWindow {
            gb_mem,

            editing_byte: false,
            target_byte_address: 0,
            target_byte_new_value: ImString::new("")
        }
    }

    pub fn draw(&mut self, ui: &Ui) {
        Window::new(im_str!("Memory Viewer")).build(ui, || {
            let style_padding = ui.push_style_var(StyleVar::FramePadding([0.0, 0.0]));
            let style_spacing = ui.push_style_var(StyleVar::ItemSpacing([5.0, 1.0]));

            let size = ui.calc_text_size(im_str!("FF"), false, 0.0);
            let mut clipper = ListClipper::new(0xFFFF / 8).items_height(ui.text_line_height() / 2.0).begin(ui);
            clipper.step();

            for line in clipper.display_start()..clipper.display_end() {
                let mut values = Vec::with_capacity(8);
                let mut current_addr = line as u16 * 8;

                for _ in 0..8 {
                    values.push(self.gb_mem.read(current_addr));
                    current_addr += 1;
                }

                ui.text(format!("{:04X} |", current_addr - 8));

                ui.same_line(0.0);

                for (idx, value) in values.iter().enumerate() {
                    let token = ui.push_id(&format!("value{}", idx));
                    let value_address = (current_addr - 8) + idx as u16;

                    if self.editing_byte && self.target_byte_address == value_address {
                        let mut flags = ImGuiInputTextFlags::empty();

                        flags.set(ImGuiInputTextFlags::CharsHexadecimal, true);
                        flags.set(ImGuiInputTextFlags::EnterReturnsTrue, true);
                        flags.set(ImGuiInputTextFlags::AutoSelectAll, true);
                        flags.set(ImGuiInputTextFlags::NoHorizontalScroll, true);
                        flags.set(ImGuiInputTextFlags::AlwaysInsertMode, true);
                        
                        ui.set_next_item_width(size[0]);

                        if ui.input_text(im_str!("##data"), &mut self.target_byte_new_value).flags(flags).resize_buffer(true).build() {
                            if let Ok(value) = u8::from_str_radix(&self.target_byte_new_value.to_string(), 16) {
                                self.gb_mem.dbg_write(value_address, value);
                            }

                            self.editing_byte = false;
                            self.target_byte_address = 0;
                            self.target_byte_new_value = ImString::new("");
                        }
                    }
                    else if Selectable::new(&ImString::from(format!("{:02X}", value))).allow_double_click(true).size(size).build(ui) {
                        self.editing_byte = true;
                        self.target_byte_address = (current_addr - 8) + idx as u16;
                        self.target_byte_new_value = ImString::from(format!("{:02X}", value));
                    }

                    token.pop(ui);
                    ui.same_line(0.0);
                }

                ui.text(" | ");
                ui.same_line(0.0);

                for (idx, value) in values.iter().enumerate() {
                    let value = if *value == 0 {'.'} else {*value as char};
                    let size = ui.calc_text_size(im_str!("F"), false, 0.0);
                    if Selectable::new(&ImString::from(format!("{}", value))).allow_double_click(true).size(size).build(ui) {

                    }

                    if idx != values.len() - 1 {
                        ui.same_line(0.0);
                    }
                }
            }

            clipper.end();

            style_padding.pop(ui);
            style_spacing.pop(ui);
        });
    }
}
