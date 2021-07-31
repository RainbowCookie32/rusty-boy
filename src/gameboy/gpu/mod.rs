use std::sync::{Arc, RwLock};

use crate::gameboy::memory::GameboyMemory;

const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;
const COLORS: [u8; 4] = [255, 192, 96, 0];

enum Mode {
    Vblank,
    Hblank,
    OamScan,
    LcdTransfer
}

#[derive(Clone)]
struct Palette {
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

        self.colors[0] = COLORS[value & 3];
        self.colors[1] = COLORS[(value >> 2) & 3];
        self.colors[2] = COLORS[(value >> 4) & 3];
        self.colors[3] = COLORS[(value >> 6) & 3];
    }

    pub fn get_color(&self, idx: u8) -> u8 {
        self.colors[idx as usize]
    }
}

pub struct GameboyGPU {
    ly: u8,
    lyc: u8,
    lcdc: u8,
    stat: u8,

    bg_palette: Palette,
    obj_palettes: Vec<Palette>,

    cycles: usize,
    
    screen: Arc<RwLock<Vec<u8>>>,
    backgrounds: Arc<RwLock<Vec<Vec<u8>>>>,

    gb_mem: Arc<GameboyMemory>
}

impl GameboyGPU {
    pub fn init(gb_mem: Arc<GameboyMemory>) -> GameboyGPU {
        GameboyGPU {
            ly: 0,
            lyc: 0,
            lcdc: 0,
            stat: 0,

            bg_palette: Palette::new(),
            obj_palettes: vec![Palette::new(); 2],

            cycles: 0,

            screen: Arc::new(RwLock::new(vec![0; SCREEN_WIDTH * SCREEN_HEIGHT])),
            backgrounds: Arc::new(RwLock::new(vec![vec![255; 256 * 256]; 2])),

            gb_mem
        }
    }

    pub fn gpu_cycle(&mut self, cycles: &mut usize) {
        self.ly = self.gb_mem.read(0xFF44);
        self.lyc = self.gb_mem.read(0xFF45);
        self.lcdc = self.gb_mem.read(0xFF40);
        self.stat = self.gb_mem.read(0xFF41);

        self.bg_palette.update(self.gb_mem.read(0xFF47));
        self.obj_palettes[0].update(self.gb_mem.read(0xFF48));
        self.obj_palettes[1].update(self.gb_mem.read(0xFF49));

        if self.lcdc & 0x80 == 0 {
            return;
        }

        self.cycles += *cycles - self.cycles;

        let current_mode = self.stat & 3;

        // Mode 2 - OAM scan.
        if self.cycles >= 80 && current_mode == 2 {
            self.set_mode(Mode::LcdTransfer);
        }
        // Mode 3 - Access OAM and VRAM to generate the picture.
        else if self.cycles >= 172 && current_mode == 3 {
            self.draw_screen_line();
            self.set_mode(Mode::Hblank);
        }
        // Mode 0 - H-Blank.
        else if self.cycles >= 204 && current_mode == 0 {
            self.ly += 1;

            if self.ly < 144 {
                self.set_mode(Mode::OamScan);
            }
            else {
                self.set_mode(Mode::Vblank);
            }

            if self.ly == self.lyc {
                // TODO: Request interrupt.
            }

            self.gb_mem.write(0xFF44, self.ly);
        }
        // Mode 1 - V-Blank.
        else if self.cycles >= 456 && current_mode == 1 {
            self.ly += 1;

            if self.ly > 153 {
                self.ly = 0;
                *cycles = 0;
                
                self.cycles = 0;
                self.set_mode(Mode::OamScan);
            }

            if self.ly == self.lyc {
                // TODO: Request interrupt.
            }

            self.draw_backgrounds();
            self.gb_mem.write(0xFF44, self.ly);
        }
    }

    pub fn get_screen_data(&self) -> Arc<RwLock<Vec<u8>>> {
        self.screen.clone()
    }

    pub fn get_backgrounds_data(&self) -> Arc<RwLock<Vec<Vec<u8>>>> {
        self.backgrounds.clone()
    }

    fn set_mode(&mut self, mode: Mode) {
        self.stat &= 0xFC;

        match mode {
            Mode::Vblank => self.stat |= 1,
            Mode::OamScan => self.stat |= 2,
            Mode::LcdTransfer => self.stat |= 3,
            _ => {}
        }

        self.gb_mem.write(0xFF41, self.stat);

        // TODO: Request an interrupt.
    }

    // Draw a screen line using the data in self.backgrounds.
    fn draw_screen_line(&mut self) {

    }

    fn draw_backgrounds(&mut self) {
        let (tiles_start, tiles_end) = if self.lcdc & 0x10 == 0 {(0x8800, 0x9800)} else {(0x8000, 0x9000)};

        if let Ok(mut lock) = self.backgrounds.write() {
            for (bg_idx, background) in lock.iter_mut().enumerate() {
                let (map_start, map_end) = if bg_idx == 0 {(0x9800, 0x9C00)} else {(0x9C00, 0xA000)};

                let tiles = {
                    let mut res = Vec::new();
                    let mut data = Vec::new();

                    for address in tiles_start..tiles_end {
                        data.push(self.gb_mem.read(address));
                    }

                    data.chunks_exact(16).for_each(|t| res.push(t.to_owned()));
                    res
                };

                let map_data = {
                    let mut res = Vec::with_capacity(1024);

                    for address in map_start..map_end {
                        res.push(self.gb_mem.read(address));
                    }

                    res
                };

                for (bg_line_idx, bg_line_data) in map_data.chunks_exact(32).enumerate() {
                    let mut x_offset = 0;
                    let y_offset = bg_line_idx * 8;

                    for tile_idx in bg_line_data {
                        let tile = &tiles[*tile_idx as usize];
                        let tile_data = tile.chunks_exact(2);

                        for (tile_y, line) in tile_data.enumerate() {
                            let mut idx = x_offset + (256 * (y_offset + tile_y));

                            for bit in (0..8).rev() {
                                let color_idx = ((line[1] >> bit) & 1) | (((line[0] >> bit) & 1) << 1);
                                let pixel_color = self.bg_palette.get_color(color_idx);

                                background[idx] = pixel_color;

                                idx += 1;
                            }
                        }

                        x_offset += 8;
                    }
                }
            }
        }
    }
}
