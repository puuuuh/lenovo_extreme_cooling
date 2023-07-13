use std::fmt::Display;

use clap::{App, SubCommand};

use crate::ec::EmbeddedController;

mod ec;
mod cpuio;

const EXTREME_COOLING_REGISTER: u8 = 0xBD;

#[repr(u8)]
enum CoolingState {
    Active = 0x40,
    Inactive = 0x00,
    Unknown,
}

impl Display for CoolingState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            CoolingState::Active => "active",
            CoolingState::Inactive => "inactive",
            CoolingState::Unknown => "unknown",
        })
    }
}

impl From<u8> for CoolingState {
    fn from(val: u8) -> Self {
        match val {
            0x40 => CoolingState::Active,
            0x00 => CoolingState::Inactive,
            _ => CoolingState::Unknown,
        }
    }
}

fn main() {
    match unsafe {libc::setuid(0)} {
        0 => {},
        _ => {
            eprintln!("Root rights required");
            return;
        }
    }
    let matches = App::new("extreme_cooling")
        .subcommand(SubCommand::with_name("switch").about("Switch extreme cooling status"))
        .subcommand(SubCommand::with_name("enable").about("Enable extreme cooling"))
        .subcommand(SubCommand::with_name("disable").about("Disable extreme cooling"))
        .subcommand(SubCommand::with_name("query").about("Get current status"))
        .get_matches();
    let mut ec = EmbeddedController::new().expect("EC init error");
    let new_state = match matches.subcommand() {
        ("query", _) => {
            let current_state = CoolingState::from(ec.read(EXTREME_COOLING_REGISTER).unwrap());
            println!("{}", current_state);
            return
        },
        ("switch", _) => {
            let current_state = CoolingState::from(ec.read(EXTREME_COOLING_REGISTER).unwrap());

            println!("Current state: {}", current_state);
            
            match current_state {
                CoolingState::Active => CoolingState::Inactive,
                CoolingState::Inactive => CoolingState::Active,
                CoolingState::Unknown => {
                    eprintln!("Unknown current state, abort");
                    return;
                }
            }
        }
        ("enable", _) => CoolingState::Active,
        ("disable", _) => CoolingState::Inactive,
        _ => {
            eprintln!("Unknown subcommand");
            return
        }
    };
    println!("Switch to: {}", new_state);
    ec.write(EXTREME_COOLING_REGISTER, new_state as u8)
        .expect("New state write error");
}
