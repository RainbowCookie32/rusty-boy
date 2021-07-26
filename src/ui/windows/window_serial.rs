use std::sync::{Arc, RwLock};

use imgui::*;

pub struct SerialWindow {
    gb_serial: Arc<RwLock<Vec<u8>>>,
    serial_show_lines_as_hex: bool
}

impl SerialWindow {
    pub fn init(gb_serial: Arc<RwLock<Vec<u8>>>) -> SerialWindow {
        SerialWindow {
            gb_serial,
            serial_show_lines_as_hex: false
        }
    }

    pub fn draw(&mut self, ui: &Ui) {
        Window::new(im_str!("Serial Output")).build(&ui, || {
            if let Ok(lock) = self.gb_serial.read() {
                let mut output = String::new();

                for b in lock.iter() {
                    let c = *b as char;
                    output.push(c);
                }

                ListBox::new(im_str!("")).size([320.0, 110.0]).build(&ui, || {
                    for line in output.lines() {
                        if self.serial_show_lines_as_hex {
                            let mut text = String::new();

                            for c in line.chars() {
                                text.push_str(&format!("${:02X} ", c as u8));
                            }

                            Selectable::new(&ImString::from(text)).build(&ui);
                        }
                        else {
                            Selectable::new(&ImString::from(line.to_string())).build(&ui);
                        }
                    }
                });

                ui.checkbox(im_str!("Show lines as hex"), &mut self.serial_show_lines_as_hex);
            }
        });
    }
}
