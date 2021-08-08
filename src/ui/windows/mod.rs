pub mod cart_info;
pub mod cpu_debugger;
pub mod disassembler;
pub mod file_picker;
pub mod memory_viewer;
pub mod notification;
pub mod screen;
pub mod serial_output;
pub mod settings;
pub mod vram_viewer;

use std::borrow::Cow;

use imgui::{Textures, TextureId};
use imgui_glium_renderer::Texture;

use glium::{Display, Texture2d};
use glium::texture::{ClientFormat, RawImage2d};
use glium::uniforms::{MagnifySamplerFilter, MinifySamplerFilter, SamplerBehavior};

#[derive(Clone)]
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

    pub fn id(&self) -> &Option<TextureId> {
        &self.id
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
