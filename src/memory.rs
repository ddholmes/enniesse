use super::rom::Rom;
use super::mapper;

use std::fmt;

const RAM_START: u16 = 0x0000;
const RAM_END: u16 = 0x1fff;
// the 2k is mirrored between 4 2k blocks
const RAM_SIZE: u16 = 2048;

const PPU_REG_START: u16 = 0x2000;
const PPU_REG_END: u16 = 0x3fff;

const APU_IO_REG_START: u16 = 0x4000;
const APU_IO_REG_END: u16 = 0x401f;

const CART_MAPPER_START: u16 = 0x4020;
const CART_MAPPER_END: u16 = 0xffff;

pub struct MemoryMap {
    pub ram: [u8; RAM_SIZE as usize],
    pub mapper: Box<MemoryRegion>
}

impl MemoryMap {
    pub fn new(rom: Box<Rom>) -> MemoryMap {
        MemoryMap {
            ram: [0; RAM_SIZE as usize],
            mapper: mapper::load_mapper(rom)
        }
    }
    
    pub fn load_word(&mut self, addr: u16) -> u16 {
        let mut region = self.get_region(addr);
        region.load_word(addr)
    }
    
    pub fn load_byte(&mut self, addr: u16) -> u8 {
        let mut region = self.get_region(addr);
        region.load_byte(addr)
    }
    
    fn get_region(&mut self, addr: u16) -> &mut Box<MemoryRegion> {
        match addr {
            RAM_START ... RAM_END => panic!("Ram not implemented"),
            PPU_REG_START ... PPU_REG_END => panic!("PPU not implemented"),
            APU_IO_REG_START ... APU_IO_REG_END => panic!("APU and IO not implemented"),
            CART_MAPPER_START ... CART_MAPPER_END => &mut self.mapper,
            _ => panic!("Address out of range: {:X}", addr)
        }
    }
}

impl fmt::Debug for MemoryMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Debug not implemented for MemoryMap")
    }
}

pub trait MemoryRegion { 
    fn load_byte(&mut self, addr: u16) -> u8;
    fn load_word(&mut self, addr: u16) -> u16 {
        self.load_byte(addr) as u16 | (self.load_byte(addr + 1) as u16) << 8
    }
    
    fn store_byte(&mut self, addr: u16, val: u8);
    fn store_word(&mut self, addr: u16, val: u16) {
        self.store_byte(addr, (val & 0xff) as u8);
        self.store_byte(addr + 1, ((val & 0xff) >> 8) as u8);
    }
}