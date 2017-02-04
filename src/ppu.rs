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
    
    scanline: i16,
    
    vram: Vram,
    oam: Oam,

    tiles_to_render: Vec<Tile>,
    sprites_to_render: Vec<Sprite>,
    
    pub display_buffer: Box<[u8; SCREEN_WIDTH * SCREEN_HEIGHT * 3]>,
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
            
            scanline: -1,
            
            vram: Vram::new(mapper),
            oam: Oam([0; 256]),
            
            tiles_to_render: Vec::with_capacity(2),
            sprites_to_render: Vec::with_capacity(8),

            display_buffer: Box::new([0; SCREEN_WIDTH * SCREEN_HEIGHT * 3])
        }
    }
    
    // run PPU for one scanline
    pub fn run(&mut self) -> PpuRunResult {
        let mut result = PpuRunResult::default();
        
        if self.scanline < SCREEN_HEIGHT as i16 {
            //println!("RENDER SCANLINE {} v:{:04X} t:{:04X}", self.scanline, self.current_vram_address, self.temporary_vram_address);
            let show_background_left = self.reg_mask.show_background_left();
            let show_background = self.reg_mask.show_background();
            let show_sprites_left = self.reg_mask.show_sprites_left();
            let show_sprites = self.reg_mask.show_sprites();
            
            if show_background || show_sprites {
                self.render_scanline(show_background, show_background_left, show_sprites, show_sprites_left);

                // load sprites for the next scanline
                self.sprites_to_render();
            }
        }

        self.scanline += 1;
        
        if self.scanline == VBLANK_SCANLINE_START {
            self.reg_status.set_vblank(true);
            if self.reg_ctrl.generate_nmi() {
                result.vblank = true;
            }
            result.render_frame = true;
            //println!("[frame]");
        } else if self.scanline == VBLANK_SCANLINE_END {
            self.scanline = -1;
            self.reg_status.set_vblank(false);
        }
        
        result
    }
    
    // rendering
    
    fn render_scanline(&mut self, show_background: bool, show_background_left: bool, show_sprites: bool, show_sprites_left: bool) {
        let backdrop_index = self.vram.load_byte(PALETTE_START);
        let backdrop_color = self.color_from_palette(backdrop_index as usize);
        
        let y = self.scanline;

        // reset sprite 0 hit and sprite overflow on the beginning of the prerender scanline
        if y == -1 {
            self.reg_status.set_sprite_0_hit(false);
            self.reg_status.set_sprite_overflow(false);
        }

        for x in 0 .. PPU_CYCLES_PER_SCANLINE {            
            if y >= 0 {
                if x > 0 && x % 8 == 0 && x < SCREEN_WIDTH as u16 {
                    self.tiles_to_render.remove(0);

                    let tile = self.fetch_tile();
                    self.tiles_to_render.push(tile);
                }
                
                // get the background color
                let mut background_color = None;
                if x < 8 && show_background_left || x >= 8 && x < SCREEN_WIDTH as u16 && show_background {
                    background_color = self.get_background_pixel(x as u8);
                }

                // get sprite color
                let mut sprite_color = None;
                let mut sprite_priority = false;
                let mut is_sprite_0 = false;
                if x < 8 && show_sprites_left || x >= 8 && x < SCREEN_WIDTH as u16 && show_sprites {
                    let sprite_info = self.get_sprite_pixel(x as u8);
                    sprite_color = sprite_info.0;
                    sprite_priority = sprite_info.1;
                    is_sprite_0 = sprite_info.2;
                }

                // determine what color to use based on priority
                let color = match (background_color, sprite_color) {
                    (None, None) => backdrop_color,
                    (None, Some(sprite)) => sprite,
                    (Some(background), None) => background,
                    (Some(background), Some(sprite)) => {
                        if is_sprite_0 {
                            self.reg_status.set_sprite_0_hit(true);
                        }
                        if sprite_priority {
                            background
                        } else {
                            sprite
                        }
                    }
                };

                // write the pixel to the display buffer
                if x < SCREEN_WIDTH as u16 {
                    self.display_buffer[(y as usize * SCREEN_WIDTH + x as usize) * 3 + 0] = color.r;
                    self.display_buffer[(y as usize * SCREEN_WIDTH + x as usize) * 3 + 1] = color.g;
                    self.display_buffer[(y as usize * SCREEN_WIDTH + x as usize) * 3 + 2] = color.b;
                }
            }

            if show_background || show_sprites {
                // after all of the visible pixels are rendered, increment y
                if x == 256 {
                    // increment y in the current vram address and wrap if needed
                    // if fine Y < 7
                    if (self.current_vram_address & 0x7000) != 0x7000 {
                        // increment fine Y
                        self.current_vram_address += 0x1000;
                    } else {
                        // fine Y = 0
                        self.current_vram_address &= !0x7000;
                        // y = coarse Y
                        let mut coarse_y = (self.current_vram_address & 0x03e0) >> 5;
                        // row 29 is the last row of tiles in the nametable
                        if coarse_y == 29 {
                            coarse_y = 0;
                            // switch vertical nametable
                            self.current_vram_address ^= 0x0800;
                        } else if coarse_y == 31 {
                            // if coarse Y is incremented from 31, it wraps to 0
                            coarse_y = 0;
                        } else {
                            // increment coarse Y
                            coarse_y += 1;
                        }
                        // put coarse Y back into v
                        self.current_vram_address = (self.current_vram_address & !0x03e0) | (coarse_y << 5);
                    }
                } else if x == 257 {
                    // at the end of each scanline copy horizontal (x) bits of t to v
                    self.current_vram_address = (self.current_vram_address & !0x041f) | (self.temporary_vram_address & 0x041f);
                } else if x == 280 && y == -1 {
                    // copy vertical bits of t to v
                    self.current_vram_address = (self.current_vram_address & !0x7be0) | (self.temporary_vram_address & 0x7be0);
                } else if x == 321 {
                    // prefetch tiles for next line
                    if self.tiles_to_render.len() > 1 {
                        self.tiles_to_render.remove(0);
                    }
                    let tile = self.fetch_tile();
                    self.tiles_to_render.push(tile);
                } else if x == 329 {
                    if self.tiles_to_render.len() > 1 {
                        self.tiles_to_render.remove(0);
                    }

                    let tile = self.fetch_tile();
                    self.tiles_to_render.push(tile);
                }
            }
        }
    }

    fn fetch_tile(&mut self) -> Tile {
        let v = self.current_vram_address as u16;
        
        // from wiki - pull the tile address bits out of v
        let tile_index = self.vram.load_byte(0x2000 | (v & 0x0FFF)) as u16;

        let pattern_address = self.reg_ctrl.background_pattern_table_address() as u16;
        let fine_y = (v >> 12) & 7;
        
        let plane0 = self.vram.load_byte(pattern_address | (tile_index << 4) | fine_y);
        let plane1 = self.vram.load_byte(pattern_address | (tile_index << 4) | fine_y | 8);

        // from wiki - pull the attribute address bits out of v
        let attribute_addr = 0x23c0 | (v & 0x0c00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07);
        let attribute_byte = self.vram.load_byte(attribute_addr);

        // grab the low 2 bytes from the attribute table in the proper quadrant
        // the row is controlled by bit 6 in v (0x40), the column by bit 1 (0x02)
        let attribute_color = match (v & 0x40 == 0x40, v & 0x02 == 0x02) {
            (false, false) => attribute_byte >> 0 & 3, // top left
            (false, true) => attribute_byte >> 2 & 3, // top right
            (true, false) => attribute_byte >> 4 & 3, // bottom left
            (true, true) => attribute_byte >> 6 & 3, // bottom right
        };

        // increment v
        // if coarse X == 31
        if (v & 0x001f) == 31 {
            // coarse X = 0
            self.current_vram_address &= !(0x001f);
            // switch horizontal nametable
            self.current_vram_address ^= 0x0400;
            
        } else {
            // increment coarse X
            self.current_vram_address += 1;
        }

        Tile::new(plane0, plane1, attribute_color)
    }

    fn get_background_pixel(&mut self, current_pixel: u8) -> Option<RgbColor> {
        let x = current_pixel % 8;
        
        let tile_select = (x + self.fine_x) / 8;
        let offset = (x  + self.fine_x) % 8;

        let tile = &self.tiles_to_render[tile_select as usize];
        
        // bit0 of the color from the high byte, bit1 from the low
        let bit0 = (tile.plane0 >> 7 - offset) & 1;
        let bit1 = (tile.plane1 >> 7 - offset) & 1;

        let pattern_color = ((bit1 << 1) | bit0) as u8;

        if pattern_color == 0 {
            return None;
        }
        
        let palette_index = (tile.attribute_color << 2) | pattern_color;
        let color_index = self.vram.load_byte(PALETTE_START + palette_index as u16);
        
        Some(self.color_from_palette(color_index as usize))
    }

    fn get_sprite_pixel(&mut self, x: u8) -> (Option<RgbColor>, bool, bool) {
        for sprite in &self.sprites_to_render {
            if x >= sprite.x_position && (x < sprite.x_position + 8 || sprite.x_position >= (SCREEN_WIDTH - 8) as u8) {
                let mut pattern_base = 0x0000;
                match self.reg_ctrl.sprite_size() {
                    SpriteSize::Size8x16 => {
                        if sprite.tile_index & 1 != 0 {
                            pattern_base = 0x1000;
                        }
                    },
                    SpriteSize::Size8x8 => {
                        pattern_base = self.reg_ctrl.sprite_pattern_table_address();
                    },
                }
                
                let pattern_address = pattern_base | sprite.tile_index as u16;
                //println!("index {:02X}", sprite.tile_index);
                
                let flip_horizontal = ((sprite.attributes >> 6) & 1) == 1;
                let flip_vertical = ((sprite.attributes >> 7) & 1) == 1;

                let mut pixel_y_index = self.scanline as u16 - sprite.y_position as u16;
                if flip_vertical {
                    pixel_y_index = 7 - pixel_y_index;
                }
                
                let plane0 = self.vram.load_byte((pattern_address << 4) | pixel_y_index);
                let plane1 = self.vram.load_byte((pattern_address << 4) | pixel_y_index | 8);
                
                let mut pixel_x_index = 7 - (x - sprite.x_position);
                if flip_horizontal {
                    pixel_x_index = x - sprite.x_position;
                }

                let bit0 = (plane0 >> pixel_x_index) & 1;
                let bit1 = (plane1 >> pixel_x_index) & 1;

                let pattern_color = (bit1 << 1) | bit0;

                if pattern_color != 0 {                
                    let palette_index = (1 << 4) | ((sprite.attributes & 3) << 2) | pattern_color;

                    let color_index = self.vram.load_byte(PALETTE_START + palette_index as u16);

                    //println!("sprite {} - color index {:02X}", sprite.x_position, color_index);
                    let priority = ((sprite.attributes >> 5) & 1) == 1;
                    
                    let color = Some(self.color_from_palette(color_index as usize));

                    return (color, priority, sprite.index == 0);
                }
            }
        }
        
        (None, false, false)
    }

    fn sprites_to_render(&mut self) {
        self.sprites_to_render.clear();

        if self.scanline == -1 {
            return;
        }

        // this is loading sprites for the next scanline
        let y = self.scanline as u8 + 1;
        
        // oam holds 64 4-byte sprites
        for n in 0 .. 64 {
            let sprite_index = 4 * n;
            
            // grab the 4 bytes from oam
            let sprite_bytes = &self.oam[sprite_index..sprite_index + 4];
            let sprite = Sprite::new(sprite_bytes[0], sprite_bytes[1], sprite_bytes[2], sprite_bytes[3], n as u8);

            if y >= sprite.y_position && y < sprite.y_position + 8 && sprite.y_position < 0xef {
                if self.sprites_to_render.len() == 8 {
                    // note: this isn't quite correct due to a hardware bug
                    self.reg_status.set_sprite_overflow(true);
                    break;
                }
                self.sprites_to_render.push(sprite);
                //println!("{:?}", sprite);
            }
        }
    }
    
    fn color_from_palette(&self, index: usize) -> RgbColor {
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
        self.current_vram_address += self.reg_ctrl.vram_address_increment();
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
        self.temporary_vram_address = (self.temporary_vram_address & 0x73ff) | ((val as u16 & 3) << 10);
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
                // set the bottom five bits to the top 5 bits of the value
                self.temporary_vram_address = (self.temporary_vram_address & 0x7fe0) | (val as u16 >> 3);
            },
            AddressByte::Lower => {
                self.write_toggle = AddressByte::Upper;
                
                let val = val as u16;
                self.temporary_vram_address = (self.temporary_vram_address & 0x7cff) | ((val & 0b1100_0000) << 2);
                self.temporary_vram_address = (self.temporary_vram_address & 0x7f1f) | ((val & 0b0011_1000) << 2);
                self.temporary_vram_address = (self.temporary_vram_address & 0x0fff) | ((val & 0b0000_0111) << 12);
            }
        }
    }
    
    fn write_addr(&mut self, val: u8) {
        match self.write_toggle {
            AddressByte::Upper => {
                self.write_toggle = AddressByte::Lower;
                
                self.temporary_vram_address = (self.temporary_vram_address & 0x00ff) | ((val as u16) << 8);
                
                // bit 14 is cleared
                // if self.scanline < VBLANK_SCANLINE_START && self.scanline >= 0 
                //     && (self.reg_mask.show_background() || self.reg_mask.show_sprites()) {
                    self.temporary_vram_address &= !(1 << 14);
                // }
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
        self.current_vram_address += self.reg_ctrl.vram_address_increment();
    }
    
    // fn trace_read(addr: u16) {
    //     println!("R: {:04X}", addr);
    // }
    
    // fn trace_write(addr: u16, val: u8) {
    //     println!("W: {:04X} -> {:02X}", addr, val);
    // }
}

// cpu memory interface to ppu registers
impl Memory for Ppu {
    fn load_byte(&mut self, addr: u16) -> u8 {
        // Ppu::trace_read(addr);
        
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
        // Ppu::trace_write(addr, val);
        
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
        let mirroring = mapper.borrow().mirroring();
        Vram {
            mapper: mapper,
            nametable: [0; PPU_RAM_SIZE],
            palette: [0; 0x20],
            mirroring: mirroring
        }
    }
    
    fn nametable_addr(&self, addr: u16) -> usize {
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
        // Ppu::trace_read(addr);
        
        match addr {
            MAPPER_START ... MAPPER_END => self.mapper.borrow_mut().load_byte_chr(addr),
            NAMETABLE_START ... NAMETABLE_END => {
                let nametable_addr = self.nametable_addr(addr);
                self.nametable[nametable_addr]
            },
            PALETTE_START ... PALETTE_END => {
                // handle mirrored addresses
                let mut addr = addr as usize & 0x1f;
                if addr & 0x13 == 0x10 {
                    addr ^= 0x10;
                }
                
                self.palette[addr]
            },
            0x4000 ... 0x7fff => self.nametable[addr as usize & (PPU_RAM_SIZE - 1)],
            _ => panic!("Unknown PPU address {:04X}", addr)
        }
    }
    fn store_byte(&mut self, addr: u16, val: u8) {
        // Ppu::trace_write(addr, val);
        
        match addr {
            MAPPER_START ... MAPPER_END => {
                self.mapper.borrow_mut().store_byte_chr(addr, val);
            },
            NAMETABLE_START ... NAMETABLE_END => {
                let nametable_addr = self.nametable_addr(addr);
                
                //println!("nametable write {:04X} {:04X} {:02X}", addr, nametable_addr, val);
                
                self.nametable[nametable_addr] = val;
            },
            PALETTE_START ... PALETTE_END => {
                //println!("palette write {:04X} {:02X}", addr, val);
                // handle mirrored addresses
                let mut addr = addr as usize & 0x1f;
                if addr & 0x13 == 0x10 {
                    addr ^= 0x10;
                }
                
                self.palette[addr] = val;
                //println!("{:?}", self.palette);
            },
            0x4000 ... 0x7fff => {
                //println!("nametable write {:04X} {:02X}", addr, val);
                self.nametable[addr as usize & (PPU_RAM_SIZE - 1)] = val;
            }
            _ => panic!("Unknown PPU address {:04X}", addr)
        }
    }
}

struct Oam([u8; 256]);

impl Memory for Oam {
    fn load_byte(&mut self, addr: u16) -> u8 {
        self.0[addr as usize & 0x00ff]
    }
    fn store_byte(&mut self, addr: u16, val: u8) {
        self.0[addr as usize & 0x00ff] = val;
    }
}

impl Deref for Oam {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Copy, Clone)]
struct StatusRegister(u8);

