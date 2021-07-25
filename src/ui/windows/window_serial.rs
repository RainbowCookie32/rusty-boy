use std::sync::{Arc, RwLock};

use imgui::*;

pub struct SerialWindow {
    gb_serial: Arc<RwLock<Vec<u8>>>
}

impl SerialWindow {
    pub fn init(gb_serial: Arc<RwLock<Vec<u8>>>) -> SerialWindow {
        SerialWindow {
            gb_serial
        }
    }

    pub fn draw(&self, ui: &Ui) {
        Window::new(im_str!("Serial Output")).build(&ui, || {
            if let Ok(lock) = self.gb_serial.read() {
                let mut output = String::new();

                for b in lock.iter() {
                    let c = *b as char;
                    output.push(c);
                }

                ListBox::new(im_str!("")).size([220.0, 70.0]).build(&ui, || {
                    for line in output.lines().rev() {
                        Selectable::new(&ImString::from(line.to_string())).build(&ui);
                    }
                });
            }
        });
    }
}
