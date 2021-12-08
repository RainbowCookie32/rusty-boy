use std::sync::{Arc, RwLock};

use imgui::*;

use crate::gameboy::Gameboy;
use crate::gameboy::memory::cart::CartHeader;

pub struct CartWindow {
    header: Arc<CartHeader>
}

impl CartWindow {
    pub fn init(gb: Arc<RwLock<Gameboy>>) -> CartWindow {
        let header = gb.read().unwrap().ui_get_header();
        
        CartWindow {
            header
        }
    }

    pub fn draw(&self, ui: &Ui, opened: &mut bool) {
        if !*opened {
            return;
        }

        Window::new("Cartridge Info").size([290.0, 105.0], Condition::Always).opened(opened).resizable(false).build(ui, || {
            ui.text(format!("Cartridge Title: {}", self.header.title()));
            ui.text(format!("Cartridge Controller: {}", self.header.cart_type()));
            
            ui.separator();

            ui.text(format!("ROM Size: {} ({} banks)", self.header.rom_size(), self.header.rom_banks_count()));
            ui.text(format!("RAM Size: {} ({} banks)", self.header.ram_size(), self.header.ram_banks_count()));
        });
    }
}
