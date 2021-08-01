use std::sync::{Arc, RwLock};

use imgui::*;

use crate::gameboy::Gameboy;

pub struct CartWindow {
    gb: Arc<RwLock<Gameboy>>
}

impl CartWindow {
    pub fn init(gb: Arc<RwLock<Gameboy>>) -> CartWindow {
        CartWindow {
            gb
        }
    }

    pub fn draw(&self, ui: &Ui) {
        Window::new(im_str!("Cartridge Info")).build(ui, || {
            if let Ok(lock) = self.gb.read() {
                let header = lock.ui_get_header();

                ui.text(format!("Cartridge Title: {}", header.title()));
                ui.text(format!("Cartridge Controller: {}", header.cart_type()));
            
                ui.separator();

                ui.text(format!("ROM Size: {} ({} banks)", header.rom_size(), header.rom_banks_count()));
                ui.text(format!("RAM Size: {} ({} banks)", header.ram_size(), header.ram_banks_count()));
            }
        });
    }
}
