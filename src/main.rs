mod gameboy;

use std::fs;
use std::sync::{Arc, RwLock};

use gameboy::*;
use gameboy::disassembler;
use gameboy::memory::regions::*;
use gameboy::memory::GameboyMemory;

use clap::{Arg, App};

use imgui::*;

use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};

use glium::glutin;
use glium::{Display, Surface};
use glium::glutin::event::{Event, WindowEvent};
use glium::glutin::event_loop::{ControlFlow, EventLoop};

#[derive(Default)]
struct AppState {
    debugger: DebuggerState,
    disassembler: DisassemblerState,
    memory_viewer: MemoryViewerState
}

#[derive(Default)]
struct DebuggerState {
    add_bp_read: bool,
    add_bp_write: bool,
    add_bp_execute: bool,
    add_bp_address: ImString,

    edit_bp_read: bool,
    edit_bp_write: bool,
    edit_bp_execute: bool,
    edit_bp_address: ImString,
    edit_bp_selected: usize,
    edit_bp_popup_opened: bool,
}

#[derive(Default)]
struct DisassemblerState {
    should_adjust_cursor: bool
}

#[derive(Default)]
struct MemoryViewerState {
    editing_byte: bool,
    target_byte_address: u16,
    target_byte_new_value: ImString
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

    let serial = gb.read().unwrap().ui_get_serial_output();

    std::thread::spawn(move || {
        let gameboy = gb_ui;

        loop {
            if let Ok(mut lock) = gameboy.try_write() {
                if lock.dbg_mode == EmulatorMode::Running {
                    lock.gb_cpu_cycle();
                }
                else if lock.dbg_mode == EmulatorMode::Stepping && lock.dbg_do_step {
                    lock.gb_cpu_cycle();
                    lock.dbg_do_step = false;
                }
            }
        }
    });

    let mut app_state = AppState::default();

