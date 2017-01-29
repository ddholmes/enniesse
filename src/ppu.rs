use std::ops::Deref;
use memory::Memory;
use mapper::{Mapper, Mirroring};

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
const NAMETABLE_END: u16   = 0x3eff;
const PALETTE_START: u16   = 0x3f00;
const PALETTE_END: u16     = 0x3fff;

const PPU_RAM_SIZE: usize = 0x800;

const CPU_CYCLES_PER_FRAME: u16 = 29781;
const SCANLINES_PER_FRAME: u16 = 262;
pub const CPU_CYCLES_PER_SCANLINE: u16 = CPU_CYCLES_PER_FRAME / SCANLINES_PER_FRAME;
const PPU_CYCLES_PER_SCANLINE: u16 = 341;

const VBLANK_SCANLINE_START: i16 = 241;
const VBLANK_SCANLINE_END: i16 = 260;

pub const SCREEN_WIDTH: usize = 256;
pub const SCREEN_HEIGHT: usize = 240;

static RGB_PALETTE: [u8; 192] = [
     84,  84,  84,    0,  30, 116,    8,  16, 144,   48,   0, 136,
     68,   0, 100,   92,   0,  48,   84,   4,   0,   60,  24,   0,
     32,  42,   0,    8,  58,   0,    0,  64,   0,    0,  60,   0,
      0,  50,  60,    0,   0,   0,    0,   0,   0,    0,   0,   0,
      
    152, 150, 152,    8,  76, 196,   48,  50, 236,   92,  30, 228,
    136,  20, 176,  160,  20, 100,  152,  34,  32,  120,  60,   0,
     84,  90,   0,   40, 114,   0,    8, 124,   0,    0, 118,  40,
      0, 102, 120,    0,   0,   0,    0,   0,   0,    0,   0,   0,
      
    236, 238, 236,   76, 154, 236,  120, 124, 236,  176,  98, 236,
    228,  84, 236,  236,  88, 180,  236, 106, 100,  212, 136,  32,
    160, 170,   0,  116, 196,   0,   76, 208,  32,   56, 204, 108,
     56, 180, 204,   60,  60,  60,    0,   0,   0,    0,   0,   0,
     
    236, 238, 236,  168, 204, 236,  188, 188, 236,  212, 178, 236,
    236, 174, 236,  236, 174, 212,  236, 180, 176,  228, 196, 144,
    204, 210, 120,  180, 222, 120,  168, 226, 144,  152, 226, 180,
    160, 214, 228,  160, 162, 160,    0,   0,   0,    0,   0,   0,
];

pub struct Ppu {
    reg_ctrl: CtrlRegister,
    reg_mask: MaskRegister,
    reg_status: StatusRegister,
    reg_oam_addr: u8,
    
    data_read_buffer: u8,
    
    current_vram_address: u16, // v
    temporary_vram_address: u16, // t
    fine_x: u8, // x
    write_toggle: AddressByte, // w
    
    shifter_low: u8,
    shifter_high: u8,
    
    scanline: i16,
    
    vram: Vram,
    oam: Oam,
    
    pub display_buffer: Box<[u8; SCREEN_WIDTH * SCREEN_HEIGHT * 3]>
}

impl Ppu {
    pub fn new(mapper: Rc<RefCell<Box<Mapper>>>) -> Ppu {
        Ppu {
            reg_ctrl: CtrlRegister(0),
            reg_mask: MaskRegister(0),
            reg_status: StatusRegister(0),
            reg_oam_addr: 0,
            
            data_read_buffer: 0,
            
            current_vram_address: 0,
            temporary_vram_address: 0,
            fine_x: 0,
            write_toggle: AddressByte::Upper,
            
            shifter_low: 0,
            shifter_high: 0,
            
            scanline: -1,
            
            vram: Vram::new(mapper),
            oam: Oam::new(),
            
            display_buffer: Box::new([0; SCREEN_WIDTH * SCREEN_HEIGHT * 3])
        }
    }
    
