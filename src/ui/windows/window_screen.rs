use std::borrow::Cow;
use std::sync::{Arc, RwLock};

use imgui::*;
use imgui_glium_renderer::Texture;

use glium::{Display, Texture2d};
use glium::texture::{ClientFormat, RawImage2d};
use glium::uniforms::{MagnifySamplerFilter, MinifySamplerFilter, SamplerBehavior};

use crate::gameboy::JoypadHandler;

const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;

pub struct GameboyTexture {
    id: Option<TextureId>,

    width: u32,
    height: u32
}

impl GameboyTexture {
    pub fn new(width: u32, height: u32) -> GameboyTexture {
        GameboyTexture {
            id: None,

            width,
            height
        }
    }

    pub fn update_texture(&mut self, data: Vec<u8>, display: &Display, textures: &mut Textures<Texture>) {
        let image = RawImage2d {
            data: Cow::Owned(data),
            width: self.width,
            height: self.height,
            format: ClientFormat::U8U8U8
        };

        if let Ok(gl_texture) = Texture2d::new(display, image) {
            let texture = Texture {
                texture: std::rc::Rc::new(gl_texture),
                sampler: SamplerBehavior {
                    magnify_filter: MagnifySamplerFilter::Nearest,
                    minify_filter: MinifySamplerFilter::Nearest,
                    ..Default::default()
                }
            };
        
            if let Some(id) = self.id.take() {
                textures.remove(id);
            }
        
            self.id = Some(textures.insert(texture));
        }
        else {
            println!("Error updating texture.");
        }
    }
}

pub struct ScreenWindow {
    show_screen: bool,
    show_background_0: bool,
    show_background_1: bool,

    screen_scale: i32,
    background_0_scale: i32,
    background_1_scale: i32,

    screen: GameboyTexture,
    backgrounds: Vec<GameboyTexture>,

    gb_joy: Arc<RwLock<JoypadHandler>>,
    screen_data: Arc<RwLock<Vec<u8>>>,
    backgrounds_data: Arc<RwLock<Vec<Vec<u8>>>>
}

impl ScreenWindow {
    pub fn init(gb_joy: Arc<RwLock<JoypadHandler>>, screen_data: Arc<RwLock<Vec<u8>>>, backgrounds_data: Arc<RwLock<Vec<Vec<u8>>>>) -> ScreenWindow {
        ScreenWindow {
            show_screen: true,
            show_background_0: false,
            show_background_1: false,

            screen_scale: 2,
            background_0_scale: 1,
            background_1_scale: 1,

            screen: GameboyTexture::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32),
            backgrounds: vec![GameboyTexture::new(256, 256), GameboyTexture::new(256, 256)],

            gb_joy,
            screen_data,
            backgrounds_data
        }
    }

    pub fn draw(&mut self, ui: &Ui, display: &Display, textures: &mut Textures<Texture>) {
        Window::new(im_str!("Video Settings")).build(ui, || {
            ui.checkbox(im_str!("Show Screen"), &mut self.show_screen);
            
            ui.checkbox(im_str!("Show BG0 ($9800)"), &mut self.show_background_0);
            ui.same_line(0.0);
            ui.checkbox(im_str!("Show BG1 ($9C00)"), &mut self.show_background_1);

            ui.input_int(im_str!("Screen Scale"), &mut self.screen_scale).build();
            ui.input_int(im_str!("BG0 Scale"), &mut self.background_0_scale).build();
            ui.input_int(im_str!("BG1 Scale"), &mut self.background_1_scale).build();
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
    
                    self.screen.update_texture(data, display, textures);
                }

                if let Some(id) = self.screen.id.as_ref() {
                    let w = SCREEN_HEIGHT * self.screen_scale as usize;
                    let h = SCREEN_HEIGHT * self.screen_scale as usize;

                    Image::new(*id, [w as f32, h as f32]).build(ui);
                }

                if ui.is_window_focused() {
                    if let Ok(mut lock) = self.gb_joy.write() {
                        lock.set_down_state(ui.is_key_down(Key::DownArrow));
                        lock.set_up_state(ui.is_key_down(Key::UpArrow));
                        lock.set_left_state(ui.is_key_down(Key::LeftArrow));
                        lock.set_right_state(ui.is_key_down(Key::RightArrow));

                        lock.set_start_state(ui.is_key_down(Key::Enter));
                        lock.set_select_state(ui.is_key_down(Key::Backspace));
                        lock.set_b_state(ui.is_key_down(Key::X));
                        lock.set_a_state(ui.is_key_down(Key::Z));
                    }
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

                self.backgrounds[0].update_texture(data, display, textures);
            }

            Window::new(im_str!("Background ($9800-$9BFF)")).build(ui, || {
                if let Some(id) = self.backgrounds[0].id.as_ref() {
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

                self.backgrounds[1].update_texture(data, display, textures);
            }

            Window::new(im_str!("Background ($9C00-$9FFF)")).build(ui, || {
                if let Some(id) = self.backgrounds[1].id.as_ref() {
                    let size = 256 * self.background_1_scale;
                    Image::new(*id, [size as f32, size as f32]).build(ui);
                }
            });
        }
    }
}