    event_loop.run(move | event, _, control_flow| {
        match event {
            Event::MainEventsCleared => {
                let gl_window = display.gl_window();

                winit_platform.prepare_frame(imgui_ctx.io_mut(), &gl_window.window()).unwrap();
                gl_window.window().request_redraw();
            }
            Event::RedrawRequested(_) => {
                let ui = imgui_ctx.frame();
                let mut pc_ui = 0;

                Window::new(im_str!("Cartridge Info")).build(&ui, || {
                    if let Ok(lock) = gb.read() {
                        let header = lock.ui_get_header();

                        ui.text(format!("Cartridge Title: {}", header.title()));
                        ui.text(format!("Cartridge Controller: {}", header.cart_type()));
                    
                        ui.separator();

                        ui.text(format!("ROM Size: {} ({} banks)", header.rom_size(), header.rom_banks_count()));
                        ui.text(format!("Selected ROM Bank: {}", gb_mem_ui.cartridge().get_selected_rom_bank()));

                        ui.separator();

                        ui.text(format!("RAM Size: {} ({} banks)", header.ram_size(), header.ram_banks_count()));
                        ui.text(format!("RAM Access Enabled: {}", gb_mem_ui.cartridge().is_ram_enabled()));
                        ui.text(format!("Selected RAM Bank: {}", gb_mem_ui.cartridge().get_selected_rom_bank()));
                    }
                });

                Window::new(im_str!("CPU Debugger")).build(&ui, || {
                    if let Ok(mut lock) = gb.write() {
                        let (af, bc, de, hl, sp, pc) = lock.ui_get_cpu_registers();

                        pc_ui = *pc;

                        ui.columns(2, im_str!("cpu_cols"), true);

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

                        ui.next_column();

                        ui.bullet_text(im_str!("CPU Flags"));

                        ui.text(format!("ZF: {}", (af & 0x80) != 0));
                        ui.same_line(0.0);
                        ui.text(format!("NF: {}", (af & 0x40) != 0));
                        
                        ui.text(format!("HF: {}", (af & 0x20) != 0));
                        ui.same_line(0.0);
                        ui.text(format!("CF: {}", (af & 0x10) != 0));

                        ui.columns(1, im_str!("cpu_cols"), false);
    
                        ui.separator();
    
                        ui.bullet_text(im_str!("CPU Controls"));
                        ui.bullet_text(&ImString::from(format!("Status: {}", lock.dbg_mode)));
    
                        if lock.dbg_mode == EmulatorMode::Running {
                            if ui.button(im_str!("Pause"), [0.0, 0.0]) {
                                lock.dbg_mode = EmulatorMode::Paused;
                            }
                        }
                        else if ui.button(im_str!("Resume"), [0.0, 0.0]) {
                            lock.dbg_mode = EmulatorMode::Running;
                            app_state.disassembler.should_adjust_cursor = false;
                        }
    
                        ui.same_line(0.0);
    
                        if ui.button(im_str!("Step"), [0.0, 0.0]) {
                            lock.dbg_mode = EmulatorMode::Stepping;
                            lock.dbg_do_step = true;
                        }

                        ui.same_line(0.0);

                        if ui.button(im_str!("Reset"), [0.0, 0.0]) {
                            lock.gb_reset();
                        }

                        ui.same_line(0.0);

                        if ui.button(im_str!("Skip bootrom"), [0.0, 0.0]) {
                            lock.gb_skip_bootrom();
                        }
    
                        ui.separator();
    
                        ui.bullet_text(im_str!("CPU Breakpoints"));
                        ListBox::new(im_str!("")).size([220.0, 70.0]).build(&ui, || {
                            for (idx, bp) in lock.dbg_breakpoint_list.iter().enumerate() {
                                let bp_string = format!("{:04X} - {}{}{}",
                                    bp.address(),
                                    if *bp.read() {"r"} else {""},
                                    if *bp.write() {"w"} else {""},
                                    if *bp.execute() {"x"} else {""},
                                );
    
                                if Selectable::new(&ImString::from(bp_string)).allow_double_click(true).build(&ui) {
                                    if ui.is_mouse_double_clicked(MouseButton::Left) {
                                        app_state.debugger.edit_bp_read = *bp.read();
                                        app_state.debugger.edit_bp_write = *bp.write();
                                        app_state.debugger.edit_bp_execute = *bp.execute();
                                        app_state.debugger.edit_bp_address = ImString::new(format!("{:04X}", bp.address()));

                                        app_state.debugger.edit_bp_selected = idx;
                                        app_state.debugger.edit_bp_popup_opened = true;
                                    }
                                }
                            }
                        });

                        if app_state.debugger.edit_bp_popup_opened {
                            ui.open_popup(im_str!("Edit breakpoint"));
                            ui.popup_modal(im_str!("Edit breakpoint")).build(|| {
                                ui.input_text(im_str!("Address"), &mut app_state.debugger.edit_bp_address).resize_buffer(true).build();
                                ui.separator();

                                ui.checkbox(im_str!("Read"), &mut app_state.debugger.edit_bp_read);
                                ui.same_line(0.0);
                                ui.checkbox(im_str!("Write"), &mut app_state.debugger.edit_bp_write);
                                ui.same_line(0.0);
                                ui.checkbox(im_str!("Execute"), &mut app_state.debugger.edit_bp_execute);

                                ui.separator();

                                if ui.button(im_str!("Save"), [0.0, 0.0]) {
                                    if let Some(bp) = lock.dbg_breakpoint_list.get_mut(app_state.debugger.edit_bp_selected) {
                                        if let Ok(address) = u16::from_str_radix(&app_state.debugger.edit_bp_address.to_string(), 16) {
                                            bp.set_address(address);
                                        }

                                        bp.set_read(app_state.debugger.edit_bp_read);
                                        bp.set_write(app_state.debugger.edit_bp_write);
                                        bp.set_execute(app_state.debugger.edit_bp_execute);
                                    }

                                    app_state.debugger.edit_bp_popup_opened = false;
                                }

                                ui.same_line(0.0);

                                if ui.button(im_str!("Remove"), [0.0, 0.0]) {
                                    lock.dbg_breakpoint_list.remove(app_state.debugger.edit_bp_selected);
                                    app_state.debugger.edit_bp_popup_opened = false;
                                }

                                ui.same_line(0.0);

                                if ui.button(im_str!("Cancel"), [0.0, 0.0]) {
                                    app_state.debugger.edit_bp_popup_opened = false;
                                }
                            });
                        }
    
                        let submitted_input: bool;
                        let submitted_button: bool;
    
                        submitted_input = ui.input_text(im_str!(""), &mut app_state.debugger.add_bp_address).enter_returns_true(true).build();
                        ui.same_line(0.0);
                        submitted_button = ui.button(im_str!("Add"), [0.0, 0.0]);
    
                        ui.checkbox(im_str!("Read"), &mut app_state.debugger.add_bp_read);
                        ui.same_line(0.0);
                        ui.checkbox(im_str!("Write"), &mut app_state.debugger.add_bp_write);
                        ui.same_line(0.0);
                        ui.checkbox(im_str!("Execute"), &mut app_state.debugger.add_bp_execute);
    
                        if submitted_input || submitted_button {
                            let valid_bp = (app_state.debugger.add_bp_read || 
                                app_state.debugger.add_bp_write || app_state.debugger.add_bp_execute) && 
                                !app_state.debugger.add_bp_address.is_empty()
                            ;

                            if valid_bp {
                                if let Ok(address) = u16::from_str_radix(&app_state.debugger.add_bp_address.to_string(), 16) {
                                    lock.dbg_breakpoint_list.push(
                                        Breakpoint::new(
                                            app_state.debugger.add_bp_read,
                                            app_state.debugger.add_bp_write,
                                            app_state.debugger.edit_bp_execute,
                                            address
                                        )
                                    );
                                }
                            }
                        }
                    }
                });

                Window::new(im_str!("Memory Viewer")).build(&ui, || {
                    let style_padding = ui.push_style_var(StyleVar::FramePadding([0.0, 0.0]));
                    let style_spacing = ui.push_style_var(StyleVar::ItemSpacing([5.0, 1.0]));

                    let size = ui.calc_text_size(im_str!("FF"), false, 0.0);
                    let mut clipper = ListClipper::new(0xFFFF / 8).items_height(ui.text_line_height() / 2.0).begin(&ui);
                    clipper.step();

                    for line in clipper.display_start()..clipper.display_end() {
                        let mut values = Vec::with_capacity(8);
                        let mut current_addr = line as u16 * 8;

                        for _ in 0..8 {
                            values.push(gb_mem_ui.read(current_addr));
                            current_addr += 1;
                        }

                        ui.text(format!("{:04X} |", current_addr - 8));

                        ui.same_line(0.0);

                        for (idx, value) in values.iter().enumerate() {
                            let token = ui.push_id(&format!("value{}", idx));
                            let value_address = (current_addr - 8) + idx as u16;

                            if app_state.memory_viewer.editing_byte && app_state.memory_viewer.target_byte_address == value_address {
                                let mut flags = ImGuiInputTextFlags::empty();

                                flags.set(ImGuiInputTextFlags::CharsHexadecimal, true);
                                flags.set(ImGuiInputTextFlags::EnterReturnsTrue, true);
                                flags.set(ImGuiInputTextFlags::AutoSelectAll, true);
                                flags.set(ImGuiInputTextFlags::NoHorizontalScroll, true);
                                flags.set(ImGuiInputTextFlags::AlwaysInsertMode, true);
                                
                                ui.set_next_item_width(size[0]);

                                if ui.input_text(im_str!("##data"), &mut app_state.memory_viewer.target_byte_new_value).flags(flags).resize_buffer(true).build() {
                                    if let Ok(value) = u8::from_str_radix(&app_state.memory_viewer.target_byte_new_value.to_string(), 16) {
                                        gb_mem_ui.dbg_write(value_address, value);
                                    }

                                    app_state.memory_viewer.editing_byte = false;
                                    app_state.memory_viewer.target_byte_address = 0;
                                    app_state.memory_viewer.target_byte_new_value = ImString::new("");
                                }
                            }
                            else if Selectable::new(&ImString::from(format!("{:02X}", value))).allow_double_click(true).size(size).build(&ui) {
                                app_state.memory_viewer.editing_byte = true;
                                app_state.memory_viewer.target_byte_address = (current_addr - 8) + idx as u16;
                                app_state.memory_viewer.target_byte_new_value = ImString::from(format!("{:02X}", value));
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

                    style_padding.pop(&ui);
                    style_spacing.pop(&ui);
                });

                Window::new(im_str!("Disassembler")).build(&ui, || {
                    let mut clipper = ListClipper::new(0xFFFF).items_height(ui.text_line_height() / 2.0).begin(&ui);
                    clipper.step();

                    let mut skipped_lines = 0;
                    let mut last_instruction_len = 0;

                    for line in clipper.display_start()..clipper.display_end() {
                        if skipped_lines == last_instruction_len {
                            let current_addr = line as u16;
                            let (len, dis) = disassembler::get_instruction_data(current_addr, &gb_mem_ui);

                            let line_p = if pc_ui == current_addr {"> "} else {""};
                            let address_p = {
                                if CARTRIDGE_ROM_BANKX.contains(&current_addr) {
                                    String::from("ROM0")
                                }
                                else if CARTRIDGE_ROM_BANKX.contains(&current_addr) {
                                    format!("ROM{:0X}", gb_mem_ui.cartridge().get_selected_rom_bank())
                                }
                                else if VRAM.contains(&current_addr) {
                                    String::from("VRAM")
                                }
                                else if CARTRIDGE_RAM.contains(&current_addr) {
                                    String::from("CRAM")
                                }
                                else if WRAM.contains(&current_addr) {
                                    String::from("WRAM")
                                }
                                else if OAM.contains(&current_addr) {
                                    String::from("OAM")
                                }
                                else if (0xFEA0..=0xFEFF).contains(&current_addr) {
                                    String::from("UNK")
                                }
                                else if IO.contains(&current_addr) {
                                    String::from("IO")
                                }
                                else if HRAM.contains(&current_addr) {
                                    String::from("HRAM")
                                }
                                else {
                                    String::from("IE")
                                }
                            };
                            let line_str = format!("{}{}: {:04X} - {}", line_p, address_p, current_addr, dis);

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

                    if let Ok(lock) = gb.read() {
                        match lock.dbg_mode {
                            EmulatorMode::Paused | EmulatorMode::BreakpointHit | EmulatorMode::UnknownInstruction(..) => {
                                if !app_state.disassembler.should_adjust_cursor {
                                    let target = ui.cursor_start_pos()[1] + pc_ui as f32 * (ui.text_line_height() / 2.0);

                                    app_state.disassembler.should_adjust_cursor = true;
                                    ui.set_scroll_from_pos_y(target);
                                }
                                
                            }
                            _ => {}
                        }
                    }
                });

                Window::new(im_str!("Serial Output")).build(&ui, || {
                    if let Ok(lock) = serial.read() {
                        let mut output = String::new();

                        for b in lock.iter() {
                            let c = *b as char;
                            output.push(c);
                        }

                        ListBox::new(im_str!("")).size([220.0, 70.0]).build(&ui, || {
                            for line in output.lines().rev() {
                                Selectable::new(&ImString::from(line.to_string())).build(&ui);
                            }
                        });
                    }
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
                        app_state.memory_viewer.editing_byte = false;
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
