use std::sync::{Arc, RwLock};

use imgui::*;

use crate::gameboy::{Breakpoint, EmulatorMode, Gameboy};

pub struct CPUWindow {
    gb: Arc<RwLock<Gameboy>>,
    callstack: Arc<RwLock<Vec<String>>>,

    registers: [u16; 6],
    dbg_mode: EmulatorMode,
    callstack_items: Vec<ImString>,
    breakpoints_list: Vec<Breakpoint>,

    bp_add_read: bool,
    bp_add_write: bool,
    bp_add_execute: bool,
    bp_add_address: ImString,

    bp_edit_idx: usize,
    bp_edit_read: bool,
    bp_edit_write: bool,
    bp_edit_execute: bool,
    bp_edit_address: ImString,
    bp_edit_popup_open: bool
}

impl CPUWindow {
    pub fn init(gb: Arc<RwLock<Gameboy>>) -> CPUWindow {
        let callstack = gb.read().unwrap().ui_get_callstack();

        CPUWindow {
            gb,
            callstack,

            registers: [0, 0, 0, 0, 0, 0],
            dbg_mode: EmulatorMode::Paused,
            callstack_items: Vec::new(),
            breakpoints_list: Vec::new(),

            bp_add_read: false,
            bp_add_write: false,
            bp_add_execute: false,
            bp_add_address: ImString::new(""),
        
            bp_edit_idx: 0,
            bp_edit_read: false,
            bp_edit_write: false,
            bp_edit_execute: false,
            bp_edit_address: ImString::new(""),
            bp_edit_popup_open: false
        }
    }