    // run PPU for one scanline
    pub fn run(&mut self) -> PpuRunResult {
        let mut result = PpuRunResult::default();
            
        // copy t to v at scanline 0 if rendering is enabled
        if self.scanline == 0 && (self.reg_mask.get_show_background() || self.reg_mask.get_show_sprites()) {
            self.current_vram_address = self.temporary_vram_address;
        }
        
        if self.scanline >= 0 && self.scanline < SCREEN_HEIGHT as i16 {
            println!("RENDER SCANLINE {} v:{:04X} t:{:04X}", self.scanline, self.current_vram_address, self.temporary_vram_address);
            self.render_scanline();
        }
        
        if self.scanline == VBLANK_SCANLINE_START {
            self.reg_status.set_vblank(true);
            if self.reg_ctrl.get_generate_nmi() {
                result.vblank = true;
            }
            result.render_frame = true;
            println!("[frame]");
        } else if self.scanline == VBLANK_SCANLINE_END {
            self.scanline = -1;
            self.reg_status.set_vblank(false);
        }
        
        self.scanline += 1;
       
        result
    }
    
    // rendering
    
    fn render_scanline(&mut self) {
        let backdrop_index = self.vram.load_byte(PALETTE_START);
        let backdrop_color = self.get_color_from_palette(backdrop_index as usize);
        
        let show_background_left = self.reg_mask.get_show_background_left();
        let show_background = self.reg_mask.get_show_background();
        let show_sprites_left = self.reg_mask.get_show_sprites_left();
        let show_sprites = self.reg_mask.get_show_sprites();

        // at the start of each scanline copy horizontal (x) bits of t to v if rendering is enabled
        if show_background || show_sprites {
            self.current_vram_address = (self.current_vram_address & !0x41f) | (self.temporary_vram_address & 0x41f);
        }
        
        let y = self.scanline;
        
        for x in 0 .. SCREEN_WIDTH {
            // get the background color
            let mut background_color = None;
            if x < 8 && show_background_left || x >= 8 && show_background {
                background_color = self.get_background_pixel((x % 8) as u8);
            }
            
            // get sprite color
            if x < 8 && show_sprites_left || x >= 8 && show_sprites {
                
            }
            
            // determine what color to use based on priority
            // TODO
            let color = if let Some(color) = background_color { color } else { backdrop_color };
            
            // write the pixel to the display buffer
            self.display_buffer[(y as usize * SCREEN_WIDTH + x) * 3 + 0] = color.r;
            self.display_buffer[(y as usize * SCREEN_WIDTH + x) * 3 + 1] = color.g;
            self.display_buffer[(y as usize * SCREEN_WIDTH + x) * 3 + 2] = color.b;
            
            // increment coarse X and wrap if needed
            if (x + 1) % 8 == 0 {
                if (self.current_vram_address & 0x001f) == 31 {
                    self.current_vram_address &= !(0x001f);
                    if self.vram.mirroring == Mirroring::Horizontal {
                        self.current_vram_address ^= 0x0400;
                    }
                } else {
                    self.current_vram_address += 1;
                }
            }
        }
        
        if show_background || show_sprites {
            // increment y in the current vram address and wrap if needed
            if (self.current_vram_address & 0x7000) != 0x7000 {
                self.current_vram_address += 0x1000;
            } else {
                self.current_vram_address &= !0x7000;
                let mut y = (self.current_vram_address & 0x03e0) >> 5;
                
                if y == 29 {
                    y = 0;
                    if self.vram.mirroring == Mirroring::Vertical {
                        self.current_vram_address ^= 0x0800;
                    }
                } else if y == 31 {
                    y = 0;
                } else {
                    y += 1;
                }
                
                self.current_vram_address = (self.current_vram_address & !0x03e0) | (y << 5);
            }
        }
    }
    
    // get the color of a pixel (0-7) within the tile referenced by the current vram address (v)
    fn get_background_pixel(&mut self, x: u8) -> Option<RgbColor> {
        let v = self.current_vram_address;
        
        let nametable_base = self.reg_ctrl.get_base_nametable_address();
        let tile_index = self.vram.load_byte(nametable_base | (v & 0x0FFF)) as u16;
        
        let attribute_addr = 0x23c0 | (v & 0x0c00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07);
        let attribute_byte = self.vram.load_byte(attribute_addr);
        
        // do something with attribute_byte
        let coarse_x = v & 0x1f;
        let coarse_y = (v >> 5) & 0x1f;
        let attribute_color = match (coarse_x % 4, coarse_y % 4) {
            (0 ... 1, 0 ... 1) => attribute_byte >> 0 & 0b0011, // top left
            (2 ... 3, 0 ... 1) => attribute_byte >> 2 & 0b0011, // top right
            (0 ... 1, 2 ... 3) => attribute_byte >> 4 & 0b0011, // bottom left
            (2 ... 3, 2 ... 3) => attribute_byte >> 6 & 0b0011, // bottom right
            (_, _) => unreachable!()
        };
        
        let pattern_address = self.reg_ctrl.get_background_pattern_table_address() as u16;
        let fine_y = (self.current_vram_address >> 12) & 7;
        
        let plane0 = self.vram.load_byte(pattern_address | (tile_index << 4) | fine_y);
        let plane1 = self.vram.load_byte(pattern_address | (tile_index << 4) | fine_y | 8);
        
        // let bit0 = (plane0 >> (7 - x - self.fine_x)) & 1;
        // let bit1 = (plane1 >> (7 - x - self.fine_x)) & 1; 
        let bit0 = (plane0 >> (7 - x)) & 1;
        let bit1 = (plane1 >> (7 - x)) & 1;
        
        let pattern_color = (bit1 << 1) | bit0;
        
        let palette = (attribute_color << 2) | pattern_color;
        let color_index = self.vram.load_byte(PALETTE_START + palette as u16);
        
        if color_index == 0 {
            return None;
        }
        
        Some(self.get_color_from_palette(color_index as usize))
    }
    
