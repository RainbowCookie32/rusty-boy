use std::borrow::Cow;

use imgui::{Textures, TextureId};
use imgui_glium_renderer::Texture;

use glium::{Display, Texture2d};
use glium::texture::{ClientFormat, RawImage2d};
use glium::uniforms::{MagnifySamplerFilter, MinifySamplerFilter, SamplerBehavior};

const BASE_PALETTE: [u8; 4] = [255, 192, 96, 0];

#[derive(Clone)]
pub struct Palette {
    colors: Vec<u8>
}

impl Palette {
    pub fn new() -> Palette {
        let colors = vec![255, 192, 96, 0];

        Palette {
            colors
        }
    }

    pub fn update(&mut self, value: u8) {
        let value = value as usize;

        self.colors[0] = BASE_PALETTE[value & 3];
        self.colors[1] = BASE_PALETTE[(value >> 2) & 3];
        self.colors[2] = BASE_PALETTE[(value >> 4) & 3];
        self.colors[3] = BASE_PALETTE[(value >> 6) & 3];
    }

    pub fn get_color(&self, idx: u8) -> u8 {
        self.colors[idx as usize]
    }
}

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

pub fn create_tile(data: &[u8], palette: &Palette) -> Vec<u8> {
    let mut tile = Vec::with_capacity(64);
    let chunks = data.chunks_exact(2);

    for tile_line in chunks {
        for bit in (0..8).rev() {
            let color_idx = ((tile_line[0] >> bit) & 1) | (((tile_line[1] >> bit) & 1) << 1);
            let pixel_color = palette.get_color(color_idx);

            tile.push(pixel_color);
        }
    }

    tile
}
