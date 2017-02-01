use rom::Rom;

pub trait Mapper {
    fn load_byte_prg(&mut self, addr: u16) -> u8;
    fn store_byte_prg(&mut self, addr: u16, val: u8);
    fn load_byte_chr(&mut self, addr: u16) -> u8;
    fn store_byte_chr(&mut self, addr: u16, val: u8);
    
    fn get_mirroring(&self) -> Mirroring;
}

pub fn load_mapper(rom: Box<Rom>) -> Box<Mapper> {
    match rom.mapper {
        0 => Box::new(Nrom::new(rom)),
        _ => panic!("Unknown mapper: {}", rom.mapper)
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Mirroring {
    Horizontal,
    Vertical
}

const NROM_RAM_SIZE: usize = 4096;

pub struct Nrom {
    rom: Box<Rom>,
    ram: [u8; NROM_RAM_SIZE]
}

impl Nrom {
    pub fn new(rom: Box<Rom>) -> Nrom {
        Nrom {
            rom: rom,
            ram: [0; NROM_RAM_SIZE]
        }
    }
}

impl Mapper for Nrom {
    fn load_byte_prg(&mut self, addr: u16) -> u8 {
        if addr < 0x8000 {
            self.ram[(addr & 0x0fff) as usize]
        } else if self.rom.prg_rom.len() > 16384 {
            // max size is 32k
            self.rom.prg_rom[addr as usize & 0x7fff]
        } else {
            // 16k
            self.rom.prg_rom[addr as usize & 0x3fff]
        }
    }
    fn store_byte_prg(&mut self, _: u16, _: u8) {}
    
    fn load_byte_chr(&mut self, addr: u16) -> u8 {
        if self.rom.chr_rom_size != 0 {
            self.rom.chr_rom[addr as usize]
        } else {
            self.ram[addr as usize & NROM_RAM_SIZE - 1]
        }
    }
    fn store_byte_chr(&mut self, addr: u16, val: u8) {
        if self.rom.chr_rom_size == 0 {
            self.ram[addr as usize & NROM_RAM_SIZE - 1] = val;
        }
    }
    
    fn get_mirroring(&self) -> Mirroring {
        if self.rom.flags6 & 1 == 0 {
            Mirroring::Horizontal
        } else {
            Mirroring::Vertical
        }
    }
}