    fn get_color_from_palette(&self, index: usize) -> RgbColor {
        RgbColor {
            r: RGB_PALETTE[index * 3],
            g: RGB_PALETTE[index * 3 + 1],
            b: RGB_PALETTE[index * 3 + 2]
        }
    }
    
    
    // register read/writes
    
    fn read_status(&mut self) -> u8 {
        // reading status resets the write toggle
        self.write_toggle = AddressByte::Upper;
        
        let status = self.reg_status;
        // vblank is cleared after reading status
        self.reg_status.set_vblank(false);
        
        *status
    }
    
    fn read_oam_data(&mut self) -> u8 {
        let addr = self.reg_oam_addr;
        self.oam.load_byte(addr as u16)
    }
    
    fn read_data(&mut self) -> u8 {
        let addr = self.current_vram_address;
        self.current_vram_address += self.reg_ctrl.get_vram_address_increment();
        let data = self.vram.load_byte(addr);
        
        // reads before the palette are buffered
        if addr < PALETTE_START {
            let buffer = self.data_read_buffer;
            self.data_read_buffer = data;
            return buffer;
        } else if addr >= PALETTE_START && addr <= PALETTE_END {
            // data is still buffered on palette reads from the corresponding nametable bytes
            self.data_read_buffer = self.vram.load_byte(NAMETABLE_START | (addr & PPU_RAM_SIZE as u16 - 1));
        }
        
        data
    }
    
    fn write_ctrl(&mut self, val: u8) {
        self.reg_ctrl = CtrlRegister(val);
        
        // the lower 2 bits of the ctrl value are put into bits 10 and 11 of t
        self.temporary_vram_address &= ((val as u16 & 3) << 10) | 0x73ff;
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
        self.reg_oam_addr = self.reg_oam_addr.wrapping_add(1);
    }
    
    fn write_scroll(&mut self, val: u8) {
        match self.write_toggle {
            AddressByte::Upper => {
                self.write_toggle = AddressByte::Lower;
                
                self.fine_x = val & 0b111;
                self.temporary_vram_address &= (val as u16 >> 3) | 0xffe0;
            },
            AddressByte::Lower => {
                self.write_toggle = AddressByte::Upper;
                
                let val = val as u16;
                self.temporary_vram_address &= ((val & 0b1100_0000) << 2) | 0xfcff;
                self.temporary_vram_address &= ((val & 0b0011_1000) << 2) | 0xff1f;
                self.temporary_vram_address &= ((val & 0b0000_0111) << 12) | 0x0fff;
            }
        }
    }
    
    fn write_addr(&mut self, val: u8) {
        match self.write_toggle {
            AddressByte::Upper => {
                self.write_toggle = AddressByte::Lower;
                
                self.temporary_vram_address = self.temporary_vram_address & 0xff | ((val as u16) << 8);
                
                // bit 14 is cleared
                if self.scanline < VBLANK_SCANLINE_START && self.scanline >= 0 
                    && (self.reg_mask.get_show_background() || self.reg_mask.get_show_sprites()) {
                    self.temporary_vram_address &= !(1 << 13);
                }
            },
            AddressByte::Lower => {
                self.write_toggle = AddressByte::Upper;
                
                self.temporary_vram_address = self.temporary_vram_address & 0xff00 | val as u16;
                self.current_vram_address = self.temporary_vram_address;
            }
        }
    }
    
    fn write_data(&mut self, val: u8) {
        let addr = self.current_vram_address;
        self.vram.store_byte(addr, val);
        self.current_vram_address += self.reg_ctrl.get_vram_address_increment();
    }
    
