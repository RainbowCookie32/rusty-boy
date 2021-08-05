use std::time;
use std::sync::{Arc, RwLock};

use crate::gameboy::memory::GameboyMemory;

const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;
const COLORS: [u8; 4] = [255, 192, 96, 0];

const LYC_BIT: u8 = 0x04;
const HBLANK_INT_BIT: u8 = 0x08;
const VBLANK_INT_BIT: u8 = 0x10;
const OAM_INT_BIT: u8 = 0x20;
const LYC_INT_BIT: u8 = 0x40;

enum Mode {
    Vblank,
    Hblank,
    OamScan,
    LcdTransfer
}

enum Interrupt {
    Coincidence,
    ModeSwitch(Mode)
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
    scy: u8,
    scx: u8,
    lcdc: u8,
    stat: u8,

    wy: u8,
    wx: u8,

    bg_palette: Palette,
    obj_palettes: Vec<Palette>,

    cycles: usize,
    
    screen: Arc<RwLock<Vec<u8>>>,
    backgrounds: Arc<RwLock<Vec<Vec<u8>>>>,

    gb_mem: Arc<GameboyMemory>,
    frame_time: time::Instant,
}

impl GameboyGPU {
    pub fn init(gb_mem: Arc<GameboyMemory>) -> GameboyGPU {
        GameboyGPU {
            ly: 0,
            lyc: 0,
            scy: 0,
            scx: 0,
            lcdc: 0,
            stat: 0,

            wy: 0,
            wx: 0,

            bg_palette: Palette::new(),
            obj_palettes: vec![Palette::new(); 2],

            cycles: 0,

            screen: Arc::new(RwLock::new(vec![255; SCREEN_WIDTH * SCREEN_HEIGHT])),
            backgrounds: Arc::new(RwLock::new(vec![vec![255; 256 * 256]; 2])),

            gb_mem,
            frame_time: time::Instant::now()
        }
    }

    pub fn gpu_cycle(&mut self, cycles: &mut usize) {
        self.ly = self.gb_mem.read(0xFF44);
        self.lyc = self.gb_mem.read(0xFF45);
        self.scy = self.gb_mem.read(0xFF42);
        self.scx = self.gb_mem.read(0xFF43);
        self.lcdc = self.gb_mem.read(0xFF40);
        self.stat = self.gb_mem.read(0xFF41);

        self.wy = self.gb_mem.read(0xFF4A);
        self.wx = self.gb_mem.read(0xFF4B);

        self.bg_palette.update(self.gb_mem.read(0xFF47));
        self.obj_palettes[0].update(self.gb_mem.read(0xFF48));
        self.obj_palettes[1].update(self.gb_mem.read(0xFF49));

        if self.lcdc & 0x80 == 0 {
            self.frame_time = time::Instant::now();
            return;
        }

        self.cycles += *cycles - self.cycles;

        let current_mode = self.stat & 3;

        // Mode 2 - OAM scan.
        if self.cycles >= 80 && current_mode == 2 {
            *cycles = 0;
            self.cycles = 0;
            self.set_mode(Mode::LcdTransfer);
        }
        // Mode 3 - Access OAM and VRAM to generate the picture.
        else if self.cycles >= 172 && current_mode == 3 {
            *cycles = 0;
            
            self.draw_screen_line();

            self.cycles = 0;
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
                self.stat |= LYC_BIT;
                self.gb_mem.write(0xFF41, self.stat);
                self.request_interrupt(Interrupt::Coincidence);
            }
            else {
                self.stat &= !LYC_BIT;
                self.gb_mem.write(0xFF41, self.stat);
            }

            *cycles = 0;
            self.cycles = 0;

            self.gb_mem.write(0xFF44, self.ly);
        }
        // Mode 1 - V-Blank.
        else if self.cycles >= 456 && current_mode == 1 {
            self.ly += 1;

            if self.ly > 153 {
                if self.frame_time.elapsed() < time::Duration::from_millis(16) {
                    let time_to_sleep = time::Duration::from_millis(16).saturating_sub(self.frame_time.elapsed());

                    std::thread::sleep(time_to_sleep);
                }

                self.ly = 0;
                self.set_mode(Mode::OamScan);
                self.frame_time = time::Instant::now();
            }

            if self.ly == self.lyc {
                self.stat |= LYC_BIT;
                self.gb_mem.write(0xFF41, self.stat);
                self.request_interrupt(Interrupt::Coincidence);
            }
            else {
                self.stat &= !LYC_BIT;
                self.gb_mem.write(0xFF41, self.stat);
            }

            *cycles = 0;
            self.cycles = 0;

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
        self.request_interrupt(Interrupt::ModeSwitch(mode));
    }

