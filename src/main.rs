use std::env;

extern crate nesrs;
use nesrs::nes;
use nesrs::rom;

fn main() {
    let mut args = env::args();
    let rom_file_name = args.nth(1).unwrap();
    
    let rom = rom::Rom::from_file(rom_file_name);
    
    let mut nes = nes::Nes::new(Box::new(rom));
    nes.power_on();
}

