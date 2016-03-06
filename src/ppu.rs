use std::ops::Deref;
use memory::Memory;
use mapper::Mapper;

use std::rc::Rc;
use std::cell::RefCell;

// the PPU register addresses repeat every 8 bits starting at 2000, so mask them to 0-7
const PPU_CTRL: u16   = 0x2000 & 0x07;
const PPU_MASK: u16   = 0x2001 & 0x07;
const PPU_STATUS: u16 = 0x2002 & 0x07;
const OAM_ADDR: u16   = 0x2003 & 0x07;
const OAM_DATA: u16   = 0x2004 & 0x07;
const PPU_SCROLL: u16 = 0x2005 & 0x07;
const PPU_ADDR: u16   = 0x2006 & 0x07;
const PPU_DATA: u16   = 0x2007 & 0x07;

// memory map
const MAPPER_START: u16    = 0x0000;
const MAPPER_END: u16      = 0x1fff;
const NAMETABLE_START: u16 = 0x2000;
const NAMETABLE_END: u16   = 0x2fff;
const PALETTE_START: u16   = 0x3f00;
const PALETTE_END: u16     = 0x3f1f;

const PPU_RAM_SIZE: usize = 0x800;

pub struct Ppu {
    reg_ctrl: CtrlRegister,
    reg_mask: MaskRegister,
    reg_status: StatusRegister,
    reg_oam_addr: u8,
    reg_scroll: ScrollRegister,
    reg_addr: AddressRegister,
    
    vram: Vram,
    oam: Oam
}

impl Ppu {
    pub fn new(mapper: Rc<RefCell<Box<Mapper>>>) -> Ppu {
        Ppu {
            reg_ctrl: CtrlRegister(0),
            reg_mask: MaskRegister(0),
            reg_status: StatusRegister(0),
            reg_oam_addr: 0,            
            reg_scroll: ScrollRegister { x: 0, y: 0, write_next: ScrollDirection::X },
            reg_addr: AddressRegister { address: 0, write_next: AddressByte::Upper },
            
            vram: Vram::new(mapper),
            oam: Oam::new()
        }
    }
    
    pub fn run(&mut self) {
        
    }
    
    fn read_status(&mut self) -> u8 {
        *self.reg_status
    }
    
    fn read_oam_data(&mut self) -> u8 {
        let addr = self.reg_oam_addr;
        self.oam.load_byte(addr as u16)
    }
    
    fn read_data(&mut self) -> u8 {
        let addr = self.reg_addr.address;
        self.vram.load_byte(addr)
    }
    
    fn write_ctrl(&mut self, val: u8) {
        self.reg_ctrl = CtrlRegister(val);
    }
    
    fn write_mask(&mut self, val: u8) {
        self.reg_mask = MaskRegister(val);
    }
    
    fn write_oam_addr(&mut self, val: u8) {
        self.reg_oam_addr = val;
    }
    
    fn write_oam_data(&mut self, val: u8) {
        let addr = self.reg_oam_addr;
        self.oam.store_byte(addr as u16, val);
        self.reg_oam_addr += 1;
    }
    
    fn write_scroll(&mut self, val: u8) {
        match self.reg_scroll.write_next {
            ScrollDirection::X => {
                self.reg_scroll.x = val;
                self.reg_scroll.write_next = ScrollDirection::Y;
            },
            ScrollDirection::Y => {
                self.reg_scroll.y = val;
                self.reg_scroll.write_next = ScrollDirection::X;
            }
        }
    }
    
    fn write_addr(&mut self, val: u8) {
        match self.reg_addr.write_next {
            AddressByte::Upper => {
                self.reg_addr.address = (self.reg_addr.address & 0x00ff) | (val as u16) << 8;
                self.reg_addr.write_next = AddressByte::Lower;
            },
            AddressByte::Lower => {
                self.reg_addr.address = (self.reg_addr.address & 0xff00) | val as u16;
                self.reg_addr.write_next = AddressByte::Upper;
            }
        }
    }
    
    fn write_data(&mut self, val: u8) {
        let addr = self.reg_addr.address;
        self.vram.store_byte(addr, val);
    }
}

// cpu memory interface to ppu registers
impl Memory for Ppu {
    fn load_byte(&mut self, addr: u16) -> u8 {
        // repeats every 8 bytes
        match addr & 0x07 {
            PPU_CTRL => 0, // write only
            PPU_MASK => 0, // write only
            PPU_STATUS => self.read_status(),
            OAM_ADDR => 0, // write only
            OAM_DATA => self.read_oam_data(),
            PPU_SCROLL => 0, // write only
            PPU_ADDR => 0, // write only
            PPU_DATA => self.read_data(),
            _ => panic!("Unknown PPU register {:04X}", addr)
        }
    }
    fn store_byte(&mut self, addr: u16, val: u8) {
        // repeats every 8 bytes
        match addr & 0x07 {
            PPU_CTRL => self.write_ctrl(val), 
            PPU_MASK => self.write_mask(val),
            PPU_STATUS => {}, // read only
            OAM_ADDR => self.write_oam_addr(val),
            OAM_DATA => self.write_oam_data(val),
            PPU_SCROLL => self.write_scroll(val),
            PPU_ADDR => self.write_addr(val),
            PPU_DATA => self.write_data(val),
            _ => panic!("Unknown PPU register {:04X}", addr)
        }
    }
}

struct Vram {
    mapper: Rc<RefCell<Box<Mapper>>>,
    nametable: [u8; PPU_RAM_SIZE], // 2kb ram
    palette_index: [u8; 0x20]
}

impl Vram {
    fn new(mapper: Rc<RefCell<Box<Mapper>>>) -> Vram {
        Vram {
            mapper: mapper,
            nametable: [0; PPU_RAM_SIZE],
            palette_index: [0; 0x20]
        }
    }
}

