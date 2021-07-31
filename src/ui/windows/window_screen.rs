use std::borrow::Cow;
use std::sync::{Arc, RwLock};

use imgui::*;
use imgui_glium_renderer::Texture;

use glium::{Display, Texture2d};
use glium::texture::{ClientFormat, RawImage2d};
use glium::uniforms::{MagnifySamplerFilter, MinifySamplerFilter, SamplerBehavior};

const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 140;

pub struct ScreenWindow {
    screen_data: Arc<RwLock<Vec<u8>>>,
    backgrounds_data: Arc<RwLock<Vec<Vec<u8>>>>,

    bg0_texture_id: Option<TextureId>,
    bg1_texture_id: Option<TextureId>,
    screen_texture_id: Option<TextureId>
}

impl ScreenWindow {
    pub fn init(screen_data: Arc<RwLock<Vec<u8>>>, backgrounds_data: Arc<RwLock<Vec<Vec<u8>>>>) -> ScreenWindow {
        ScreenWindow {
            screen_data,
            backgrounds_data,

            bg0_texture_id: None,
            bg1_texture_id: None,
            screen_texture_id: None
        }
    }

    pub fn draw(&mut self, ui: &Ui, display: &Display, textures: &mut Textures<Texture>) {
        Window::new(im_str!("Screen")).build(ui, || {
            if let Ok(lock) = self.backgrounds_data.try_read() {
                for (i, background) in lock.iter().enumerate() {
                    let mut data: Vec<u8> = Vec::with_capacity(256 * 256);

                    for b in background {                        
                        data.push(*b);
                        data.push(*b);
                        data.push(*b);
                    }

                    let image = RawImage2d {
                        data: Cow::Owned(data),
                        width: 256,
                        height: 256,
                        format: ClientFormat::U8U8U8
                    };
        
                    let gl_texture = Texture2d::new(display, image).unwrap();
                    let texture = Texture {
                        texture: std::rc::Rc::new(gl_texture),
                        sampler: SamplerBehavior {
                            magnify_filter: MagnifySamplerFilter::Nearest,
                            minify_filter: MinifySamplerFilter::Nearest,
                            ..Default::default()
                        }
                    };
        
                    if i == 0 {
                        if let Some(id) = self.bg0_texture_id.take() {
                            textures.remove(id);
                        }

                        self.bg0_texture_id = Some(textures.insert(texture));
                    }
                    else {
                        if let Some(id) = self.bg1_texture_id.take() {
                            textures.remove(id);
                        }

                        self.bg1_texture_id = Some(textures.insert(texture));
                    }
                }
            }

            if let Some(id) = self.screen_texture_id.as_ref() {
                ui.text("Screen Output");
                Image::new(*id, [SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32]).build(ui);
            }

            ui.columns(2, im_str!("bg_cols"), true);
            
            ui.text("Background ($9800-$9BFF)");

            if let Some(id) = self.bg0_texture_id.as_ref() {
                Image::new(*id, [256.0, 256.0]).build(ui);
            }

            ui.next_column();

            ui.text("Background ($9C00-$9FFF)");

            if let Some(id) = self.bg1_texture_id.as_ref() {
                Image::new(*id, [256.0, 256.0]).build(ui);
            }
        });
    }
}
