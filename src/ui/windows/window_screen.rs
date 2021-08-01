use std::borrow::Cow;
use std::sync::{Arc, RwLock};

use imgui::*;
use imgui_glium_renderer::Texture;

use glium::{Display, Texture2d};
use glium::texture::{ClientFormat, RawImage2d};
use glium::uniforms::{MagnifySamplerFilter, MinifySamplerFilter, SamplerBehavior};

const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;

pub struct ScreenWindow {
    show_screen: bool,
    show_background_0: bool,
    show_background_1: bool,

    screen_scale: i32,
    background_0_scale: i32,
    background_1_scale: i32,

    screen_data: Arc<RwLock<Vec<u8>>>,
    backgrounds_data: Arc<RwLock<Vec<Vec<u8>>>>,

    bg0_texture_id: Option<TextureId>,
    bg1_texture_id: Option<TextureId>,
    screen_texture_id: Option<TextureId>
}

impl ScreenWindow {
    pub fn init(screen_data: Arc<RwLock<Vec<u8>>>, backgrounds_data: Arc<RwLock<Vec<Vec<u8>>>>) -> ScreenWindow {
        ScreenWindow {
            show_screen: true,
            show_background_0: false,
            show_background_1: false,

            screen_scale: 1,
            background_0_scale: 1,
            background_1_scale: 1,

            screen_data,
            backgrounds_data,

            bg0_texture_id: None,
            bg1_texture_id: None,
            screen_texture_id: None
        }
    }

    pub fn draw(&mut self, ui: &Ui, display: &Display, textures: &mut Textures<Texture>) {
        Window::new(im_str!("Video Settings")).build(ui, || {
            ui.checkbox(im_str!("Show screen"), &mut self.show_screen);
            
            ui.checkbox(im_str!("Show Background ($9800-$9BFF)"), &mut self.show_background_0);
            ui.same_line(0.0);
            ui.checkbox(im_str!("Show Background ($9C00-$9FFF)"), &mut self.show_background_1);

            ui.input_int(im_str!("Screen Scale"), &mut self.screen_scale).build();
            ui.input_int(im_str!("Background 0 Scale"), &mut self.background_0_scale).build();
            ui.input_int(im_str!("Background 1 Scale"), &mut self.background_1_scale).build();
        });

        if self.show_screen {
            Window::new(im_str!("Screen")).build(ui, || {
                if let Ok(lock) = self.screen_data.try_read() {
                    let mut data: Vec<u8> = Vec::with_capacity(SCREEN_WIDTH * SCREEN_HEIGHT);
    
                    for b in lock.iter() {                        
                        data.push(*b);
                        data.push(*b);
                        data.push(*b);
                    }
    
                    let image = RawImage2d {
                        data: Cow::Owned(data),
                        width: SCREEN_WIDTH as u32,
                        height: SCREEN_HEIGHT as u32,
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
        
                    if let Some(id) = self.screen_texture_id.take() {
                        textures.remove(id);
                    }
    
                    self.screen_texture_id = Some(textures.insert(texture));
                }

                if let Some(id) = self.screen_texture_id.as_ref() {
                    let w = SCREEN_HEIGHT * self.screen_scale as usize;
                    let h = SCREEN_HEIGHT * self.screen_scale as usize;

                    Image::new(*id, [w as f32, h as f32]).build(ui);
                }
            });
        }

        if self.show_background_0 {
            if let Ok(backgrounds) = self.backgrounds_data.try_read() {
                let background = &backgrounds[0];
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

                if let Some(id) = self.bg0_texture_id.take() {
                    textures.remove(id);
                }

                self.bg0_texture_id = Some(textures.insert(texture));
            }

            Window::new(im_str!("Background ($9800-$9BFF)")).build(ui, || {
                if let Some(id) = self.bg0_texture_id.as_ref() {
                    let size = 256 * self.background_0_scale;
                    Image::new(*id, [size as f32, size as f32]).build(ui);
                }
            });
        }

        if self.show_background_1 {
            if let Ok(backgrounds) = self.backgrounds_data.try_read() {
                let background = &backgrounds[1];
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

                if let Some(id) = self.bg1_texture_id.take() {
                    textures.remove(id);
                }

                self.bg1_texture_id = Some(textures.insert(texture));
            }

            Window::new(im_str!("Background ($9C00-$9FFF)")).build(ui, || {
                if let Some(id) = self.bg1_texture_id.as_ref() {
                    let size = 256 * self.background_1_scale;
                    Image::new(*id, [size as f32, size as f32]).build(ui);
                }
            });
        }
    }
}
