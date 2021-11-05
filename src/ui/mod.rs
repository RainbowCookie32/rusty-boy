mod windows;

use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::sync::mpsc::Sender;

use imgui::*;

use imgui_glium_renderer::{Renderer, Texture};
use imgui_winit_support::{HiDpiMode, WinitPlatform};

use glium::{Display, Surface};
use glium::glutin::ContextBuilder;
use glium::glutin::dpi::LogicalSize;
use glium::glutin::window::WindowBuilder;
use glium::glutin::event_loop::{ControlFlow, EventLoop};
use glium::glutin::event::{ElementState, Event, VirtualKeyCode, WindowEvent};

use serde::{Deserialize, Serialize};

use ron::de::from_reader;
use ron::ser::{PrettyConfig, to_string_pretty};

use windows::*;
use windows::settings::SettingsWindow;
use windows::notification::Notification;
use windows::file_picker::FilePickerWindow;

use crate::gameboy::memory::GameboyMemory;
use crate::gameboy::{EmulatorMode, Gameboy, JoypadHandler};


pub struct AppState {
    config: AppConfig,

    rom_data: Vec<u8>,
    bootrom_data: Vec<u8>,

    reload: bool,
    picking_rom: bool,
    picking_bootrom: bool,
    settings_opened: bool,

    gb: Option<Arc<RwLock<Gameboy>>>,
    gb_mem: Option<Arc<RwLock<GameboyMemory>>>,
    gb_exit_tx: Option<Sender<()>>,

    notifications: Vec<Notification>,
    file_picker_instance: FilePickerWindow,

    window_cart_info: (bool, Option<cart_info::CartWindow>),
    window_cpu_debugger: (bool, Option<cpu_debugger::CPUWindow>),
    window_disassembler: (bool, Option<disassembler::DisassemblerWindow>),
    window_memory_viewer: (bool, Option<memory_viewer::MemoryWindow>),
    window_screen: (bool, Option<screen::ScreenWindow>),
    window_serial: (bool, Option<serial_output::SerialWindow>),
    window_vram_viewer: (bool, Option<vram_viewer::VramViewerWindow>)
}

impl AppState {
    pub fn init() -> AppState {
        let config = AppConfig::load();
        let current_path = config.last_dir_rom.clone();

        AppState {
            config,

            rom_data: Vec::new(),
            bootrom_data: Vec::new(),

            reload: false,
            picking_rom: false,
            picking_bootrom: false,
            settings_opened: false,

            gb: None,
            gb_mem: None,
            gb_exit_tx: None,

            notifications: Vec::new(),
            file_picker_instance: FilePickerWindow::init(current_path),

            window_cart_info: (false, None),
            window_cpu_debugger: (false, None),
            window_disassembler: (false, None),
            window_memory_viewer: (false, None),
            window_screen: (false, None),
            window_serial: (false, None),
            window_vram_viewer: (false, None)
        }
    }

    fn emu_reset(&self) {
        if let Some(gb) = self.gb.as_ref() {
            if let Ok(mut lock) = gb.write() {
                lock.gb_reset();
            }
        }
    }

    fn emu_do_step(&self) {
        if let Some(gb) = self.gb.as_ref() {
            if let Ok(mut lock) = gb.write() {
                lock.dbg_do_step = true;
            }
        }
    }

    fn emu_get_mode(&self) -> EmulatorMode {
        if let Some(gb) = self.gb.as_ref() {
            if let Ok(lock) = gb.read() {
                return lock.dbg_mode.clone();
            }
        }
        
        EmulatorMode::Paused
    }

    fn emu_set_mode(&self, mode: EmulatorMode) {
        if let Some(gb) = self.gb.as_ref() {
            if let Ok(mut lock) = gb.write() {
                lock.dbg_mode = mode;
            }
        }
    }
}

#[derive(Default, Deserialize, Serialize)]
pub struct AppConfig {
    keybinds: Keybinds,
    screen_size: [f32; 2],

    pause_emulator_on_startup: bool,
    pause_emulator_on_focus_loss: bool,

    last_dir_rom: PathBuf,
    last_dir_bootrom: PathBuf
}

impl AppConfig {
    pub fn load() -> AppConfig {
        if let Ok(file) = std::fs::File::open("config.ron") {
            if let Ok(config) = from_reader(file) {
                return config;
            }
        }
        
        AppConfig {
            screen_size: [160.0, 144.0],
            ..Default::default()
        }
    }

