mod gameboy;

use std::fs;
use std::sync::{Arc, RwLock};

use gameboy::*;
use gameboy::disassembler;
use gameboy::memory::GameboyMemory;

use clap::{Arg, App};

use imgui::*;

use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};

use glium::glutin;
use glium::{Display, Surface};
use glium::glutin::event::{Event, WindowEvent};
use glium::glutin::event_loop::{ControlFlow, EventLoop};

struct AppState {
    dbg_breakpoint_read: bool,
    dbg_breakpoint_write: bool,
    dbg_breakpoint_execute: bool,
    dbg_breakpoint_input: ImString,

    dbg_breakpoint_edit_read: bool,
    dbg_breakpoint_edit_write: bool,
    dbg_breakpoint_edit_execute: bool,
    dbg_breakpoint_edit_opened: bool,
    dbg_breakpoint_edit_address: ImString,
    dbg_breakpoint_edit_selected_idx: usize,

    mem_viewer_edit_byte_active: bool,
    mem_viewer_edit_byte_address: u16,
    mem_viewer_edit_byte_value: ImString
}

fn main() {
    let matches = App::new("rusty-boy")
        .author("RainbowCookie32")
        .about("A (probably broken) Gameboy emulator written in Rust")
        .arg(
            Arg::with_name("bootrom")
                .short("b")
                .long("bootrom")
                .takes_value(true)
                .help("Path to a Gameboy bootrom.")
        )
        .arg(
            Arg::with_name("romfile")
                .short("r")
                .long("romfile")
                .takes_value(true)
                .help("Path to a Gameboy ROM file.")
        )
        .get_matches()
    ;

    let bootrom_path = matches.value_of("bootrom").expect("Path to bootrom wasn't specified").trim();
    let romfile_path = matches.value_of("romfile").expect("Path to romfile wasn't specified").trim();

    let bootrom_data = fs::read(bootrom_path).expect(&format!("Couldn't read bootrom file at path {}", bootrom_path));
    let romfile_data = fs::read(romfile_path).expect(&format!("Couldn't read Gameboy romfile at path {}", romfile_path));

    let event_loop = EventLoop::new();
    let glutin_context = glutin::ContextBuilder::new().with_vsync(true);
    let window_builder = glutin::window::WindowBuilder::new()
        .with_title("rusty-boy")
        .with_inner_size(glutin::dpi::LogicalSize::new(1024, 600))
    ;
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

    let gb_mem = Arc::from(GameboyMemory::init(bootrom_data, romfile_data));
    let gb = Arc::from(RwLock::from(Gameboy::init(gb_mem.clone())));
    
    let gb_ui = gb.clone();
    let gb_mem_ui = gb_mem;

    std::thread::spawn(move || {
        let gameboy = gb_ui;

        loop {
            if let Ok(mut lock) = gameboy.try_write() {
                if lock.dbg_mode == EmulatorMode::Running {
                    lock.gb_cpu_cycle();
                }
                else if lock.dbg_mode == EmulatorMode::Stepping {
                    if lock.dbg_do_step {
                        lock.gb_cpu_cycle();
                        lock.dbg_do_step = false;
                    }
                }
            }
        }
    });

    let mut app_state = AppState {
        dbg_breakpoint_read: false,
        dbg_breakpoint_write: false,
        dbg_breakpoint_execute: false,
        dbg_breakpoint_input: ImString::new("FFFF"),

        dbg_breakpoint_edit_read: false,
        dbg_breakpoint_edit_write: false,
        dbg_breakpoint_edit_execute: false,
        dbg_breakpoint_edit_opened: false,
        dbg_breakpoint_edit_address: ImString::new(""),
        dbg_breakpoint_edit_selected_idx: 0,

        mem_viewer_edit_byte_active: false,
        mem_viewer_edit_byte_address: 0x0000,
        mem_viewer_edit_byte_value: ImString::new("")
    };

    event_loop.run(move | event, _, control_flow| {
        match event {
            Event::MainEventsCleared => {
                let gl_window = display.gl_window();

                winit_platform.prepare_frame(imgui_ctx.io_mut(), &gl_window.window()).unwrap();
                gl_window.window().request_redraw();
            }
            Event::RedrawRequested(_) => {
                let ui = imgui_ctx.frame();

                Window::new(im_str!("ROM Info")).build(&ui, || {
                    if let Ok(lock) = gb.read() {
                        let header = lock.ui_get_header();

                        ui.text(format!("ROM Title: {}", header.title()));
                    
                        ui.separator();

                        ui.text(format!("ROM Size: {} ({} banks)", header.rom_size(), header.rom_banks_count()));
                        ui.text(format!("RAM Size: {} ({} banks)", header.ram_size(), header.ram_banks_count()));
                        ui.text(format!("Cartridge Controller: {}", header.cart_type()));
                    }
                });

                let mut pc_ui = 0;

                Window::new(im_str!("CPU Debugger")).build(&ui, || {
                    if let Ok(mut lock) = gb.write() {
                        let (af, bc, de, hl, sp, pc) = lock.ui_get_cpu_registers();

                        pc_ui = *pc;

                        ui.bullet_text(im_str!("CPU Registers"));
                        
                        ui.text(format!("AF: {:04X}", af));
                        ui.same_line(0.0);
                        ui.text(format!("BC: {:04X}", bc));
                        
                        ui.text(format!("DE: {:04X}", de));
                        ui.same_line(0.0);
                        ui.text(format!("HL: {:04X}", hl));
    
                        ui.text(format!("SP: {:04X}", sp));
                        ui.same_line(0.0);
                        ui.text(format!("PC: {:04X}", pc));
    
                        ui.separator();
    
                        ui.bullet_text(im_str!("CPU Controls"));
                        ui.bullet_text(&ImString::from(format!("Current state: {}", lock.dbg_mode)));
    
                        if lock.dbg_mode == EmulatorMode::Running {
                            if ui.button(im_str!("Pause"), [0.0, 0.0]) {
                                lock.dbg_mode = EmulatorMode::Paused;
                            }
                        }
                        else {
                            if ui.button(im_str!("Resume"), [0.0, 0.0]) {
                                lock.dbg_mode = EmulatorMode::Running;
                            }
                        }
    
                        ui.same_line(0.0);
    
                        if ui.button(im_str!("Step"), [0.0, 0.0]) {
                            lock.dbg_mode = EmulatorMode::Stepping;
                            lock.dbg_do_step = true;
                        }
    
                        ui.separator();
    
                        ui.bullet_text(im_str!("CPU Breakpoints"));
                        ListBox::new(im_str!("")).size([220.0, 70.0]).build(&ui, || {
                            for (idx, bp) in lock.dbg_breakpoint_list.iter().enumerate() {
                                let bp_string = format!(
                                    "{:04X} - {}{}{}",
                                    bp.address(),
                                    if *bp.read() {"r"} else {""},
                                    if *bp.write() {"w"} else {""},
                                    if *bp.execute() {"x"} else {""},
                                );
    
                                if Selectable::new(&ImString::from(bp_string)).allow_double_click(true).build(&ui) {
                                    if ui.is_mouse_double_clicked(MouseButton::Left) {
                                        app_state.dbg_breakpoint_edit_read = *bp.read();
                                        app_state.dbg_breakpoint_edit_write = *bp.write();
                                        app_state.dbg_breakpoint_edit_execute = *bp.execute();
                                        app_state.dbg_breakpoint_edit_address = ImString::new(format!("{:04X}", bp.address()));

                                        app_state.dbg_breakpoint_edit_opened = true;
                                        app_state.dbg_breakpoint_edit_selected_idx = idx;
                                    }
                                }
                            }
                        });

                        if app_state.dbg_breakpoint_edit_opened {
                            ui.open_popup(im_str!("Edit breakpoint"));
                            ui.popup_modal(im_str!("Edit breakpoint")).build(|| {
                                ui.input_text(im_str!("Address"), &mut app_state.dbg_breakpoint_edit_address).resize_buffer(true).build();
                                ui.separator();

                                ui.checkbox(im_str!("Read"), &mut app_state.dbg_breakpoint_edit_read);
                                ui.same_line(0.0);
                                ui.checkbox(im_str!("Write"), &mut app_state.dbg_breakpoint_edit_write);
                                ui.same_line(0.0);
                                ui.checkbox(im_str!("Execute"), &mut app_state.dbg_breakpoint_edit_execute);

                                ui.separator();

                                if ui.button(im_str!("Save"), [0.0, 0.0]) {
                                    if let Some(bp) = lock.dbg_breakpoint_list.get_mut(app_state.dbg_breakpoint_edit_selected_idx) {
                                        if let Ok(address) = u16::from_str_radix(&app_state.dbg_breakpoint_edit_address.to_string(), 16) {
                                            bp.set_address(address);
                                        }

                                        bp.set_read(app_state.dbg_breakpoint_edit_read);
                                        bp.set_write(app_state.dbg_breakpoint_edit_write);
                                        bp.set_execute(app_state.dbg_breakpoint_edit_execute);
                                    }

                                    app_state.dbg_breakpoint_edit_opened = false;
                                }

                                ui.same_line(0.0);

                                if ui.button(im_str!("Remove"), [0.0, 0.0]) {
                                    lock.dbg_breakpoint_list.remove(app_state.dbg_breakpoint_edit_selected_idx);
                                    app_state.dbg_breakpoint_edit_opened = false;
                                }

                                ui.same_line(0.0);

                                if ui.button(im_str!("Cancel"), [0.0, 0.0]) {
                                    app_state.dbg_breakpoint_edit_opened = false;
                                }
                            });
                        }
    
                        let submitted_input: bool;
                        let submitted_button: bool;
    
                        submitted_input = ui.input_text(im_str!(""), &mut app_state.dbg_breakpoint_input).enter_returns_true(true).build();
                        ui.same_line(0.0);
                        submitted_button = ui.button(im_str!("Add"), [0.0, 0.0]);
    
                        ui.checkbox(im_str!("Read"), &mut app_state.dbg_breakpoint_read);
                        ui.same_line(0.0);
                        ui.checkbox(im_str!("Write"), &mut app_state.dbg_breakpoint_write);
                        ui.same_line(0.0);
                        ui.checkbox(im_str!("Execute"), &mut app_state.dbg_breakpoint_execute);
    
                        if submitted_input || submitted_button {
                            if app_state.dbg_breakpoint_read || app_state.dbg_breakpoint_write || app_state.dbg_breakpoint_execute {
                                if let Ok(address) = u16::from_str_radix(&app_state.dbg_breakpoint_input.to_string(), 16) {
                                    lock.dbg_breakpoint_list.push(
                                        Breakpoint::new(
                                            app_state.dbg_breakpoint_read,
                                            app_state.dbg_breakpoint_write,
                                            app_state.dbg_breakpoint_execute,
                                            address
                                        )
                                    );
                                }
                            }
                        }
                    }
                });

                Window::new(im_str!("Memory Viewer")).build(&ui, || {
                    let size = ui.calc_text_size(im_str!("FFF"), false, 0.0);
                    let mut clipper = ListClipper::new(0xFFFF / 8).items_height(ui.text_line_height()).begin(&ui);
                    clipper.step();

                    for line in clipper.display_start()..clipper.display_end() {
                        let mut values = Vec::with_capacity(8);
                        let mut current_addr = line as u16 * 8;

                        for _ in 0..8 {
                            values.push(gb_mem_ui.read(current_addr));
                            current_addr += 1;
                        }

                        ui.text(format!("{:04X} | ", current_addr - 8));

                        ui.same_line(0.0);

                        for (idx, value) in values.iter().enumerate() {
                            let token = ui.push_id(&format!("value{}", idx));
                            let value_address = (current_addr - 8) + idx as u16;

                            if app_state.mem_viewer_edit_byte_active && app_state.mem_viewer_edit_byte_address == value_address {
                                let mut flags = ImGuiInputTextFlags::empty();

                                flags.set(ImGuiInputTextFlags::CharsHexadecimal, true);
                                flags.set(ImGuiInputTextFlags::EnterReturnsTrue, true);
                                flags.set(ImGuiInputTextFlags::AutoSelectAll, true);
                                flags.set(ImGuiInputTextFlags::NoHorizontalScroll, true);
                                flags.set(ImGuiInputTextFlags::AlwaysInsertMode, true);
                                
                                ui.set_next_item_width(size[0]);

                                if ui.input_text(im_str!("##data"), &mut app_state.mem_viewer_edit_byte_value).flags(flags).resize_buffer(true).build() {
                                    if let Ok(value) = u8::from_str_radix(&app_state.mem_viewer_edit_byte_value.to_string(), 16) {
                                        gb_mem_ui.dbg_write(value_address, value);
                                    }

                                    app_state.mem_viewer_edit_byte_address = 0;
                                    app_state.mem_viewer_edit_byte_active = false;
                                    app_state.mem_viewer_edit_byte_value = ImString::new("");
                                }
                            }
                            else {
                                if Selectable::new(&ImString::from(format!("{:02X}", value))).allow_double_click(true).size(size).build(&ui) {
                                    app_state.mem_viewer_edit_byte_active = true;
                                    app_state.mem_viewer_edit_byte_value = ImString::from(format!("{:02X}", value));
                                    app_state.mem_viewer_edit_byte_address = (current_addr - 8) + idx as u16;
                                }
                            }

                            token.pop(&ui);
                            ui.same_line(0.0);
                        }

                        ui.text(" | ");
                        ui.same_line(0.0);

                        for (idx, value) in values.iter().enumerate() {
                            let value = *value as char;
                            let size = ui.calc_text_size(im_str!("F"), false, 0.0);
                            if Selectable::new(&ImString::from(format!("{}", value))).allow_double_click(true).size(size).build(&ui) {

                            }

                            if idx != values.len() - 1 {
                                ui.same_line(0.0);
                            }
                        }
                    }

                    clipper.end();
                });

                Window::new(im_str!("Disassembler")).build(&ui, || {
                    let mut clipper = ListClipper::new(0xFFFF).items_height(ui.text_line_height()).begin(&ui);
                    clipper.step();

                    let mut skipped_lines = 0;
                    let mut last_instruction_len = 0;

                    for line in clipper.display_start()..clipper.display_end() {
                        if skipped_lines == last_instruction_len {
                            let current_addr = line as u16;
                            let (len, dis) = disassembler::get_instruction_data(current_addr, &gb_mem_ui);

                            let line_p = if pc_ui == current_addr {"> "} else {""};
                            let line_str = format!("{}{:04X}: {}", line_p, current_addr, dis);

                            skipped_lines = 1;
                            last_instruction_len = len;

                            let mut bp_idx = 0;
                            let mut address_is_bp = false;

                            if let Ok(lock) = gb.read() {
                                for (idx, bp) in lock.dbg_breakpoint_list.iter().enumerate() {
                                    if current_addr == *bp.address() && *bp.execute() {
                                        bp_idx = idx;
                                        address_is_bp = true;

                                        break;
                                    }
                                }
                            }

                            let entry = || if Selectable::new(&ImString::from(line_str)).allow_double_click(true).build(&ui) {
                                if ui.is_mouse_double_clicked(MouseButton::Left) {
                                    if let Ok(mut lock) = gb.write() {
                                        if address_is_bp {
                                            lock.dbg_breakpoint_list.remove(bp_idx);
                                        }
                                        else {
                                            lock.dbg_breakpoint_list.push(
                                                Breakpoint::new(false, false, true, current_addr)
                                            );
                                        }
                                    }
                                }
                            };

                            if address_is_bp {
                                let token = ui.push_style_color(StyleColor::Text, [1.0, 0.0, 0.0, 1.0]);

                                (entry)();

                                token.pop(&ui);
                            }
                            else if pc_ui == current_addr {
                                let token = ui.push_style_color(StyleColor::Text, [0.0, 1.0, 0.0, 1.0]);

                                (entry)();

                                token.pop(&ui);
                            }
                            else {
                                (entry)();
                            }
                        }
                        else {
                            skipped_lines += 1;
                        }
                    }

                    clipper.end();
                });

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
                if let Some(keycode) = input.virtual_keycode {
                    if keycode == glutin::event::VirtualKeyCode::Escape {
                        app_state.mem_viewer_edit_byte_active = false;
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
