use std::env;
use std::fs;
use std::io::Read;
use std::path::Path;

extern crate byteorder;

mod cpu;
mod nes;
mod rom;
mod memory;
mod mapper;

fn main() {
    let args = env::args().collect::<Vec<String>>();
    
    let rom_file_name = &args[1];
    let rom_buf = read_file(rom_file_name);
    
    let rom = rom::Rom::from(rom_buf);
    
    //println!("{}", rom.mapper);
    
    let mut nes = nes::Nes::new(Box::new(rom));
    nes.power_on();
    
    println!("{:#?}", nes);
}

fn read_file<P: AsRef<Path>>(path: P) -> Box<[u8]> {
    let mut rom_file = fs::File::open(path).unwrap();
    let mut rom_buf = Vec::<u8>::new();
    rom_file.read_to_end(&mut rom_buf);
    
    rom_buf.into_boxed_slice()
}