    pub fn draw(&mut self, ui: &Ui) -> bool {
        let mut adjust_cursor = false;

        Window::new(im_str!("CPU Debugger")).build(ui, || {
            if ui.is_window_focused() {
                if let Ok(lock) = self.gb.try_read() {
                    let (af, bc, de, hl, sp, pc) = lock.ui_get_cpu_registers();
                    let mut breakpoints_list = Vec::with_capacity(lock.dbg_breakpoint_list.len());

                    self.registers[0] = *af;
                    self.registers[1] = *bc;
                    self.registers[2] = *de;
                    self.registers[3] = *hl;
                    self.registers[4] = *sp;
                    self.registers[5] = *pc;

                    self.dbg_mode = lock.dbg_mode.clone();

                    for bp in lock.dbg_breakpoint_list.iter() {
                        breakpoints_list.push(bp.clone());
                    }

                    self.breakpoints_list = breakpoints_list;
                }

                if let Ok(lock) = self.callstack.try_read() {
                    let mut callstack_items = Vec::with_capacity(lock.len());

                    for call in lock.iter().rev() {
                        callstack_items.push(ImString::from(call.clone()));
                    }

                    self.callstack_items = callstack_items;
                }
            }

            ui.columns(2, im_str!("cpu_cols"), true);

            ui.bullet_text(im_str!("CPU Registers"));
            
            ui.text(format!("AF: {:04X}", self.registers[0]));
            ui.same_line(0.0);
            ui.text(format!("BC: {:04X}", self.registers[1]));
            
            ui.text(format!("DE: {:04X}", self.registers[2]));
            ui.same_line(0.0);
            ui.text(format!("HL: {:04X}", self.registers[3]));

            ui.text(format!("SP: {:04X}", self.registers[4]));
            ui.same_line(0.0);
            ui.text(format!("PC: {:04X}", self.registers[5]));

            ui.next_column();

            ui.bullet_text(im_str!("CPU Flags"));

            ui.text(format!("ZF: {}", (self.registers[0] & 0x80) != 0));
            ui.same_line(0.0);
            ui.text(format!("NF: {}", (self.registers[0] & 0x40) != 0));
            
            ui.text(format!("HF: {}", (self.registers[0] & 0x20) != 0));
            ui.same_line(0.0);
            ui.text(format!("CF: {}", (self.registers[0] & 0x10) != 0));

            ui.columns(1, im_str!("cpu_cols"), false);

            ui.separator();
            ui.bullet_text(im_str!("CPU Controls"));

            ui.bullet_text(&ImString::from(format!("Status: {}", self.dbg_mode)));

            if self.dbg_mode == EmulatorMode::Running {
                if ui.button(im_str!("Pause"), [0.0, 0.0]) {
                    adjust_cursor = true;

                    if let Ok(mut lock) = self.gb.write() {
                        self.dbg_mode = EmulatorMode::Paused;
                        lock.dbg_mode = EmulatorMode::Paused;
                    }
                }
            }
            else if ui.button(im_str!("Resume"), [0.0, 0.0]) {
                adjust_cursor = true;
                
                if let Ok(mut lock) = self.gb.write() {
                    self.dbg_mode = EmulatorMode::Running;
                    lock.dbg_mode = EmulatorMode::Running;
                }
            }

            ui.same_line(0.0);

            if ui.button(im_str!("Step"), [0.0, 0.0]) {
                if let Ok(mut lock) = self.gb.write() {
                    lock.dbg_do_step = true;
                    self.dbg_mode = EmulatorMode::Stepping;
                    lock.dbg_mode = EmulatorMode::Stepping;
                }
            }

            ui.same_line(0.0);

            if ui.button(im_str!("Reset"), [0.0, 0.0]) {
                adjust_cursor = true;

                if let Ok(mut lock) = self.gb.write() {
                    lock.gb_reset();
                }
            }

            ui.same_line(0.0);

            if ui.button(im_str!("Skip bootrom"), [0.0, 0.0]) {
                adjust_cursor = true;

                if let Ok(mut lock) = self.gb.write() {
                    lock.gb_skip_bootrom();
                }
            }

            ui.separator();
            ui.bullet_text(im_str!("CPU Breakpoints"));

            ListBox::new(im_str!("")).size([220.0, 70.0]).build(ui, || {
                for (idx, bp) in self.breakpoints_list.iter().enumerate() {
                    let bp_string = format!("{:04X} - {}{}{}",
                        bp.address(),
                        if *bp.read() {"r"} else {""},
                        if *bp.write() {"w"} else {""},
                        if *bp.execute() {"x"} else {""},
                    );

                    let selected = Selectable::new(&ImString::from(bp_string)).allow_double_click(true).build(ui);

                    if selected && ui.is_mouse_double_clicked(MouseButton::Left) {
                        self.bp_edit_read = *bp.read();
                        self.bp_edit_write = *bp.write();
                        self.bp_edit_execute = *bp.execute();
                        self.bp_edit_address = ImString::new(format!("{:04X}", bp.address()));
                    
                        self.bp_edit_idx = idx;
                        self.bp_edit_popup_open = true;
                    }
                }
            });

            if self.bp_edit_popup_open {
                ui.open_popup(im_str!("Edit breakpoint"));
                ui.popup_modal(im_str!("Edit breakpoint")).build(|| {
                    ui.input_text(im_str!("Address"), &mut self.bp_edit_address).resize_buffer(true).build();
                    ui.separator();

                    ui.checkbox(im_str!("Read"), &mut self.bp_edit_read);
                    ui.same_line(0.0);
                    ui.checkbox(im_str!("Write"), &mut self.bp_edit_write);
                    ui.same_line(0.0);
                    ui.checkbox(im_str!("Execute"), &mut self.bp_edit_execute);

                    ui.separator();

                    if ui.button(im_str!("Save"), [0.0, 0.0]) {
                        if let Ok(mut lock) = self.gb.write() {
                            if let Some(bp) = lock.dbg_breakpoint_list.get_mut(self.bp_edit_idx) {
                                if let Ok(address) = u16::from_str_radix(&self.bp_edit_address.to_string(), 16) {
                                    bp.set_address(address);
                                }
    
                                bp.set_read(self.bp_edit_read);
                                bp.set_write(self.bp_edit_write);
                                bp.set_execute(self.bp_edit_execute);
                            }
    
                            self.bp_edit_popup_open = false;
                        }
                    }

                    ui.same_line(0.0);

                    if ui.button(im_str!("Remove"), [0.0, 0.0]) {
                        if let Ok(mut lock) = self.gb.write() {
                            lock.dbg_breakpoint_list.remove(self.bp_edit_idx);
                            self.bp_edit_popup_open = false;
                        }
                    }

                    ui.same_line(0.0);

                    if ui.button(im_str!("Cancel"), [0.0, 0.0]) {
                        self.bp_edit_popup_open = false;
                    }
                });
            }

            let submitted_input: bool;
            let submitted_button: bool;

            submitted_input = ui.input_text(im_str!(""), &mut self.bp_add_address).enter_returns_true(true).build();
            ui.same_line(0.0);
            submitted_button = ui.button(im_str!("Add"), [0.0, 0.0]);

            ui.checkbox(im_str!("Read"), &mut self.bp_add_read);
            ui.same_line(0.0);
            ui.checkbox(im_str!("Write"), &mut self.bp_add_write);
            ui.same_line(0.0);
            ui.checkbox(im_str!("Execute"), &mut self.bp_add_execute);

            if submitted_input || submitted_button {
                let valid_bp = (self.bp_add_read || self.bp_add_write || self.bp_add_execute) && !self.bp_add_address.is_empty();

                if valid_bp {
                    if let Ok(address) = u16::from_str_radix(&self.bp_add_address.to_string(), 16) {
                        let bp = Breakpoint::new(
                            self.bp_add_read,
                            self.bp_add_write,
                            self.bp_add_execute,
                            address
                        );

                        if let Ok(mut lock) = self.gb.write() {
                            lock.dbg_breakpoint_list.push(bp);
                        }
                    }
                }
            }

            ui.separator();
            ui.bullet_text(im_str!("CPU Callstack"));

            ListBox::new(im_str!("##c")).size([220.0, 70.0]).build(ui, || {
                for call in self.callstack_items.iter() {
                    Selectable::new(call).build(&ui);
                }
            });
        });

        adjust_cursor
    }
}
