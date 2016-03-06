use std::ops::Deref;
use memory::Memory;

const PPU_STATUS: u16 = 0x02;

pub struct Ppu {
    reg_ctrl: u8,
    reg_mask: u8,
    reg_status: StatusRegister,
    reg_oam_addr: u8,
    reg_oam_data: u8,
    reg_scroll: u8,
    reg_addr: u8,
    reg_data: u8,
    reg_oam_dma: u8
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            reg_ctrl: 0,
            reg_mask: 0,
            reg_status: StatusRegister(0),
            reg_oam_addr: 0,
            reg_oam_data: 0,
            reg_scroll: 0,
            reg_addr: 0,
            reg_data: 0,
            reg_oam_dma: 0
        }
    }
    
    pub fn run(&mut self) {
        self.reg_status.set_vblank(true);
    }
}

impl Memory for Ppu {
    fn load_byte(&mut self, addr: u16) -> u8 {
        // repeats every 7 bytes
        match addr & 7 {
            PPU_STATUS => {
                let val = *self.reg_status;
                self.reg_status.set_vblank(false);
                val
            },
            _ => panic!("Unknown PPU address {:04X}", addr)
        }
    }
    fn store_byte(&mut self, addr: u16, val: u8) {
        
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