    fn request_interrupt(&mut self, int: Interrupt) {
        let mut vblank = false;
        let mut if_value = self.gb_mem.read(0xFF0F);

        let enabled = {
            match int {
                Interrupt::Coincidence => (self.stat & LYC_INT_BIT) != 0,
                Interrupt::ModeSwitch(mode) => {
                    match mode {
                        Mode::Vblank => {
                            vblank = true;
                            (self.stat & VBLANK_INT_BIT) != 0
                        }
                        Mode::Hblank => (self.stat & HBLANK_INT_BIT) != 0,
                        Mode::OamScan => (self.stat & OAM_INT_BIT) != 0,
                        Mode::LcdTransfer => false
                    }
                }
            }
        };

        if vblank {
            if_value |= 1;
        }

        if enabled {
            if_value |= 2;
        }

        self.gb_mem.write(0xFF0F, if_value);
    }

    // Draw a screen line using the data in self.backgrounds.
    fn draw_screen_line(&mut self) {
        if let Ok(backgrounds) = self.backgrounds.read() {
            let start = 256 * self.ly.wrapping_add(self.scy) as usize;

            let background = if self.lcdc & 0x08 == 0 { &backgrounds[0] } else { &backgrounds[1] };
            let background_line = &background[start..start+256];

            let mut screen_idx = 160 * self.ly as usize;

            for screen_point in 0..160 {
                let screen_point: u8 = screen_point;
                let background_line_idx: u8 = screen_point.wrapping_add(self.scx);

                if let Ok(mut screen) = self.screen.write() {
                    screen[screen_idx] = background_line[background_line_idx as usize];
                }

                screen_idx += 1;
            }

            let window_enabled = self.lcdc & 0x20 != 0;

            if window_enabled && self.ly >= self.wy {
                let window_on_screen = self.wx <= 166 && self.wy <= 143;

                if window_on_screen {
                    // The window doesn't have a "current line" counter,
                    // so this gives us the current line on the *window* background map.
                    let window_line_offset = self.ly - self.wy;
                    let current_window_line = self.wy + window_line_offset;
                    let background_offset = 256 * window_line_offset as usize;
    
                    let background = if self.lcdc & 0x40 == 0 { &backgrounds[0] } else { &backgrounds[1] };
                    let background_line = &background[background_offset..background_offset+256];
    
                    screen_idx = 160 * current_window_line as usize;
    
                    for screen_point in 0..160 {
                        let screen_point: u8 = screen_point;
                        let background_line_idx: u8 = screen_point.wrapping_add(self.wx - 7);
    
                        if let Ok(mut screen) = self.screen.write() {
                            screen[screen_idx] = background_line[background_line_idx as usize];
                        }
    
                        screen_idx += 1;
                    }
                }
            }
        }
    }

    fn draw_backgrounds(&mut self) {
        let (signed, tiles_start, tiles_end) = if self.lcdc & 0x10 == 0 {(true, 0x8800, 0x9800)} else {(false, 0x8000, 0x9000)};

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
                        let tile_idx = if signed {
                            (*tile_idx as i8 as i16 + 128) as u16
                        }
                        else {
                            *tile_idx as u16
                        };

                        let tile = &tiles[tile_idx as usize];
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
