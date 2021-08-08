use imgui::*;

use crate::ui::{AppConfig, AppState};

pub struct SettingsWindow;

impl SettingsWindow {
    pub fn init() -> SettingsWindow {
        SettingsWindow {}
    }

    pub fn draw(&mut self, ui: &Ui, app_state: &mut AppState) {
        ui.popup_modal(im_str!("Emulator Settings")).build(|| {
            TabBar::new(im_str!("Settings Tabs")).build(ui, || {
                TabItem::new(im_str!("General")).build(ui, || {
                    ui.checkbox(im_str!("Pause emulator on startup"), &mut app_state.config.pause_emulator_on_startup);
                    ui.checkbox(im_str!("Pause emulator on screen focus loss"), &mut app_state.config.pause_emulator_on_focus_loss);

                    ui.input_float2(im_str!("Screen size (Default: 160x140)"), &mut app_state.config.screen_size).build();
                });
            });

            ui.separator();

            if ui.button(im_str!("Save"), [0.0, 0.0]) {
                app_state.config.save();
                app_state.settings_opened = false;
            }

            ui.same_line(0.0);

            if ui.button(im_str!("Cancel"), [0.0, 0.0]) {
                app_state.config = AppConfig::load();
                app_state.settings_opened = false;
            }
        });

        ui.open_popup(im_str!("Emulator Settings"));
    }
}
