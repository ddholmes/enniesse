use std::env;
use std::fs;
use std::io::Read;
use std::path::Path;

extern crate regex;

mod cpu;
mod nes;
mod rom;
mod memory;
mod mapper;

fn main() {
    let mut args = env::args();
    let rom_file_name = args.nth(1).unwrap();
    let rom_buf = read_file(rom_file_name);
    
    let rom = rom::Rom::from(rom_buf);
    
    let mut nes = nes::Nes::new(Box::new(rom));
    nes.power_on();
}

fn read_file<P: AsRef<Path>>(path: P) -> Box<[u8]> {
    let mut rom_file = fs::File::open(path).unwrap();
    let mut rom_buf = Vec::<u8>::new();
    let _ = rom_file.read_to_end(&mut rom_buf);
    
    rom_buf.into_boxed_slice()
}

