use rom::Rom;
use memory::MemoryRegion;

pub trait Mapper {
    fn load_byte_prg(&mut self, addr: u16) -> u8;
    fn store_byte_prg(&mut self, addr: u16, val: u8);
    fn load_byte_chr(&mut self, addr: u16) -> u8;
    fn store_byte_chr(&mut self, addr: u16, val: u8);
}

impl<T> MemoryRegion for T where T: Mapper {
    fn load_byte(&mut self, addr: u16) -> u8 {
        self.load_byte_prg(addr)
    }
    
    fn store_byte(&mut self, addr: u16, val: u8) {
        self.store_byte_prg(addr, val);
    }
}

pub fn load_mapper(rom: Box<Rom>) -> Box<MemoryRegion> {
    match rom.mapper {
        0 => Box::new(Nrom::new(rom)),
        _ => panic!("Unknown mapper: {}", rom.mapper)
    }
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
            self.ram[(addr - 0x6000) as usize]
        } else if self.rom.prg_rom.len() > 16384 {
            // max size is 32k
            self.rom.prg_rom[addr as usize & 0x7fff]

        } else {
            // 16k
            self.rom.prg_rom[addr as usize & 0x3fff]
        }
    }
    fn store_byte_prg(&mut self, addr: u16, val: u8) {}
    
    fn load_byte_chr(&mut self, addr: u16) -> u8 {
        0
    }
    fn store_byte_chr(&mut self, addr: u16, val: u8) {}
}