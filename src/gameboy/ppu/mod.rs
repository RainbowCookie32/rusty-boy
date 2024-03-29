pub mod utils;

use std::time;
use std::sync::{Arc, RwLock};

use utils::Palette;

use crate::gameboy::memory::GameboyMemory;
use crate::gameboy::memory::io::IoRegister;

const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;

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

struct Sprite {
    pos_y: u8,
    pos_x: u8,
    tile_id: u8,

    bg_priority: bool,
    flip_x: bool,
    flip_y: bool,
    palette: bool
}

impl Sprite {
    pub fn new(data: &[u8]) -> Sprite {
        let bg_priority = data[3] & 0x80 != 0;
        let flip_x = data[3] & 0x40 != 0;
        let flip_y = data[3] & 0x20 != 0;
        let palette = data[3] & 0x10 != 0;

        Sprite {
            pos_y: data[0].saturating_sub(16),
            pos_x: data[1].saturating_sub(8),
            tile_id: data[2],

            bg_priority,
            flip_x,
            flip_y,
            palette
        }
    }
}

pub struct GameboyPPU {
    lcdc: Arc<IoRegister>,
    stat: Arc<IoRegister>,

    scy: Arc<IoRegister>,
    scx: Arc<IoRegister>,

    ly: Arc<IoRegister>,
    lyc: Arc<IoRegister>,

    wy: Arc<IoRegister>,
    wx: Arc<IoRegister>,

    bg_palette: Palette,
    obj_palettes: Vec<Palette>,

    gb_cyc: Arc<RwLock<usize>>,
    
    screen: Arc<RwLock<Vec<u8>>>,
    backgrounds: Arc<RwLock<Vec<Vec<u8>>>>,

    gb_mem: Arc<RwLock<GameboyMemory>>,
    frame_time: time::Instant,
}

impl GameboyPPU {
    pub fn init(gb_cyc: Arc<RwLock<usize>>, gb_mem: Arc<RwLock<GameboyMemory>>) -> GameboyPPU {
        let lcdc = gb_mem.read().unwrap().get_io_reg(0xFF40);
        let stat = gb_mem.read().unwrap().get_io_reg(0xFF41);
        let scy = gb_mem.read().unwrap().get_io_reg(0xFF42);
        let scx = gb_mem.read().unwrap().get_io_reg(0xFF43);
        let ly = gb_mem.read().unwrap().get_io_reg(0xFF44);
        let lyc = gb_mem.read().unwrap().get_io_reg(0xFF45);
        let wy = gb_mem.read().unwrap().get_io_reg(0xFF4A);
        let wx = gb_mem.read().unwrap().get_io_reg(0xFF4B);

        GameboyPPU {
            lcdc,
            stat,

            scy,
            scx,

            ly,
            lyc,

            wy,
            wx,

            bg_palette: Palette::new(),
            obj_palettes: vec![Palette::new(); 2],

            gb_cyc,

            screen: Arc::new(RwLock::new(vec![255; SCREEN_WIDTH * SCREEN_HEIGHT])),
            backgrounds: Arc::new(RwLock::new(vec![vec![255; 256 * 256]; 2])),

            gb_mem,
            frame_time: time::Instant::now()
        }
    }

    pub fn ppu_cycle(&mut self) {
        let bg_pal = self.read(0xFF47);
        let obj0_pal = self.read(0xFF48) & 0xFC;
        let obj1_pal = self.read(0xFF49) & 0xFC;

        self.bg_palette.update(bg_pal);
        self.obj_palettes[0].update(obj0_pal);
        self.obj_palettes[1].update(obj1_pal);

        if self.lcdc.get() & 0x80 == 0 {
            self.frame_time = time::Instant::now();
            return;
        }

        let current_mode = self.stat.get() & 3;

        // Mode 2 - OAM scan.
        if *self.gb_cyc.read().unwrap() >= 80 && current_mode == 2 {
            *self.gb_cyc.write().unwrap() = 0;
            self.set_mode(Mode::LcdTransfer);
        }
        // Mode 3 - Access OAM and VRAM to generate the picture.
        else if *self.gb_cyc.read().unwrap() >= 172 && current_mode == 3 {
            *self.gb_cyc.write().unwrap() = 0;
            
            self.draw_screen_line();
            self.draw_sprites();

            self.set_mode(Mode::Hblank);
        }
        // Mode 0 - H-Blank.
        else if *self.gb_cyc.read().unwrap() >= 204 && current_mode == 0 {
            self.ly.set(self.ly.get().wrapping_add(1));

            if self.ly.get() < 144 {
                self.set_mode(Mode::OamScan);
            }
            else {
                self.set_mode(Mode::Vblank);
            }

            let mut stat = self.stat.get();

            if self.ly.get() == self.lyc.get() {
                stat |= LYC_BIT;
                self.request_interrupt(Interrupt::Coincidence);
            }
            else {
                stat &= !LYC_BIT;
            }

            self.stat.set(stat);
            *self.gb_cyc.write().unwrap() = 0;
        }
        // Mode 1 - V-Blank.
        else if *self.gb_cyc.read().unwrap() >= 456 && current_mode == 1 {
            self.ly.set(self.ly.get().wrapping_add(1));

            if self.ly.get() > 153 {
                if self.frame_time.elapsed() < time::Duration::from_millis(16) {
                    let time_to_sleep = time::Duration::from_millis(16).saturating_sub(self.frame_time.elapsed());

                    std::thread::sleep(time_to_sleep);
                }

                self.ly.set(0);
                self.set_mode(Mode::OamScan);
                self.frame_time = time::Instant::now();
            }

            let mut stat = self.stat.get();

            if self.ly.get() == self.lyc.get() {
                stat |= LYC_BIT;
                self.request_interrupt(Interrupt::Coincidence);
            }
            else {
                stat &= !LYC_BIT;
            }

            self.stat.set(stat);
            self.draw_backgrounds();
            *self.gb_cyc.write().unwrap() = 0;
        }
    }

