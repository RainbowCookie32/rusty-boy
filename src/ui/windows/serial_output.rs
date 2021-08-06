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
        Window::new(im_str!("Serial Output")).build(ui, || {
            if let Ok(lock) = self.gb_serial.read() {
                let mut output = String::new();

                ListBox::new(im_str!("")).size([420.0, 110.0]).build(ui, || {
                    if self.serial_show_lines_as_hex {
                        for b in lock.iter() {
                            if *b == 0x0A {
                                output.push('\n');
                            }
                            else {
                                output.push_str(&format!("${:02X} ", *b));
                            }
                        }
                    }
                    else {
                        output = String::from_utf8_lossy(&lock).to_string();
                    }

                    for line in output.lines() {
                        Selectable::new(&ImString::from(line.to_string())).build(ui);
                    }
                });

                ui.checkbox(im_str!("Show lines as hex"), &mut self.serial_show_lines_as_hex);
            }
        });
    }
}