impl StatusRegister {    
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
    fn vram_address_increment(&self) -> u16 {
        if self.0 & 0b0000_0100 != 0 { 32 } else { 1 }
    }
    
    fn sprite_pattern_table_address(&self) -> u16 {
        if self.0 & 0b0000_1000 != 0 { 0x1000 } else { 0x0000 }
    }
    
    fn background_pattern_table_address(&self) -> u16 {
        if self.0 & 0b0001_0000 != 0 { 0x1000 } else { 0x0000 }
    }
    
    fn sprite_size(&self) -> SpriteSize {
        if self.0 & 0b0010_0000 != 0 { SpriteSize::Size8x16 } else { SpriteSize::Size8x8 }
    }
    
    fn generate_nmi(&self) -> bool {
        self.0 & 0b1000_0000 != 0
    }
}

impl Deref for CtrlRegister {
    type Target = u8;
    
    fn deref(&self) -> &u8 {
        &self.0
    }
}

#[derive(Copy, Clone)]
struct MaskRegister(u8);

impl MaskRegister {    
    fn show_background_left(&self) -> bool {
        self.0 & 0b0000_0010 != 0
    }
    
    fn show_sprites_left(&self) -> bool {
        self.0 & 0b0000_0100 != 0
    }
    
    fn show_background(&self) -> bool {
        self.0 & 0b0000_1000 != 0
    }
    
