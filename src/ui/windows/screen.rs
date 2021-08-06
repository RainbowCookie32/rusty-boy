use std::sync::{Arc, RwLock};

use imgui::*;
use imgui_glium_renderer::Texture;

use glium::Display;
use glium::glutin::event::VirtualKeyCode;

use crate::gameboy::Gameboy;
use crate::gameboy::JoypadHandler;
use crate::ui::windows::GameboyTexture;

const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;

pub struct ScreenWindow {
    screen: GameboyTexture,

    gb_joy: Arc<RwLock<JoypadHandler>>,
    screen_data: Arc<RwLock<Vec<u8>>>,
}

impl ScreenWindow {
    pub fn init(gb: Arc<RwLock<Gameboy>>) -> ScreenWindow {
        let gb_joy = gb.read().unwrap().ui_get_joypad_handler();
        let screen_data = gb.read().unwrap().ui_get_screen_data();

        ScreenWindow {
            screen: GameboyTexture::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32),

            gb_joy,
            screen_data
        }
    }

    pub fn draw(&mut self, ui: &Ui, display: &Display, textures: &mut Textures<Texture>) {
        let size = [SCREEN_WIDTH as f32 * 2.0, SCREEN_HEIGHT as f32 * 2.0];

        Window::new(im_str!("Screen")).size(size, Condition::Once).build(ui, || {
            let window_size = ui.content_region_avail();

            let x_scale = window_size[0] / SCREEN_WIDTH as f32;
            let y_scale = window_size[1] / SCREEN_HEIGHT as f32;

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
                let w = SCREEN_WIDTH as f32 * x_scale;
                let h = SCREEN_HEIGHT as f32 * y_scale;

                Image::new(*id, [w as f32, h as f32]).build(ui);
            }

            if ui.is_window_focused() {
                if let Ok(mut lock) = self.gb_joy.write() {
                    lock.set_down_state(ui.is_key_down(Key::DownArrow));
                    lock.set_up_state(ui.is_key_down(Key::UpArrow));
                    lock.set_left_state(ui.is_key_down(Key::LeftArrow));
                    lock.set_right_state(ui.is_key_down(Key::RightArrow));

                    lock.set_start_state(ui.is_key_down(Key::Enter));
                    lock.set_select_state(ui.io().keys_down[VirtualKeyCode::RShift as usize]);
                    lock.set_b_state(ui.io().keys_down[VirtualKeyCode::S as usize]);
                    lock.set_a_state(ui.io().keys_down[VirtualKeyCode::A as usize]);
                }
            }
        });
    }
}
