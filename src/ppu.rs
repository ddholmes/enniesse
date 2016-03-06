use memory::Memory;

pub struct Ppu {
    reg_ctrl: u8,
    reg_mask: u8,
    reg_status: u8,
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
            reg_status: 0,
            reg_oam_addr: 0,
            reg_oam_data: 0,
            reg_scroll: 0,
            reg_addr: 0,
            reg_data: 0,
            reg_oam_dma: 0
        }
    }
}

impl Memory for Ppu {
    fn load_byte(&mut self, addr: u16) -> u8 {
        0
    }
    fn store_byte(&mut self, addr: u16, val: u8) {
        
    }
}