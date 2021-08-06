use std::sync::{Arc, RwLock};

use imgui::*;
use imgui_glium_renderer::Texture;

use glium::Display;

use crate::gameboy::Gameboy;
use crate::ui::windows::GameboyTexture;

pub struct VramViewerWindow {
    backgrounds: Vec<GameboyTexture>,
    backgrounds_data: Arc<RwLock<Vec<Vec<u8>>>>
}

impl VramViewerWindow {
    pub fn init(gb: Arc<RwLock<Gameboy>>) -> VramViewerWindow {
        let backgrounds = vec![GameboyTexture::new(256, 256); 2];
        let backgrounds_data = gb.read().unwrap().ui_get_backgrounds_data();

        VramViewerWindow {
            backgrounds,
            backgrounds_data
        }
    }

    pub fn draw(&mut self, ui: &Ui, display: &Display, textures: &mut Textures<Texture>) {
        Window::new(im_str!("VRAM Viewer")).size([256.0, 256.0], Condition::Once).build(ui, || {
            TabBar::new(im_str!("Viewer Tabs")).build(ui, || {
                TabItem::new(im_str!("Background 0")).build(ui, || {
                    let window_size = ui.content_region_avail();

                    let x_scale = window_size[0] / 256.0;
                    let y_scale = window_size[1] / 256.0;

                    if let Ok(backgrounds) = self.backgrounds_data.try_read() {
                        let background = &backgrounds[0];
                        let mut data: Vec<u8> = Vec::with_capacity(256 * 256);
        
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

                TabItem::new(im_str!("Background 1")).build(ui, || {
                    let window_size = ui.content_region_avail();

                    let x_scale = window_size[0] / 256.0;
                    let y_scale = window_size[1] / 256.0;
                    
                    if let Ok(backgrounds) = self.backgrounds_data.try_read() {
                        let background = &backgrounds[1];
                        let mut data: Vec<u8> = Vec::with_capacity(256 * 256);
        
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
            });
        });
    }
}
