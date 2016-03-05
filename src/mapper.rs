use rom::Rom;

pub trait Mapper {
    fn load_byte_prg(&mut self, addr: u16) -> u8;
    fn store_byte_prg(&mut self, addr: u16, val: u8);
    fn load_byte_chr(&mut self, addr: u16) -> u8;
    fn store_byte_chr(&mut self, addr: u16, val: u8);
}

pub fn load_mapper(rom: Box<Rom>) -> Box<Mapper> {
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
    fn store_byte_prg(&mut self, _: u16, _: u8) {}
    
    fn load_byte_chr(&mut self, addr: u16) -> u8 {
        self.rom.chr_rom[addr as usize]
    }
    fn store_byte_chr(&mut self, _: u16, _: u8) {}
}