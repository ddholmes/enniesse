use memory::Memory;

use std::ops::Deref;

const PULSE1_START: u16 = 0x4000;
const PULSE1_END: u16 = 0x4003;
const PULSE2_START: u16 = 0x4004;
const PULSE2_END: u16 = 0x4007;
const TRIANGLE_START: u16 = 0x4008;
const TRIANGLE_END: u16 = 0x400b;
const NOISE_START: u16 = 0x400c;
const NOISE_END: u16 = 0x400f;
const DMC_START: u16 = 0x4010;
const DMC_END: u16 = 0x4013;
const STATUS: u16 = 0x4015;
const FRAME_COUNTER: u16 = 0x4017;

pub struct Apu {
    reg_pulse1: PulseRegister,
    reg_pulse2: PulseRegister,
    reg_triangle: TriangleRegister,
    reg_noise: NoiseRegister,
    reg_dmc: DmcRegister,
    reg_status: StatusRegister,
    reg_frame_counter: FrameCounterRegister
}

impl Apu {
    pub fn new() -> Apu {
        Apu {
            reg_pulse1: PulseRegister,
            reg_pulse2: PulseRegister,
            reg_triangle: TriangleRegister,
            reg_noise: NoiseRegister,
            reg_dmc: DmcRegister,
            reg_status: StatusRegister(0),
            reg_frame_counter: FrameCounterRegister(0)
        }
    }
}

impl Memory for Apu {
    fn load_byte(&mut self, addr: u16) -> u8 {
        match addr {
            PULSE1_START ... PULSE1_END => 0, // TODO: implement
            PULSE2_START ... PULSE2_END => 0, // TODO: implement
            TRIANGLE_START ... TRIANGLE_END => 0, // TODO: implement
            NOISE_START ... NOISE_END => 0, // TODO: implement
            DMC_START ... DMC_END => 0, // TODO: implement
            STATUS => *self.reg_status,
            FRAME_COUNTER => *self.reg_frame_counter,
            _ => panic!("Unknown APU register {:04X}", addr)
        }
    }
    fn store_byte(&mut self, addr: u16, val: u8) {
        match addr {
            PULSE1_START ... PULSE1_END => {}, // TODO: implement
            PULSE2_START ... PULSE2_END => {}, // TODO: implement
            TRIANGLE_START ... TRIANGLE_END => {}, // TODO: implement
            NOISE_START ... NOISE_END => {}, // TODO: implement
            DMC_START ... DMC_END => {}, // TODO: implement
            STATUS => self.reg_status = StatusRegister(val),
            FRAME_COUNTER => self.reg_frame_counter = FrameCounterRegister(val),
            _ => panic!("Unknown APU register {:04X}", addr)
        }
    }
}

struct PulseRegister;
struct TriangleRegister;
struct NoiseRegister;
struct DmcRegister;
struct StatusRegister(u8);
impl Deref for StatusRegister {
    type Target = u8;
    
    fn deref(&self) -> &u8 {
        &self.0
    }
}

struct FrameCounterRegister(u8);
impl Deref for FrameCounterRegister {
    type Target = u8;
    
    fn deref(&self) -> &u8 {
        &self.0
    }
}