    fn read(&self, address: u16) -> u8 {
        if let Ok(lock) = self.gb_mem.read() {
            lock.read(address)
        }
        else {
            0
        }
    }

    fn write(&self, address: u16, value: u8) {
        if let Ok(mut lock) = self.gb_mem.write() {
            lock.write(address, value);
        }
    }

    pub fn get_screen_data(&self) -> Arc<RwLock<Vec<u8>>> {
        self.screen.clone()
    }

    pub fn get_backgrounds_data(&self) -> Arc<RwLock<Vec<Vec<u8>>>> {
        self.backgrounds.clone()
    }

    fn set_mode(&mut self, mode: Mode) {
        let mut stat = self.stat.get() & 0xFC;

        match mode {
            Mode::Vblank => stat |= 1,
            Mode::OamScan => stat |= 2,
            Mode::LcdTransfer => stat |= 3,
            _ => {}
        }

        self.stat.set(stat);
        self.request_interrupt(Interrupt::ModeSwitch(mode));
    }

    fn request_interrupt(&mut self, int: Interrupt) {
        let mut vblank = false;
        let mut if_value = self.read(0xFF0F);

        let enabled = {
            match int {
                Interrupt::Coincidence => (self.stat.get() & LYC_INT_BIT) != 0,
                Interrupt::ModeSwitch(mode) => {
                    match mode {
                        Mode::Vblank => {
                            vblank = true;
                            (self.stat.get() & VBLANK_INT_BIT) != 0
                        }
                        Mode::Hblank => (self.stat.get() & HBLANK_INT_BIT) != 0,
                        Mode::OamScan => (self.stat.get() & OAM_INT_BIT) != 0,
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

        self.write(0xFF0F, if_value);
    }

    // Draw a screen line using the data in self.backgrounds.
    fn draw_screen_line(&mut self) {
        if self.lcdc.get() & 1 == 0 {
            return;
        }

        let ly = self.ly.get();
        let scy = self.scy.get();
        let scx = self.scx.get();
        let lcdc = self.lcdc.get();

        if let Ok(backgrounds) = self.backgrounds.read() {
            let start = 256 * ly.wrapping_add(scy) as usize;

            let background = if lcdc & 0x08 == 0 { &backgrounds[0] } else { &backgrounds[1] };
            let background_line = &background[start..start+256];

            let mut screen_idx = 160 * ly as usize;

            for screen_point in 0..160 {
                let screen_point: u8 = screen_point;
                let background_line_idx: u8 = screen_point.wrapping_add(scx);

                if let Ok(mut screen) = self.screen.write() {
                    screen[screen_idx] = background_line[background_line_idx as usize];
                }

                screen_idx += 1;
            }

            let wy = self.wy.get();
            let wx = self.wx.get();
            let window_enabled = lcdc & 0x20 != 0;

            if window_enabled && ly >= wy {
                let window_on_screen = wx <= 166 && wy <= 143;

                if window_on_screen {
                    // The window doesn't have a "current line" counter,
                    // so this gives us the current line on the *window* background map.
                    let window_line_offset = ly - wy;
                    let current_window_line = wy + window_line_offset;
                    let background_offset = 256 * window_line_offset as usize;
    
                    let background = if lcdc & 0x40 == 0 { &backgrounds[0] } else { &backgrounds[1] };
                    let background_line = &background[background_offset..background_offset+256];
    
                    screen_idx = 160 * current_window_line as usize;
    
                    for screen_point in 0..160 {
                        let screen_point: u8 = screen_point;
                        let background_line_idx: u8 = screen_point.wrapping_add(wx - 7);
    
                        if let Ok(mut screen) = self.screen.write() {
                            screen[screen_idx] = background_line[background_line_idx as usize];
                        }
    
                        screen_idx += 1;
                    }
                }
            }
        }
    }

    fn draw_sprites(&mut self) {
        let ly = self.ly.get();
        let lcdc = self.lcdc.get();

        // OBJ Enabled flag.
        if lcdc & 2 != 0 {
            // Whether to use 8x16 sprites or 8x8.
            let sprite_heigth = if lcdc & 4 != 0 {16} else {8};
            let mut oam_data = Vec::with_capacity(160);
            let mut sprites_to_draw = Vec::with_capacity(10);

            for offset in 0..160 {
                oam_data.push(self.read(0xFE00 + offset));
            }
            
            for chunk in oam_data.chunks_exact(4) {
                let sprite = Sprite::new(chunk);
                
                match ly.cmp(&sprite.pos_y){
                    std::cmp::Ordering::Equal => sprites_to_draw.push(sprite),
                    std::cmp::Ordering::Greater => {
                        if (ly - sprite.pos_y) < sprite_heigth {
                            sprites_to_draw.push(sprite);
                        }
                    }
                    _ => {}
                }

                // Can only draw 10 sprites per line.
                if sprites_to_draw.len() >= 10 {
                    break;
                }
            }

            for sprite in sprites_to_draw {
                // Sprite is off-screen.
                if sprite.pos_x == 0 || sprite.pos_x >= 160 || sprite.pos_y == 0 || sprite.pos_y >= 144 {
                    continue;
                }

                let sprite_line_offset = (ly - sprite.pos_y) as usize;
                let mut tile_data = Vec::with_capacity((sprite_heigth * 2) as usize);

                let palette = if !sprite.palette {&self.obj_palettes[0]} else {&self.obj_palettes[1]};

                if sprite_heigth == 16 {
                    let tiles = [sprite.tile_id & 0xFE, sprite.tile_id | 1];

                    for idx in tiles {
                        let tile_addr = 0x8000 + (16 * idx as u16);
                        
                        for offset in 0..16 {
                            tile_data.push(self.read(tile_addr + offset));
                        }
                    }
                }
                else {
                    let idx = sprite.tile_id as u16;
                    let tile_addr = 0x8000 + (16 * idx);
                        
                    for offset in 0..16 {
                        tile_data.push(self.read(tile_addr + offset));
                    }
                }

                let idx = {
                    if sprite.flip_x {
                        ((sprite_heigth as usize * 2) - 2) - (2 * sprite_line_offset)
                    }
                    else {
                        2 * sprite_line_offset
                    }
                };
                let sprite_line = &tile_data[idx..idx+2];

                let mut result = Vec::new();
                let mut screen_idx = (160 * ly as usize) + sprite.pos_x as usize;

                if sprite.flip_y {
                    for bit in 0..8 {
                        let color_idx = ((sprite_line[0] >> bit) & 1) | (((sprite_line[1] >> bit) & 1) << 1);
                        result.push(color_idx);
                    }
                }
                else {
                    for bit in (0..8).rev() {
                        let color_idx = ((sprite_line[0] >> bit) & 1) | (((sprite_line[1] >> bit) & 1) << 1);
                        result.push(color_idx);
                    }
                }

                for color_idx in result {
                    if color_idx == 0 {
                        screen_idx += 1;
                        continue;
                    }

                    let pixel_color = palette.get_color(color_idx);
    
                    if let Ok(mut lock) = self.screen.write() {
                        if sprite.bg_priority {
                            let point_color = lock[screen_idx];
                            let color_0 = self.bg_palette.get_color(0);
    
                            if point_color == color_0 {
                                lock[screen_idx] = pixel_color;
                            }
                        }
                        else {
                            lock[screen_idx] = pixel_color;
                        }
                    }
    
                    screen_idx += 1;
                }
            }
        }
    }

    fn draw_backgrounds(&mut self) {
        let (signed, tiles_start, tiles_end) = if self.lcdc.get() & 0x10 == 0 {(true, 0x8800, 0x9800)} else {(false, 0x8000, 0x9000)};

        if let Ok(mut lock) = self.backgrounds.write() {
            for (bg_idx, background) in lock.iter_mut().enumerate() {
                let (map_start, map_end) = if bg_idx == 0 {(0x9800, 0x9C00)} else {(0x9C00, 0xA000)};

                let tiles = {
                    let mut res = Vec::new();
                    let mut data = Vec::new();

                    for address in tiles_start..tiles_end {
                        data.push(self.read(address));
                    }

                    data.chunks_exact(16).for_each(|t| res.push(t.to_owned()));
                    res
                };

                let map_data = {
                    let mut res = Vec::with_capacity(1024);

                    for address in map_start..map_end {
                        res.push(self.read(address));
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

                        let tile = utils::create_tile(&tiles[tile_idx as usize], &self.bg_palette);
                        let tile_data = tile.chunks_exact(8);

                        for (tile_y, line) in tile_data.enumerate() {
                            let mut idx = x_offset + (256 * (y_offset + tile_y));

                            for pixel in line {
                                background[idx] = *pixel;
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