    pub fn save(&self) {
        if let Ok(data) = to_string_pretty(self, PrettyConfig::default()) {
            if let Err(error) = std::fs::write("config.ron", data) {
                println!("Error saving config: {}", error.to_string());
            }
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct Keybinds {
    gb_a: VirtualKeyCode,
    gb_b: VirtualKeyCode,
    gb_start: VirtualKeyCode,
    gb_select: VirtualKeyCode,

    gb_up: VirtualKeyCode,
    gb_down: VirtualKeyCode,
    gb_left: VirtualKeyCode,
    gb_right: VirtualKeyCode,

    emu_step: VirtualKeyCode,
    emu_resume: VirtualKeyCode
}

impl Default for Keybinds {
    fn default() -> Keybinds {
        Keybinds {
            gb_a: VirtualKeyCode::A,
            gb_b: VirtualKeyCode::S,
            gb_start: VirtualKeyCode::Return,
            gb_select: VirtualKeyCode::RShift,

            gb_up: VirtualKeyCode::Up,
            gb_down: VirtualKeyCode::Down,
            gb_left: VirtualKeyCode::Left,
            gb_right: VirtualKeyCode::Right,

            emu_step: VirtualKeyCode::F3,
            emu_resume: VirtualKeyCode::F9
        }
    }
}

pub fn run_app() {
    let event_loop = EventLoop::new();
    let glutin_context = ContextBuilder::new().with_vsync(true);
    let window_builder = WindowBuilder::new().with_title("rusty-boy").with_inner_size(LogicalSize::new(1280, 768));
    let display = Display::new(window_builder, glutin_context, &event_loop)
        .expect("Failed to create glium display")
    ;

    let mut imgui_ctx = Context::create();
    let mut winit_platform = WinitPlatform::init(&mut imgui_ctx);

    {
        let gl_window = display.gl_window();
        let window = gl_window.window();
        winit_platform.attach_window(imgui_ctx.io_mut(), window, HiDpiMode::Rounded);
    }

    let mut renderer = Renderer::init(&mut imgui_ctx, &display)
        .expect("Failed to create imgui renderer")
    ;

    let mut app_state = AppState::init();
    let mut settings_window = SettingsWindow::init();

    event_loop.run(move | event, _, control_flow| {
        match event {
            Event::MainEventsCleared => {
                let gl_window = display.gl_window();

                winit_platform.prepare_frame(imgui_ctx.io_mut(), gl_window.window()).unwrap();
                gl_window.window().request_redraw();
            }
            Event::RedrawRequested(_) => {
                let ui = imgui_ctx.frame();
                
                draw_menu_bar(&mut app_state, &ui, control_flow);

                if app_state.picking_rom {
                    draw_rom_picker(&mut app_state, &ui);
                }

                if app_state.picking_bootrom {
                    draw_bootrom_picker(&mut app_state, &ui);
                }

                if app_state.settings_opened {
                    settings_window.draw(&ui, &mut app_state);
                }

                if app_state.reload {
                    reload_app(&mut app_state, &ui);
                }
                else if app_state.gb.is_some() {
                    draw_windows(&mut app_state, &ui, &display, renderer.textures());
                }

                show_notifications(&mut app_state, &ui);

                let gl_window = display.gl_window();
                let mut target = display.draw();

                target.clear_color_srgb(0.2, 0.2, 0.2, 1.0);
                winit_platform.prepare_render(&ui, gl_window.window());

                let draw_data = ui.render();

                renderer.render(&mut target, draw_data).unwrap();
                target.finish().unwrap();
            }
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent { event: WindowEvent::KeyboardInput { input, ..}, ..} => {
                if input.state == ElementState::Pressed {
                    if let Some(keycode) = input.virtual_keycode {
                        match keycode {
                            VirtualKeyCode::F3 => {
                                if app_state.emu_get_mode() == EmulatorMode::Stepping {
                                    app_state.emu_do_step();
                                }
                            }
                            VirtualKeyCode::F9 => {
                                if app_state.emu_get_mode() != EmulatorMode::Running {
                                    app_state.emu_set_mode(EmulatorMode::Running)
                                }
                                else {
                                    app_state.emu_set_mode(EmulatorMode::Paused)
                                }
                            }
                            _ => {}
                        }
                    }
                }

                winit_platform.handle_event(imgui_ctx.io_mut(), display.gl_window().window(), &event);
            }
            event => {
                winit_platform.handle_event(imgui_ctx.io_mut(), display.gl_window().window(), &event);
            }
        }
    });
}

fn create_windows(app_state: &mut AppState) {
    if let Some(gb) = app_state.gb.as_ref() {
        app_state.window_cart_info = (true, Some(cart_info::CartWindow::init(gb.clone())));
        app_state.window_cpu_debugger = (false, Some(cpu_debugger::CPUWindow::init(gb.clone())));

        if let Some(gb_mem) = app_state.gb_mem.as_ref() {
            app_state.window_disassembler = (false, Some(disassembler::DisassemblerWindow::init(gb.clone())));
            app_state.window_memory_viewer = (false, Some(memory_viewer::MemoryWindow::init(gb_mem.clone())));
        }

        app_state.window_screen = (true, Some(screen::ScreenWindow::init(gb.clone())));
        app_state.window_serial = (false, Some(serial_output::SerialWindow::init(gb.clone())));
        app_state.window_vram_viewer = (false, Some(vram_viewer::VramViewerWindow::init(gb.clone())));
    }
}

fn reload_app(app_state: &mut AppState, ui: &Ui) {
    if !app_state.rom_data.is_empty() && !app_state.bootrom_data.is_empty() {
        let bootrom_data = app_state.bootrom_data.clone();
        let romfile_data = app_state.rom_data.clone();

        let gb_joy = Arc::new(RwLock::new(JoypadHandler::default()));

        let gb_mem = Arc::new(RwLock::new(GameboyMemory::init(bootrom_data, romfile_data, gb_joy)));
        let gb = Arc::new(RwLock::new(Gameboy::init(gb_mem.clone())));

        let gb_exit_tx = Gameboy::gb_start(gb.clone());

        app_state.gb = Some(gb);
        app_state.gb_mem = Some(gb_mem);
        app_state.gb_exit_tx = Some(gb_exit_tx);

        app_state.notifications.push(
            Notification::init(
                ImString::new("rusty-boy"),
                ImString::new("Emulator ready!"),
                ui.time()
            )
        );

        create_windows(app_state);

        if !app_state.config.pause_emulator_on_startup {
            app_state.emu_set_mode(EmulatorMode::Running);
        }
    }

    app_state.reload = false;
}

fn show_notifications(app_state: &mut AppState, ui: &Ui) {
    let mut finished_notifications = 0;

    for (i, n) in app_state.notifications.iter_mut().enumerate() {
        if n.draw(ui, i) {
            finished_notifications += 1;
        }
    }

    for _ in 0..finished_notifications {
        app_state.notifications.remove(0);
    }
}

fn draw_menu_bar(app_state: &mut AppState, ui: &Ui, control_flow: &mut ControlFlow) {
    ui.main_menu_bar(|| {
        ui.menu("File", || {
            if MenuItem::new("Load ROM").build(ui) {
                app_state.picking_rom = true;
                app_state.file_picker_instance = FilePickerWindow::init(app_state.config.last_dir_rom.clone());
            }

            if MenuItem::new("Load Bootrom").build(ui) {
                app_state.picking_bootrom = true;
                app_state.file_picker_instance = FilePickerWindow::init(app_state.config.last_dir_bootrom.clone());
            }

            ui.separator();

            if MenuItem::new("Reload").enabled(app_state.gb.is_some()).build(ui) {
                if let Some(tx) = app_state.gb_exit_tx.as_ref() {
                    tx.send(()).unwrap();
                }

                app_state.reload = true;

                app_state.gb = None;
                app_state.gb_mem = None;
                app_state.gb_exit_tx = None;
            }

            ui.separator();

            if MenuItem::new("Settings").build(ui) {
                app_state.settings_opened = true;
            }

            if MenuItem::new("Exit").build(ui) {
                *control_flow = ControlFlow::Exit;
            }
        });

        ui.menu_with_enabled("Emulator", app_state.gb.is_some(), || {
            let mode = app_state.emu_get_mode();
            
            match mode {
                EmulatorMode::Running => {
                    if MenuItem::new("Pause").build(ui) {
                        app_state.emu_set_mode(EmulatorMode::Paused);
                    }
                }
                EmulatorMode::UnknownInstruction(_, _) => {
                    MenuItem::new("Resume").enabled(false).build(ui);
                }
                _ => {
                    if MenuItem::new("Resume").build(ui) {
                        app_state.emu_set_mode(EmulatorMode::Running);
                    }
                }
            }

            if MenuItem::new("Restart").build(ui) {
                app_state.emu_reset();
            }
        });

        ui.menu_with_enabled("View", app_state.gb.is_some(), || {
            if app_state.window_cart_info.0 {
                if MenuItem::new("Hide cartridge info").build(ui) {
                    app_state.window_cart_info.0 = false;
                }
            }
            else if MenuItem::new("Show cartridge info").build(ui) {
                app_state.window_cart_info.0 = true;
            }

            if app_state.window_cpu_debugger.0 {
                if MenuItem::new("Hide CPU debugger").build(ui) {
                    app_state.window_cpu_debugger.0 = false;
                }
            }
            else if MenuItem::new("Show CPU debugger").build(ui) {
                app_state.window_cpu_debugger.0 = true;
            }

            if app_state.window_disassembler.0 {
                if MenuItem::new("Hide disassembler").build(ui) {
                    app_state.window_disassembler.0 = false;
                }
            }
            else if MenuItem::new("Show disassembler").build(ui) {
                app_state.window_disassembler.0 = true;
            }

            if app_state.window_memory_viewer.0 {
                if MenuItem::new("Hide memory viewer").build(ui) {
                    app_state.window_memory_viewer.0 = false;
                }
            }
            else if MenuItem::new("Show memory viewer").build(ui) {
                app_state.window_memory_viewer.0 = true;
            }

            if app_state.window_serial.0 {
                if MenuItem::new("Hide serial output").build(ui) {
                    app_state.window_serial.0 = false;
                }
            }
            else if MenuItem::new("Show serial output").build(ui) {
                app_state.window_serial.0 = true;
            }

            if app_state.window_vram_viewer.0 {
                if MenuItem::new("Hide VRAM viewer").build(ui) {
                    app_state.window_vram_viewer.0 = false;
                }
            }
            else if MenuItem::new("Show VRAM viewer").build(ui) {
                app_state.window_vram_viewer.0 = true;
            }
        });
    });
}

fn draw_windows(app_state: &mut AppState, ui: &Ui, display: &Display, textures: &mut Textures<Texture>) {
    let mut adjust = false;

    if app_state.window_cart_info.0 {
        if let Some(cart_win) = app_state.window_cart_info.1.as_ref() {
            cart_win.draw(ui);
        }
    }
    
    if app_state.window_cpu_debugger.0 {
        if let Some(cpu_win) = app_state.window_cpu_debugger.1.as_mut() {
            adjust = cpu_win.draw(ui);
        }
    }

    if app_state.window_disassembler.0 {
        if let Some(disas_win) = app_state.window_disassembler.1.as_mut() {
            disas_win.draw(ui, adjust);
        }
    }

    if app_state.window_memory_viewer.0 {
        if let Some(mem_win) = app_state.window_memory_viewer.1.as_mut() {
            mem_win.draw(ui);
        }
    }

    if app_state.window_screen.0 {
        if let Some(screen_win) = app_state.window_screen.1.as_mut() {
            if !screen_win.draw(&mut app_state.config, ui, display, textures) && app_state.config.pause_emulator_on_focus_loss {
                app_state.emu_set_mode(EmulatorMode::Paused);
            }
        }
    }

    if app_state.window_serial.0 {
        if let Some(serial_win) = app_state.window_serial.1.as_mut() {
            serial_win.draw(ui);
        }
    }

    if app_state.window_vram_viewer.0 {
        if let Some(vram_win) = app_state.window_vram_viewer.1.as_mut() {
            vram_win.draw(ui, display, textures);
        }
    }
}

fn draw_rom_picker(app_state: &mut AppState, ui: &Ui) {
    if let Some(path) = app_state.file_picker_instance.draw(ui) {
        if path.exists() {
            let rom_result = std::fs::read(&path);

            if let Ok(data) = rom_result {
                let filename = {
                    if let Some(filename) = path.file_name() {
                        filename.to_string_lossy()
                    }
                    else {
                        std::borrow::Cow::from("filename")
                    }
                };

                app_state.rom_data = data;
                app_state.reload = true;
                app_state.picking_rom = false;
                app_state.config.last_dir_rom = path.parent().unwrap().into();
        
                app_state.config.save();

                app_state.notifications.push(
                    Notification::init(
                        ImString::new("Loader"),
                        ImString::new(format!("Loaded ROM file {}.", filename)),
                        ui.time()
                    )
                );
            }
            else if let Err(error) = rom_result {
                app_state.reload = false;

                app_state.notifications.push(
                    Notification::init(
                        ImString::new("Loader"),
                        ImString::new(format!("Failed to load ROM file ({}).", error.to_string())),
                        ui.time()
                    )
                );
            }
        }
    }
}

fn draw_bootrom_picker(app_state: &mut AppState, ui: &Ui) {
    if let Some(path) = app_state.file_picker_instance.draw(ui) {
        if path.exists() {
            let bootrom_result = std::fs::read(&path);

            if let Ok(data) = bootrom_result {
                let filename = {
                    if let Some(filename) = path.file_name() {
                        filename.to_string_lossy()
                    }
                    else {
                        std::borrow::Cow::from("filename")
                    }
                };

                app_state.bootrom_data = data;
                app_state.reload = true;
                app_state.picking_bootrom = false;
                app_state.config.last_dir_bootrom = path.parent().unwrap().into();
        
                app_state.config.save();

                app_state.notifications.push(
                    Notification::init(
                        ImString::new("Loader"),
                        ImString::new(format!("Loaded bootrom file {}.", filename)),
                        ui.time()
                    )
                );
            }
            else if let Err(error) = bootrom_result {
                app_state.reload = false;

                app_state.notifications.push(
                    Notification::init(
                        ImString::new("Loader"),
                        ImString::new(format!("Failed to load bootrom file ({}).", error.to_string())),
                        ui.time()
                    )
                );
            }
        }
    }
}
