use std::env;

extern crate minifb;
extern crate nesrs;

mod emu;

fn main() {
    let mut args = env::args();
    let rom_file_name = args.nth(1).unwrap();
    
    let mut emu = emu::Emu::new(rom_file_name);
    emu.start();
}

