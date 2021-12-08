use std::sync::{Arc, RwLock};

use imgui::*;
use imgui_glium_renderer::Texture;

use glium::Display;

use crate::gameboy::Gameboy;
use crate::gameboy::memory::GameboyMemory;

use crate::gameboy::ppu::utils;
use crate::gameboy::ppu::utils::GameboyTexture;

pub struct VramViewerWindow {
    gb_mem: Arc<RwLock<GameboyMemory>>,
    
    tiles: Vec<GameboyTexture>,
    backgrounds: Vec<GameboyTexture>,
    backgrounds_data: Arc<RwLock<Vec<Vec<u8>>>>
}

impl VramViewerWindow {
    pub fn init(gb: Arc<RwLock<Gameboy>>) -> VramViewerWindow {
        let gb_mem = gb.read().unwrap().ui_get_memory();

        let tiles = vec![GameboyTexture::new(8, 8); 256];
        let backgrounds = vec![GameboyTexture::new(256, 256); 2];
        let backgrounds_data = gb.read().unwrap().ui_get_backgrounds_data();

        VramViewerWindow {
            gb_mem,

            tiles,
            backgrounds,
            backgrounds_data
        }
    }

    pub fn draw(&mut self, ui: &Ui, opened: &mut bool, display: &Display, textures: &mut Textures<Texture>) {
        if !*opened {
            return;
        }
        
        ui.window("VRAM Viewer").size([256.0, 256.0], Condition::FirstUseEver).opened(opened).build(|| {
            TabBar::new("Viewer Tabs").build(ui, || {
                TabItem::new("Background 0").build(ui, || {
                    let window_size = ui.content_region_avail();

                    let x_scale = window_size[0] / 256.0;
                    let y_scale = window_size[1] / 256.0;

                    if let Ok(backgrounds) = self.backgrounds_data.try_read() {
                        let background = &backgrounds[0];
                        let mut data: Vec<u8> = Vec::with_capacity((256 * 256) * 3);
        
                        for b in background {                        
                            data.push(*b);
                            data.push(*b);
                            data.push(*b);
                        }
        
                        self.backgrounds[0].update_texture(data, display, textures);
                    }

                    if let Some(id) = self.backgrounds[0].id().as_ref() {
                        Image::new(*id, [256.0 * x_scale, 256.0 * y_scale]).build(ui);
                    }
                });

                TabItem::new("Background 1").build(ui, || {
                    let window_size = ui.content_region_avail();

                    let x_scale = window_size[0] / 256.0;
                    let y_scale = window_size[1] / 256.0;
                    
                    if let Ok(backgrounds) = self.backgrounds_data.try_read() {
                        let background = &backgrounds[1];
                        let mut data: Vec<u8> = Vec::with_capacity((256 * 256) * 3);
        
                        for b in background {                        
                            data.push(*b);
                            data.push(*b);
                            data.push(*b);
                        }
        
                        self.backgrounds[1].update_texture(data, display, textures);
                    }

                    if let Some(id) = self.backgrounds[1].id().as_ref() {
                        Image::new(*id, [256.0 * x_scale, 256.0 * y_scale]).build(ui);
                    }
                });

                TabItem::new("Tiles").build(ui, || {
                    let mut palette = utils::Palette::new();
                    let mut data = Vec::new();

                    if let Ok(lock) = self.gb_mem.read() {
                        palette.update(lock.read(0xFF47));

                        for address in 0x8000..0x87FF {
                            data.push(lock.read(address));
                        }
    
                        for address in 0x8800..0x8FFF {
                            data.push(lock.read(address));
                        }
                    }

                    for (idx, tile_data) in data.chunks_exact(16).enumerate() {
                        let tile = utils::create_tile(tile_data, &palette);
                        let mut data = Vec::with_capacity(64 * 3);

                        for byte in tile {
                            data.push(byte);
                            data.push(byte);
                            data.push(byte);
                        }

                        self.tiles[idx].update_texture(data, display, textures);
                    }

                    let mut tile_addr = 0x8000;
                    let mut same_line_offset = 0.0;

                    for (idx, tex) in self.tiles.iter().enumerate() {
                        if let Some(id) = tex.id().as_ref() {
                            Image::new(*id, [8.0 * 3.0, 8.0 * 3.0]).build(ui);

                            if ui.is_item_hovered() {
                                ui.tooltip(|| {
                                    ui.text(format!("Tile ID: ${:02X}", idx));
                                    ui.text(format!("Tile Address: ${:04X}", tile_addr));
                                });
                            }

                            tile_addr += 16;
                        }

                        if tile_addr == 0x8800 {
                            ui.spacing();
                            same_line_offset = 0.0;
                        }
                        else if same_line_offset > ui.content_region_avail()[0] {
                            same_line_offset = 0.0;
                        }
                        else {
                            same_line_offset += (8.0 * 3.0) + 3.5;
                            ui.same_line_with_pos(same_line_offset);
                        }
                    }
                });
            });
        });
    }
}
