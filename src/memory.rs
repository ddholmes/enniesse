use rom::Rom;
use mapper;
use mapper::Mapper;
use apu::Apu;
use ppu::Ppu;
use input::Input;

use std::rc::Rc;
use std::cell::RefCell;

// the 2k is mirrored between 4 2k blocks
const RAM_SIZE: u16 = 2048;

const RAM_START: u16             = 0x0000;
const RAM_END: u16               = 0x1fff;
const PPU_REG_START: u16         = 0x2000;
pub const PPU_OAM_DATA: u16      = 0x2004;
const PPU_REG_END: u16           = 0x3fff;
const APU_REG_START: u16         = 0x4000;
const APU_REG_END: u16           = 0x4013;
pub const PPU_OAM_DMA: u16       = 0x4014;
const APU_STATUS_REG: u16        = 0x4015;
pub const IO_REG: u16            = 0x4016;
pub const APU_IO_SHARED_REG: u16 = 0x4017;
const CART_MAPPER_START: u16     = 0x4020;
const CART_MAPPER_END: u16       = 0xffff;

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
    pub mapper: Rc<RefCell<Box<Mapper>>>,
    pub apu: Apu,
    pub ppu: Ppu,
    pub input: Input
}

impl MemoryInterface {
    pub fn new(rom: Box<Rom>) -> MemoryInterface {
        let mapper = mapper::load_mapper(rom);
        // Rc allows sharing the pointer, RefCell allows mutability
        let shared_mapper = Rc::new(RefCell::new(mapper));
        let ppu = Ppu::new(shared_mapper.clone());
        
        MemoryInterface {
            ram: Ram::new(),
            mapper: shared_mapper,
            apu: Apu::new(),
            ppu: ppu,
            input: Input::new()
        }
    }
}

impl Memory for MemoryInterface {
    fn load_byte(&mut self, addr: u16) -> u8 {
        match addr {
            RAM_START ... RAM_END => self.ram.load_byte(addr),
            PPU_REG_START ... PPU_REG_END => self.ppu.load_byte(addr),
            APU_REG_START ... APU_REG_END => self.apu.load_byte(addr),
            APU_STATUS_REG => self.apu.load_byte(addr),
            IO_REG => self.input.load_byte(addr),
            APU_IO_SHARED_REG => self.apu.load_byte(addr) | self.input.load_byte(addr),
            CART_MAPPER_START ... CART_MAPPER_END => self.mapper.borrow_mut().load_byte_prg(addr),
            _ => panic!("Read address out of range: {:X}", addr)
        }
    }
    
    fn store_byte(&mut self, addr: u16, val: u8) {
        match addr {
            RAM_START ... RAM_END => self.ram.store_byte(addr, val),
            PPU_REG_START ... PPU_REG_END => self.ppu.store_byte(addr, val),
            APU_REG_START ... APU_REG_END => self.apu.store_byte(addr, val),
            APU_STATUS_REG => self.apu.store_byte(addr, val),
            IO_REG => self.input.store_byte(addr, val),
            APU_IO_SHARED_REG => {
                self.apu.store_byte(addr, val);
                self.input.store_byte(addr, val);
            },
            CART_MAPPER_START ... CART_MAPPER_END => self.mapper.borrow_mut().store_byte_prg(addr, val),
            _ => panic!("Write address out of range: {:X}", addr)
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