impl Memory for Vram {
    fn load_byte(&mut self, addr: u16) -> u8 {
        match addr {
            MAPPER_START ... MAPPER_END => self.mapper.borrow_mut().load_byte_chr(addr),
            NAMETABLE_START ... NAMETABLE_END => self.nametable[addr as usize & (PPU_RAM_SIZE - 1)],
            PALETTE_START ... PALETTE_END => self.palette_index[addr as usize & 0x1f],
            _ => panic!("Unknown PPU address {:04X}", addr)
        }
    }
    fn store_byte(&mut self, addr: u16, val: u8) {
        match addr {
            MAPPER_START ... MAPPER_END => self.mapper.borrow_mut().store_byte_chr(addr, val),
            NAMETABLE_START ... NAMETABLE_END => self.nametable[addr as usize & (PPU_RAM_SIZE - 1)] = val,
            PALETTE_START ... PALETTE_END => self.palette_index[addr as usize & 0x1f] = val,
            _ => panic!("Unknown PPU address {:04X}", addr)
        }
    }
}

struct Oam {
    oam : [u8; 256]
}

impl Oam {
    fn new() -> Oam {
        Oam {
            oam: [0; 256]
        }
    }
}

impl Memory for Oam {
    fn load_byte(&mut self, addr: u16) -> u8 {
        self.oam[addr as usize & 0x00ff]
    }
    fn store_byte(&mut self, addr: u16, val: u8) {
        self.oam[addr as usize & 0x00ff] = val;
    }
}

struct StatusRegister(u8);

impl StatusRegister {
    fn get_vblank(&self) -> bool {
        self.0 & 0b1000_0000 != 0
    }
    
    fn get_sprite_0_hit(&self) -> bool {
        self.0 & 0b0100_0000 != 0
    }
    
    fn get_sprite_overflow(&self) -> bool {
        self.0 & 0b0010_0000 != 0
    }
    
    fn set_vblank(&mut self, val: bool) {
        if val {
            self.0 |= 0b1000_0000;
        } else {
            self.0 &= 0b0111_1111;
        }
    }
    
    fn set_sprite_0_hit(&mut self, val: bool) {
        if val {
            self.0 |= 0b0100_0000;
        } else {
            self.0 &= 0b1011_1111;
        }
    }
    
    fn set_sprite_overflow(&mut self, val: bool) {
        if val {
            self.0 |= 0b0010_0000;
        } else {
            self.0 &= 0b1101_1111;
        }
    }
}

impl Deref for StatusRegister {
    type Target = u8;
    
    fn deref(&self) -> &u8 {
        &self.0
    }
}

struct CtrlRegister(u8);

impl CtrlRegister {
    fn get_base_nametable_address(&self) -> u16 {
        match self.0 & 0b0000_0011 {
            0 => 0x2000,
            1 => 0x2400,
            2 => 0x2800,
            3 => 0x2c00,
            _ => unreachable!()
        }
    }
    
    fn get_x_scroll_offset(&self) -> u8 {
        if self.0 & 0b0000_0001 != 0 { 256 } else { 0 }
    }
    
    fn get_y_scroll_offset(&self) -> u8 {
        if self.0 & 0b0000_0010 != 0 { 240 } else { 0 }
    }
    
    fn get_vram_address_increment(&self) -> u8 {
        if self.0 & 0b0000_0100 != 0 { 32 } else { 1 }
    }
    
    fn get_sprite_pattern_table_address(&self) -> u16 {
        if self.0 & 0b0000_1000 != 0 { 0x1000 } else { 0x0000 }
    }
    
    fn get_background_pattern_table_address(&self) -> u16 {
        if self.0 & 0b0001_0000 != 0 { 0x1000 } else { 0x0000 }
    }
    
    fn get_sprite_size(&self) -> SpriteSize {
        if self.0 & 0b0010_0000 != 0 { SpriteSize::Size8x16 } else { SpriteSize::Size8x8 }
    }
    
    fn get_master_slave_select(&self) -> bool {
        self.0 & 0b0100_0000 != 0
    }
    
    fn get_generate_nmi(&self) -> bool {
        self.0 & 0b1000_0000 != 0
    }
}

impl Deref for CtrlRegister {
    type Target = u8;
    
    fn deref(&self) -> &u8 {
        &self.0
    }
}

enum SpriteSize {
    Size8x8,
    Size8x16
}

struct MaskRegister(u8);

impl MaskRegister {
    fn get_greyscale(&self) -> bool {
        self.0 & 0b0000_0001 != 0
    }
    
    fn get_show_background_left(&self) -> bool {
        self.0 & 0b0000_0010 != 0
    }
    
    fn get_show_sprites_left(&self) -> bool {
        self.0 & 0b0000_0100 != 0
    }
    
    fn get_show_background(&self) -> bool {
        self.0 & 0b0000_1000 != 0
    }
    
    fn get_show_sprites(&self) -> bool {
        self.0 & 0b0001_0000 != 0
    }
    
    fn get_emphasize_red(&self) -> bool {
        self.0 & 0b0010_0000 != 0
    }
    
    fn get_emphasize_green(&self) -> bool {
        self.0 & 0b0100_0000 != 0
    }
    
    fn get_emphasize_blue(&self) -> bool {
        self.0 & 0b1000_0000 != 0
    }
}

impl Deref for MaskRegister {
    type Target = u8;
    
    fn deref(&self) -> &u8 {
        &self.0
    }
}

struct ScrollRegister {
    x: u8,
    y: u8,
    write_next: ScrollDirection
}

enum ScrollDirection {
    X,
    Y
}

struct AddressRegister {
    address: u16,
    write_next: AddressByte
}

enum AddressByte {
    Upper,
    Lower
}