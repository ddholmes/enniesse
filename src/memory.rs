use super::rom::Rom;
use super::mapper;
use super::apu::Apu;
use super::ppu::Ppu;

const RAM_START: u16 = 0x0000;
const RAM_END: u16 = 0x1fff;
// the 2k is mirrored between 4 2k blocks
const RAM_SIZE: u16 = 2048;

const PPU_REG_START: u16 = 0x2000;
const PPU_REG_END: u16 = 0x3fff;

const APU_REG_START: u16 = 0x4000;
const APU_REG_END: u16 = 0x4015;
const IO_REG: u16 = 0x4016;
const APU_IO_SHARED_REG: u16 = 0x4017;

const CART_MAPPER_START: u16 = 0x4020;
const CART_MAPPER_END: u16 = 0xffff;

pub trait Memory { 
    fn load_byte(&mut self, addr: u16) -> u8;
    fn load_word(&mut self, addr: u16) -> u16 {
        self.load_byte(addr) as u16 | (self.load_byte(addr + 1) as u16) << 8
    }
    fn load_word_zero_page(&mut self, addr: u8) -> u16 {
        self.load_byte(addr as u16) as u16 | (self.load_byte(addr.wrapping_add(1) as u16) as u16) << 8
    }
    
    fn store_byte(&mut self, addr: u16, val: u8);
    fn store_word(&mut self, addr: u16, val: u16) {
        self.store_byte(addr, (val & 0xff) as u8);
        self.store_byte(addr + 1, ((val >> 8) & 0xff) as u8);
    }
}

pub struct MemoryInterface {
    pub ram: Ram,
    pub mapper: Box<mapper::Mapper>,
    pub apu: Apu,
    pub ppu: Ppu
}

impl MemoryInterface {
    pub fn new(rom: Box<Rom>) -> MemoryInterface {
        MemoryInterface {
            ram: Ram::new(),
            mapper: mapper::load_mapper(rom),
            apu: Apu::new(),
            ppu: Ppu::new()
        }
    }
}

impl Memory for MemoryInterface {
    fn load_byte(&mut self, addr: u16) -> u8 {
        match addr {
            RAM_START ... RAM_END => self.ram.load_byte(addr),
            PPU_REG_START ... PPU_REG_END => self.ppu.load_byte(addr),
            APU_REG_START ... APU_REG_END => self.apu.load_byte(addr),
            IO_REG => panic!("IO not implemented"),
            APU_IO_SHARED_REG => self.apu.load_byte(addr), // TODO: also map to io
            CART_MAPPER_START ... CART_MAPPER_END => self.mapper.load_byte_prg(addr),
            _ => panic!("Address out of range: {:X}", addr)
        }
    }
    
    fn store_byte(&mut self, addr: u16, val: u8) {
        match addr {
            RAM_START ... RAM_END => self.ram.store_byte(addr, val),
            PPU_REG_START ... PPU_REG_END => self.ppu.store_byte(addr, val),
            APU_REG_START ... APU_REG_END => self.apu.store_byte(addr, val),
            IO_REG => panic!("IO not implemented"),
            APU_IO_SHARED_REG => self.apu.store_byte(addr, val), // TODO: also map to io
            CART_MAPPER_START ... CART_MAPPER_END => self.mapper.store_byte_prg(addr, val),
            _ => panic!("Address out of range: {:X}", addr)
        }
    }
}

pub struct Ram {
    pub ram: [u8; RAM_SIZE as usize]
}

impl Ram {
    fn new() -> Ram {
        Ram {
            ram: [0; RAM_SIZE as usize]
        }
    }
}

impl Memory for Ram {
    fn load_byte(&mut self, addr: u16) -> u8 {
        let mut idx = addr;
        if idx >= RAM_SIZE {
            idx = idx % RAM_SIZE;
        }
        self.ram[idx as usize]
    }
    
    fn store_byte(&mut self, addr: u16, val: u8) {
        let mut idx = addr;
        if idx >= RAM_SIZE {
            idx = idx % RAM_SIZE;
        }        
        self.ram[idx as usize] = val;
    }
}