    fn trace_read(addr: u16) {
        //println!("R: {:04X}", addr);
    }
    
    fn trace_write(addr: u16, val: u8) {
        //println!("W: {:04X} -> {:02X}", addr, val);
    }
}

// cpu memory interface to ppu registers
impl Memory for Ppu {
    fn load_byte(&mut self, addr: u16) -> u8 {
        Ppu::trace_read(addr);
        
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
        Ppu::trace_write(addr, val);
        
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

#[derive(Copy, Clone, Debug)]
struct RgbColor {
    r: u8,
    g: u8,
    b: u8
}

#[derive(Default)]
pub struct PpuRunResult {
    pub vblank: bool,
    pub mapper_irq: bool,
    pub render_frame: bool
}

struct Vram {
    mapper: Rc<RefCell<Box<Mapper>>>,
    nametable: [u8; PPU_RAM_SIZE], // 2kb ram
    palette: [u8; 0x20],
    mirroring: Mirroring
}

impl Vram {
    fn new(mapper: Rc<RefCell<Box<Mapper>>>) -> Vram {
        let mirroring = mapper.borrow().get_mirroring();
        Vram {
            mapper: mapper,
            nametable: [0; PPU_RAM_SIZE],
            palette: [0; 0x20],
            mirroring: mirroring
        }
    }
    
    fn get_nametable_addr(&self, addr: u16) -> usize {
        let mut nametable_addr = addr as usize & 0xfff;
        
        nametable_addr = match (self.mirroring, nametable_addr) {
            (Mirroring::Horizontal, 0x0000 ... 0x07ff) => nametable_addr & !(1 << 10),
            (Mirroring::Horizontal, 0x0800 ... 0x0fff) => nametable_addr - 0x400,
            (Mirroring::Vertical, 0x0000 ... 0x07ff) => nametable_addr,
            (Mirroring::Vertical, 0x0800 ... 0x0fff) => nametable_addr & !(1 << 11),
            (_, _) => nametable_addr
        };
        
        nametable_addr & (PPU_RAM_SIZE - 1)
    }
}

impl Memory for Vram {
    fn load_byte(&mut self, addr: u16) -> u8 {
        Ppu::trace_read(addr);
        
        match addr {
            MAPPER_START ... MAPPER_END => self.mapper.borrow_mut().load_byte_chr(addr),
            NAMETABLE_START ... NAMETABLE_END => {
                let nametable_addr = self.get_nametable_addr(addr);
                self.nametable[nametable_addr]
            },
            PALETTE_START ... PALETTE_END => {
                // handle mirrored addresses
                let mut addr = addr as usize & 0x1f;
                if addr >= 0x10 && addr % 4 == 0 {
                    addr -= 0x10;
                }
                
                self.palette[addr as usize & 0x1f]
            },
            0x4000 ... 0x7fff => self.nametable[addr as usize & (PPU_RAM_SIZE - 1)],
            _ => panic!("Unknown PPU address {:04X}", addr)
        }
    }
    fn store_byte(&mut self, addr: u16, val: u8) {
        Ppu::trace_write(addr, val);
        
        match addr {
            MAPPER_START ... MAPPER_END => {
                self.mapper.borrow_mut().store_byte_chr(addr, val);
            },
            NAMETABLE_START ... NAMETABLE_END => {
                let nametable_addr = self.get_nametable_addr(addr);
                
                //println!("nametable write {:04X} {:04X} {:02X}", addr, nametable_addr, val);
                
                self.nametable[nametable_addr] = val;
            },
            PALETTE_START ... PALETTE_END => {
                //println!("palette write {:04X} {:02X}", addr, val);
                // handle mirrored addresses
                let mut addr = addr as usize & 0x1f;
                if addr >= 0x10 && addr % 4 == 0 {
                    addr -= 0x10;
                }
                
                self.palette[addr as usize & 0x1f] = val
            },
            0x4000 ... 0x7fff => {
                //println!("nametable write {:04X} {:02X}", addr, val);
                self.nametable[addr as usize & (PPU_RAM_SIZE - 1)] = val;
            }
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

#[derive(Copy, Clone)]
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

#[derive(Copy, Clone)]
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
    
    fn get_vram_address_increment(&self) -> u16 {
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
    
    fn set_generate_nmi(&mut self, val: bool) {
        if val {
            self.0 |= 0b1000_0000;
        } else {
            self.0 &= 0b0111_1111;
        }
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

#[derive(Copy, Clone)]
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

enum AddressByte {
    Upper,
    Lower
}