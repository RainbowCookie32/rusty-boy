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

    bp_add_addr: String,
    bp_edit_addr: String,
    bp_edit_show_popup: bool,

    bp_add: (usize, Breakpoint),
    bp_edit: (usize, Breakpoint)
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

            bp_add_addr: String::new(),
            bp_edit_addr: String::new(),
            bp_edit_show_popup: false,

            bp_add: (0, Breakpoint::new(false, false, false, 0xFFFF)),
            bp_edit: (0, Breakpoint::new(false, false, false, 0xFFFF))
        }
    }

    pub fn draw(&mut self, ui: &Ui, opened: &mut bool) -> bool {
        if !*opened {
            return false;
        }

        let mut adjust_cursor = false;

        Window::new("CPU Debugger").size([290.0, 400.0], Condition::FirstUseEver).opened(opened).build(ui, || {
            if ui.is_window_focused() {
                if let Ok(lock) = self.gb.read() {
                    let (af, bc, de, hl, sp, pc) = lock.ui_get_cpu_registers();
                    let mut breakpoints_list = Vec::with_capacity(lock.dbg_breakpoint_list.len());

                    self.registers[0] = af;
                    self.registers[1] = bc;
                    self.registers[2] = de;
                    self.registers[3] = hl;
                    self.registers[4] = sp;
                    self.registers[5] = pc;

                    self.dbg_mode = lock.dbg_mode.clone();

                    for bp in lock.dbg_breakpoint_list.iter() {
                        breakpoints_list.push(bp.clone());
                    }

                    self.breakpoints_list = breakpoints_list;
                }

                if let Ok(lock) = self.callstack.read() {
                    let mut callstack_items = Vec::with_capacity(lock.len());

                    for call in lock.iter().rev() {
                        callstack_items.push(ImString::from(call.clone()));
                    }

                    self.callstack_items = callstack_items;
                }
            }

            ui.columns(2, "cpu_cols", true);

            ui.bullet_text("CPU Registers");
            
            ui.text(format!("AF: {:04X}", self.registers[0]));
            ui.same_line();
            ui.text(format!("BC: {:04X}", self.registers[1]));
            
            ui.text(format!("DE: {:04X}", self.registers[2]));
            ui.same_line();
            ui.text(format!("HL: {:04X}", self.registers[3]));

            ui.text(format!("SP: {:04X}", self.registers[4]));
            ui.same_line();
            ui.text(format!("PC: {:04X}", self.registers[5]));

            ui.next_column();

            ui.bullet_text("CPU Flags");

            ui.text(format!("ZF: {}", (self.registers[0] & 0x80) != 0));
            ui.same_line();
            ui.text(format!("NF: {}", (self.registers[0] & 0x40) != 0));
            
            ui.text(format!("HF: {}", (self.registers[0] & 0x20) != 0));
            ui.same_line();
            ui.text(format!("CF: {}", (self.registers[0] & 0x10) != 0));

            ui.columns(1, "cpu_cols", false);

            ui.separator();
            ui.bullet_text("CPU Controls");

            ui.bullet_text(&ImString::from(format!("Status: {}", self.dbg_mode)));

            if self.dbg_mode == EmulatorMode::Running {
                if ui.button("Pause") {
                    adjust_cursor = true;

                    if let Ok(mut lock) = self.gb.write() {
                        self.dbg_mode = EmulatorMode::Paused;
                        lock.dbg_mode = EmulatorMode::Paused;
                    }
                }
            }
            else if ui.button("Resume") {
                adjust_cursor = true;
                
                if let Ok(mut lock) = self.gb.write() {
                    self.dbg_mode = EmulatorMode::Running;
                    lock.dbg_mode = EmulatorMode::Running;
                }
            }

            ui.same_line();

            if ui.button("Step") {
                if let Ok(mut lock) = self.gb.write() {
                    lock.dbg_do_step = true;
                    self.dbg_mode = EmulatorMode::Stepping;
                    lock.dbg_mode = EmulatorMode::Stepping;
                }
            }

            ui.same_line();

            if ui.button("Reset") {
                adjust_cursor = true;

                if let Ok(mut lock) = self.gb.write() {
                    lock.gb_reset();
                }
            }

            ui.separator();
            ui.bullet_text("CPU Breakpoints");

            ListBox::new("").size([220.0, 70.0]).build(ui, || {
                for (idx, bp) in self.breakpoints_list.iter().enumerate() {
                    let bp_string = format!("{:04X} - {}{}{}",
                        bp.address(),
                        if *bp.read() {"r"} else {""},
                        if *bp.write() {"w"} else {""},
                        if *bp.execute() {"x"} else {""},
                    );

                    let selected = Selectable::new(&ImString::from(bp_string)).allow_double_click(true).build(ui);

                    if selected && ui.is_mouse_double_clicked(MouseButton::Left) {
                        self.bp_edit = (idx, bp.clone());
                        self.bp_edit_addr = format!("{:04X}", bp.address());
                        self.bp_edit_show_popup = true;
                    }
                }
            });

            if self.bp_edit_show_popup {
                ui.open_popup("Edit breakpoint");

                if let Some(_token) = PopupModal::new("Edit breakpoint").begin_popup(ui) {
                    ui.input_text("Address", &mut self.bp_edit_addr).build();
                    ui.separator();

                    ui.checkbox("Read", self.bp_edit.1.read_mut());
                    ui.same_line();
                    ui.checkbox("Write", self.bp_edit.1.write_mut());
                    ui.same_line();
                    ui.checkbox("Execute", self.bp_edit.1.execute_mut());

                    ui.separator();

                    if ui.button("Save") {
                        if let Ok(mut lock) = self.gb.write() {
                            if let Some(bp) = lock.dbg_breakpoint_list.get_mut(self.bp_edit.0) {
                                if let Ok(address) = u16::from_str_radix(&self.bp_edit_addr.to_string(), 16) {
                                    self.bp_edit.1.set_address(address);
                                    *bp = self.bp_edit.1.clone();
                                }
                            }

                            self.breakpoints_list[self.bp_edit.0] = self.bp_edit.1.clone();
                            self.bp_edit = (0, Breakpoint::new(false, false, false, 0xFFFF));
                            self.bp_edit_show_popup = false;
                        }
                    }

                    ui.same_line();

                    if ui.button("Remove") {
                        if let Ok(mut lock) = self.gb.write() {
                            lock.dbg_breakpoint_list.remove(self.bp_edit.0);
                            self.bp_edit_show_popup = false;
                        }
                    }

                    ui.same_line();

                    if ui.button("Cancel") {
                        self.bp_edit_show_popup = false;
                    }
                };
            }

            let submitted_input: bool;
            let submitted_button: bool;

            submitted_input = ui.input_text("", &mut self.bp_add_addr).enter_returns_true(true).build();
            ui.same_line();
            submitted_button = ui.button("Add");

            ui.checkbox("Read", self.bp_add.1.read_mut());
            ui.same_line();
            ui.checkbox("Write", self.bp_add.1.write_mut());
            ui.same_line();
            ui.checkbox("Execute", self.bp_add.1.execute_mut());

            if submitted_input || submitted_button {
                let valid_bp = self.bp_add.1.is_valid() && !self.bp_add_addr.is_empty();

                if valid_bp {
                    if let Ok(address) = u16::from_str_radix(&self.bp_add_addr.to_string(), 16) {
                        if let Ok(mut lock) = self.gb.write() {
                            self.bp_add.1.set_address(address);
                            lock.dbg_breakpoint_list.push(self.bp_add.1.clone());
                            self.bp_add = (0, Breakpoint::new(false, false, false, 0xFFFF));
                        }
                    }
                }
            }

            ui.separator();
            ui.bullet_text("CPU Callstack");

            ListBox::new("##c").size([220.0, 70.0]).build(ui, || {
                for call in self.callstack_items.iter() {
                    Selectable::new(call).build(ui);
                }
            });
        });

        adjust_cursor
    }
}
