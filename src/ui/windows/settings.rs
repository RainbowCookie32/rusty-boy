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

                TabItem::new(im_str!("Keybinds")).build(ui, || {
                    ui.bullet_text(im_str!("Gameboy"));
                    ui.separator();

                    ui.text(im_str!("A     "));
                    ui.same_line(0.0);
                    ui.button(&ImString::from(format!("{:#?}", app_state.config.keybinds.gb_a)), [60.0, 0.0]);

                    ui.same_line(160.0);

                    ui.text(im_str!("Up   "));
                    ui.same_line(0.0);
                    ui.button(&ImString::from(format!("{:#?}", app_state.config.keybinds.gb_up)), [60.0, 0.0]);

                    ui.text(im_str!("B     "));
                    ui.same_line(0.0);
                    ui.button(&ImString::from(format!("{:#?}", app_state.config.keybinds.gb_b)), [60.0, 0.0]);

                    ui.same_line(160.0);

                    ui.text(im_str!("Down "));
                    ui.same_line(0.0);
                    ui.button(&ImString::from(format!("{:#?}", app_state.config.keybinds.gb_down)), [60.0, 0.0]);

                    ui.text(im_str!("Start "));
                    ui.same_line(0.0);
                    ui.button(&ImString::from(format!("{:#?}", app_state.config.keybinds.gb_start)), [60.0, 0.0]);

                    ui.same_line(160.0);

                    ui.text(im_str!("Left "));
                    ui.same_line(0.0);
                    ui.button(&ImString::from(format!("{:#?}", app_state.config.keybinds.gb_left)), [60.0, 0.0]);

                    ui.text(im_str!("Select"));
                    ui.same_line(0.0);
                    ui.button(&ImString::from(format!("{:#?}", app_state.config.keybinds.gb_down)), [60.0, 0.0]);

                    ui.same_line(160.0);

                    ui.text(im_str!("Right"));
                    ui.same_line(0.0);
                    ui.button(&ImString::from(format!("{:#?}", app_state.config.keybinds.gb_right)), [60.0, 0.0]);
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
