mod windows;

use std::sync::{Arc, RwLock};

use imgui::*;

use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};

use glium::glutin;
use glium::{Display, Surface};
use glium::glutin::event::{Event, WindowEvent};
use glium::glutin::event_loop::{ControlFlow, EventLoop};

use windows::*;

use crate::gameboy::Gameboy;
use crate::gameboy::memory::GameboyMemory;

pub fn draw_windows(gb: Arc<RwLock<Gameboy>>, gb_mem: Arc<GameboyMemory>, gb_serial: Arc<RwLock<Vec<u8>>>) {
    let gb = gb;
    let gb_mem = gb_mem;
    let callstack = gb.read().unwrap().ui_get_callstack();

    let event_loop = EventLoop::new();
    let glutin_context = glutin::ContextBuilder::new().with_vsync(true);
    let window_builder = glutin::window::WindowBuilder::new()
        .with_title("rusty-boy")
        .with_inner_size(glutin::dpi::LogicalSize::new(1280, 600))
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

    let mut win_cpu = window_cpu::CPUWindow::init(gb.clone(), callstack);
    let win_cart = window_cart::CartWindow::init(gb.clone(), gb_mem.clone());
    let win_serial = window_serial::SerialWindow::init(gb_serial);
    let mut win_memory = window_memory::MemoryWindow::init(gb_mem.clone());
    let mut win_disassembler = window_disassembler::DisassemblerWindow::init(gb, gb_mem);

    event_loop.run(move | event, _, control_flow| {
        match event {
            Event::MainEventsCleared => {
                let gl_window = display.gl_window();

                winit_platform.prepare_frame(imgui_ctx.io_mut(), &gl_window.window()).unwrap();
                gl_window.window().request_redraw();
            }
            Event::RedrawRequested(_) => {
                let ui = imgui_ctx.frame();
                let adjust = win_cpu.draw(&ui);

                win_cart.draw(&ui);
                win_serial.draw(&ui);
                win_memory.draw(&ui);
                win_disassembler.draw(&ui, adjust);

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
                        //app_state.memory_viewer.editing_byte = false;
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
