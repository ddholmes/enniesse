use std::env;

extern crate nesrs;
use nesrs::emu;
use nesrs::rom;

fn main() {
    let mut args = env::args();
    let rom_file_name = args.nth(1).unwrap();
    
    let rom = rom::Rom::from_file(rom_file_name);

    let mut emu = emu::Emu::new(rom);
    emu.start();
}

