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

const CPU_CYCLES_PER_FRAME: u16 = 29781;
const SCANLINES_PER_FRAME: u16 = 262;
pub const CPU_CYCLES_PER_SCANLINE: u16 = CPU_CYCLES_PER_FRAME / SCANLINES_PER_FRAME;
const PPU_CYCLES_PER_SCANLINE: u16 = 341;

const VBLANK_SCANLINE_START: u16 = 241;
const VBLANK_SCANLINE_END: u16 = 261;

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
    reg_scroll: ScrollRegister,
    reg_addr: AddressRegister,
    
    scanline: u16,
    
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
            reg_scroll: ScrollRegister { x: 0, y: 0, write_next: ScrollDirection::X },
            reg_addr: AddressRegister { address: 0, write_next: AddressByte::Upper },
            
            scanline: 0,
            
            vram: Vram::new(mapper),
            oam: Oam::new(),
            
            display_buffer: Box::new([0; SCREEN_WIDTH * SCREEN_HEIGHT * 3])
        }
    }
    
    pub fn run(&mut self) -> PpuRunResult {
        let mut result = PpuRunResult::default();
            
        // TODO: render stuff
        if self.scanline < SCREEN_HEIGHT as u16 {
            self.render_scanline();
        }
        
        self.scanline += 1;
        
        if self.scanline == VBLANK_SCANLINE_START {
            self.reg_status.set_vblank(true);
            if self.reg_ctrl.get_generate_nmi() {
                result.vblank = true;
            }
        } else if self.scanline == VBLANK_SCANLINE_END {
            self.scanline = 0;
            self.reg_status.set_vblank(false);
            result.render_frame = true;
        }
       
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
        
        let y = self.scanline;
        
        for x in 0 .. SCREEN_WIDTH {
            // get the background color
            let mut background_color = None;
            if x < 8 && show_background_left || show_background {
                background_color = self.get_background_color(x as u16, y);
            }
            
            // get sprite color
            if x < 8 && show_sprites_left || show_sprites {
                
            }
            
            // determine what color to use based on priority
            // TODO
            let color = if let Some(color) = background_color { color } else { backdrop_color };
            
            // write the pixel to the display buffer
            self.display_buffer[(y as usize * SCREEN_WIDTH + x) * 3 + 0] = color.r;
            self.display_buffer[(y as usize * SCREEN_WIDTH + x) * 3 + 1] = color.g;
            self.display_buffer[(y as usize * SCREEN_WIDTH + x) * 3 + 2] = color.b;
        }
    }
    
    fn get_background_color(&mut self, x: u16, y: u16) -> Option<RgbColor> {
        let base = self.reg_ctrl.get_base_nametable_address();
        let x_index = (x / 8) % 32;
        let y_index = (y / 8) % 30;
        
        // 32 8x8 sections per row
        let tile = self.vram.load_byte(base + x_index + 32 * y_index);
        
        // 8 32x32 sections per row (so 4 8pixel squares per section)
        let attribute_addr = base + 0x3c0 + (y_index / 4 * 8) + x_index / 4;
        let attribute_byte = self.vram.load_byte(attribute_addr);
        
        // byte is divided into 4 sections, each 2 bits
        let attribute_color = match (x_index % 4, y_index % 4) {
            (0 ... 1, 0 ... 1) => attribute_byte >> 0 & 0b0011, // top left
            (2 ... 3, 0 ... 1) => attribute_byte >> 2 & 0b0011, // top right
            (0 ... 1, 2 ... 3) => attribute_byte >> 4 & 0b0011, // bottom left
            (2 ... 3, 2 ... 3) => attribute_byte >> 6 & 0b0011, // bottom right
            (_, _) => unreachable!()
        };
        
        // fetch from pattern table
        let tile_x = (x % 8) as u8;
        let tile_y = (y % 8) as u8;
        let mut pattern_offset = self.reg_ctrl.get_background_pattern_table_address();
        pattern_offset += ((tile << 4) + tile_y) as u16;
        
        //println!("pattern offset {:X}", pattern_offset);
        
        let plane0 = self.vram.load_byte(pattern_offset);
        let plane1 = self.vram.load_byte(pattern_offset + 8);
        
        if plane0 != 0 || plane1 != 0 {
            println!("plane 0: {}, plane 1: {}", plane0, plane1);
        }
        
        let bit0 = (plane0 >> (7 - tile_x)) & 1;
        let bit1 = (plane1 >> (7 - tile_x)) & 1;
        
        let pattern_color = (bit1 << 1) | bit0;
        
        let palette_index = (attribute_color << 2) | pattern_color;
        let color_index = self.vram.load_byte(PALETTE_START + palette_index as u16);
        
        //println!("base: {:X}", base);
        //println!("tile: {}", tile);
        //println!("attr color: {}, pattern color: {}, palette index: {}", attribute_color, pattern_color, palette_index);
        
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
        // reading status resets these address latches
        self.reg_scroll.write_next = ScrollDirection::X;
        self.reg_addr.write_next = AddressByte::Upper;
        
        let status = *self.reg_status;
        // vblank is cleared after reading status
        self.reg_status.set_vblank(false);
        
        status
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
                
                // TODO: this second write affects scrolling
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
            PALETTE_START ... PALETTE_END => {
                // handle mirrored addresses
                let mut addr = addr as usize & 0x1f;
                if addr >= 0x10 && addr % 4 == 0 {
                    addr -= 0x10;
                }
                self.palette_index[addr as usize & 0x1f]
            },
            _ => panic!("Unknown PPU address {:04X}", addr)
        }
    }
    fn store_byte(&mut self, addr: u16, val: u8) {
        match addr {
            MAPPER_START ... MAPPER_END => {
                self.mapper.borrow_mut().store_byte_chr(addr, val);
            },
            NAMETABLE_START ... NAMETABLE_END => {
                //println!("nametable write {:X} {:X}", addr, val);
                self.nametable[addr as usize & (PPU_RAM_SIZE - 1)] = val;
            },
            PALETTE_START ... PALETTE_END => {
                // handle mirrored addresses
                let mut addr = addr as usize & 0x1f;
                if addr >= 0x10 && addr % 4 == 0 {
                    addr -= 0x10;
                }
                self.palette_index[addr as usize & 0x1f] = val
            },
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