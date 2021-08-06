use std::sync::Arc;

use imgui::*;

use crate::gameboy::memory::cart::CartHeader;

pub struct CartWindow {
    header: Arc<CartHeader>
}

impl CartWindow {
    pub fn init(header: Arc<CartHeader>) -> CartWindow {
        CartWindow {
            header
        }
    }

    pub fn draw(&self, ui: &Ui) {
        Window::new(im_str!("Cartridge Info")).build(ui, || {
            ui.text(format!("Cartridge Title: {}", self.header.title()));
            ui.text(format!("Cartridge Controller: {}", self.header.cart_type()));
            
            ui.separator();

            ui.text(format!("ROM Size: {} ({} banks)", self.header.rom_size(), self.header.rom_banks_count()));
            ui.text(format!("RAM Size: {} ({} banks)", self.header.ram_size(), self.header.ram_banks_count()));
        });
    }
}
