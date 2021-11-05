use std::sync::{Arc, RwLock};

use imgui::*;

use crate::gameboy::Gameboy;

pub struct SerialWindow {
    gb_serial: Arc<RwLock<Vec<u8>>>,
    serial_show_lines_as_hex: bool
}

impl SerialWindow {
    pub fn init(gb: Arc<RwLock<Gameboy>>) -> SerialWindow {
        let gb_serial = gb.read().unwrap().ui_get_serial_output();
        
        SerialWindow {
            gb_serial,
            serial_show_lines_as_hex: false
        }
    }

    pub fn draw(&mut self, ui: &Ui) {
        Window::new("Serial Output").size([475.0, 170.0], Condition::FirstUseEver).build(ui, || {
            if let Ok(lock) = self.gb_serial.read() {
                let mut output = String::new();

                ListBox::new("").size([420.0, 110.0]).build(ui, || {
                    if self.serial_show_lines_as_hex {
                        for b in lock.iter() {
                            if *b == 0x0A {
                                output.push('\n');
                            }
                            else {
                                output.push(*b as char);
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

                ui.checkbox("Show lines as hex", &mut self.serial_show_lines_as_hex);
            }
        });
    }
}
