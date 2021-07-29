use std::sync::{Arc, RwLock};

use imgui::*;

use crate::gameboy::Gameboy;
use crate::gameboy::memory::GameboyMemory;

pub struct CartWindow {
    gb: Arc<RwLock<Gameboy>>,
    gb_mem: Arc<GameboyMemory>
}

impl CartWindow {
    pub fn init(gb: Arc<RwLock<Gameboy>>, gb_mem: Arc<GameboyMemory>) -> CartWindow {
        CartWindow {
            gb,
            gb_mem
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
                ui.text(format!("Selected ROM Bank: {}", self.gb_mem.cartridge().get_selected_rom_bank()));

                ui.separator();

                ui.text(format!("RAM Size: {} ({} banks)", header.ram_size(), header.ram_banks_count()));
                ui.text(format!("RAM Access Enabled: {}", self.gb_mem.cartridge().is_ram_enabled()));
                ui.text(format!("Selected RAM Bank: {}", self.gb_mem.cartridge().get_selected_rom_bank()));
            }
        });
    }
}
