use memory::Memory;

pub struct Apu {
    reg_pulse: [Pulse; 2],
    reg_triangle: Triangle,
    reg_noise: Noise,
    reg_dmc: Dmc,
    reg_status: Status
}

impl Apu {
    pub fn new() -> Apu {
        Apu {
            reg_pulse: [Pulse, Pulse],
            reg_triangle: Triangle,
            reg_noise: Noise,
            reg_dmc: Dmc,
            reg_status: Status
        }
    }
}

impl Memory for Apu {
    fn load_byte(&mut self, addr: u16) -> u8 {
        0
    }
    fn store_byte(&mut self, addr: u16, val: u8) {
        
    }
}

struct Pulse;
struct Triangle;
struct Noise;
struct Dmc;
struct Status;