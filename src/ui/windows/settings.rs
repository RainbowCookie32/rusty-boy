use imgui::*;

use crate::ui::{AppConfig, AppState};

pub struct SettingsWindow;

impl SettingsWindow {
    pub fn init() -> SettingsWindow {
        SettingsWindow {}
    }

    pub fn draw(&mut self, ui: &Ui, app_state: &mut AppState) {
        if let Some(_token) = PopupModal::new("Emulator Settings").begin_popup(ui) {
            TabBar::new("Settings Tabs").build(ui, || {
                TabItem::new("General").build(ui, || {
                    ui.checkbox("Pause emulator on startup", &mut app_state.config.pause_emulator_on_startup);
                    ui.checkbox("Pause emulator on screen focus loss", &mut app_state.config.pause_emulator_on_focus_loss);

                    ui.input_float2("Screen size (Default: 160x144)", &mut app_state.config.screen_size).build();
                });

                TabItem::new("Keybinds").build(ui, || {
                    ui.bullet_text("Gameboy");
                    ui.separator();

                    ui.text("A     ");
                    ui.same_line();
                    ui.button(&ImString::from(format!("{:#?}", app_state.config.keybinds.gb_a)));

                    ui.same_line_with_pos(160.0);

                    ui.text("Up   ");
                    ui.same_line();
                    ui.button(&ImString::from(format!("{:#?}", app_state.config.keybinds.gb_up)));

                    ui.text("B     ");
                    ui.same_line();
                    ui.button(&ImString::from(format!("{:#?}", app_state.config.keybinds.gb_b)));

                    ui.same_line_with_pos(160.0);

                    ui.text("Down ");
                    ui.same_line();
                    ui.button(&ImString::from(format!("{:#?}", app_state.config.keybinds.gb_down)));

                    ui.text("Start ");
                    ui.same_line();
                    ui.button(&ImString::from(format!("{:#?}", app_state.config.keybinds.gb_start)));

                    ui.same_line_with_pos(160.0);

                    ui.text("Left ");
                    ui.same_line();
                    ui.button(&ImString::from(format!("{:#?}", app_state.config.keybinds.gb_left)));

                    ui.text("Select");
                    ui.same_line();
                    ui.button(&ImString::from(format!("{:#?}", app_state.config.keybinds.gb_down)));

                    ui.same_line_with_pos(160.0);

                    ui.text("Right");
                    ui.same_line();
                    ui.button(&ImString::from(format!("{:#?}", app_state.config.keybinds.gb_right)));
                });
            });

            ui.separator();

            if ui.button("Save") {
                app_state.config.save();
                app_state.settings_opened = false;
            }

            ui.same_line();

            if ui.button("Cancel") {
                app_state.config = AppConfig::load();
                app_state.settings_opened = false;
            }
        };

        ui.open_popup("Emulator Settings");
    }
}
