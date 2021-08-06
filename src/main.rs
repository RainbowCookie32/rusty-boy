mod ui;
mod gameboy;

use std::fs;
use std::sync::{Arc, RwLock};

use gameboy::memory::GameboyMemory;
use gameboy::{Gameboy, EmulatorMode, JoypadHandler};

use clap::{Arg, App};

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

    let bootrom_data = fs::read(bootrom_path).expect("Couldn't read bootrom file at path");
    let romfile_data = fs::read(romfile_path).expect("Couldn't read Gameboy romfile at path");

    let gb_joy = Arc::new(RwLock::new(JoypadHandler::default()));
    let gb_mem = Arc::from(GameboyMemory::init(bootrom_data, romfile_data, gb_joy.clone()));
    let gb = Arc::from(RwLock::from(Gameboy::init(gb_mem.clone(), gb_joy)));
    
    let gb_ui = gb.clone();
    let gb_mem_ui = gb_mem;
    let gb_serial = gb.read().unwrap().ui_get_serial_output();

    std::thread::spawn(move || {
        let gameboy = gb;

        loop {
            if let Ok(mut lock) = gameboy.try_write() {
                if lock.dbg_mode == EmulatorMode::Running {
                    lock.gb_cpu_cycle();
                    lock.gb_gpu_cycle();
                }
                else if lock.dbg_mode == EmulatorMode::Stepping && lock.dbg_do_step {
                    lock.gb_cpu_cycle();
                    lock.gb_gpu_cycle();
                    lock.dbg_do_step = false;
                }
            }
        }
    });

    ui::run_app(gb_ui, gb_mem_ui, gb_serial);
}