    fn show_sprites(&self) -> bool {
        self.0 & 0b0001_0000 != 0
    }

    // color manipulation stuff unused for now
    // fn greyscale(&self) -> bool {
    //     self.0 & 0b0000_0001 != 0
    // }
    
    // fn emphasize_red(&self) -> bool {
    //     self.0 & 0b0010_0000 != 0
    // }
    
    // fn emphasize_green(&self) -> bool {
    //     self.0 & 0b0100_0000 != 0
    // }
    
    // fn emphasize_blue(&self) -> bool {
    //     self.0 & 0b1000_0000 != 0
    // }
}

impl Deref for MaskRegister {
    type Target = u8;
    
    fn deref(&self) -> &u8 {
        &self.0
    }
}

#[derive(Debug, Copy, Clone)]
struct Sprite {
    y_position: u8,
    tile_index: u8,
    attributes: u8,
    x_position: u8,
    index: u8,
}

impl Sprite {
    fn new(y_position: u8, tile_index: u8, attributes: u8, x_position: u8, index: u8) -> Sprite {
        Sprite {
            y_position: y_position,
            tile_index: tile_index,
            attributes: attributes,
            x_position: x_position,
            index: index,
        }
    }
}

struct Tile {
    plane0: u8,
    plane1: u8,
    attribute_color: u8
}

impl Tile {
    fn new (plane0: u8, plane1: u8, attribute_color: u8) -> Tile {
        Tile {
            plane0: plane0,
            plane1: plane1,
            attribute_color: attribute_color,
        }
    }
}

enum SpriteSize {
    Size8x8,
    Size8x16
}

enum AddressByte {
    Upper,
    Lower
}