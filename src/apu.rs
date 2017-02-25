use memory::Memory;

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

#[allow(dead_code)]
pub struct Apu {
    reg_pulse1: PulseRegister,
    reg_pulse2: PulseRegister,
    reg_triangle: TriangleRegister,
    reg_noise: NoiseRegister,
    reg_dmc: DmcRegister,

    enable_dmc: bool,
    enable_noise: bool,
    enable_triangle: bool,
    enable_pulse1: bool,
    enable_pulse2: bool,

    frame_mode: FrameMode,
    frame_interrupt: bool,

    dmc_interrupt: bool,
}

impl Apu {
    pub fn new() -> Apu {
        Apu {
            reg_pulse1: PulseRegister::default(),
            reg_pulse2: PulseRegister::default(),
            reg_triangle: TriangleRegister::default(),
            reg_noise: NoiseRegister::default(),
            reg_dmc: DmcRegister::default(),

            enable_dmc: false,
            enable_noise: false,
            enable_triangle: false,
            enable_pulse1: false,
            enable_pulse2: false,

            frame_mode: FrameMode::FourStep,
            frame_interrupt: false,

            dmc_interrupt: false,
        }
    }

    fn read_status(&mut self) -> u8 {
        let status = (self.dmc_interrupt as u8) << 7
                    | (self.frame_interrupt as u8) << 6
                    | (self.enable_dmc as u8) << 4
                    | (self.enable_noise as u8) << 3
                    | (self.enable_triangle as u8) << 2
                    | (self.enable_pulse2 as u8) << 1
                    | (self.enable_pulse1 as u8);
        
        // reading clears frame interrupt flag
        self.frame_interrupt = false;

        status
    }
    fn write_status(&mut self, val: u8) {
        self.enable_dmc = (val >> 4) & 1 == 1;
        self.enable_noise = (val >> 3) & 1 == 1;
        self.enable_triangle = (val >> 2) & 1 == 1;
        self.enable_pulse2 = (val >> 1) & 1 == 1;
        self.enable_pulse1 = val & 1 == 1;
    }

    fn write_frame_counter(&mut self, val: u8) {
        if (val >> 7) & 1 == 0 {
            self.frame_mode = FrameMode::FourStep;
        } else {
            self.frame_mode = FrameMode::FiveStep;
        }

        self.frame_interrupt = (val >> 6) & 1 != 1;
    }

    fn write_triangle(&mut self, addr: u16, val: u8) {
        match addr {
            0x4008 => {
                // CRRR RRRR	Length counter halt / linear counter control (C), linear counter load (R)
                self.reg_triangle.length_counter_halt = (val >> 7) & 1 == 1;
                self.reg_triangle.linear_counter = val & 0x7f;
            },
            0x4009 => {}, // unused
            0x400a => {
                // TTTT TTTT	Timer low (T)
                self.reg_triangle.timer = (self.reg_triangle.timer & 0xff00) | val as u16;
            },
            0x400b => {
                // LLLL LTTT	Length counter load (L), timer high (T)
                self.reg_triangle.length_counter = (val >> 3) & 0x1f;
                self.reg_triangle.timer = (self.reg_triangle.timer & 0x00ff) | ((val as u16 & 7) << 8);
            },
            _ => unreachable!(),
        }
    }

    fn write_noise(&mut self, addr: u16, val: u8) {
        match addr {
            0x400c => {
                // --LC VVVV	Envelope loop / length counter halt (L), constant volume (C), volume/envelope (V)
                self.reg_noise.length_counter_halt = (val >> 5) & 1 == 1;
                self.reg_noise.constant_volume = (val >> 4) & 1 == 1;
                self.reg_noise.envelope_volume = val & 0x0f;
            },
            0x400d => {}, // unused
            0x400e => {
                // L--- PPPP	Loop noise (L), noise period (P)
                self.reg_noise.noise_loop = (val >> 7) & 1 == 1;
                self.reg_noise.noise_period = val & 0x0f;
            },
            0x400f => {
                // LLLL L---	Length counter load (L)
                self.reg_noise.length_counter = val >> 3;
            },
            _ => unreachable!(),
        }
    }

    fn write_dmc(&mut self, addr: u16, val: u8) {
        match addr {
            0x4010 => {
                // IL-- RRRR	IRQ enable (I), loop (L), frequency (R)
                self.reg_dmc.interrupt_enable = (val >> 7) & 1 == 1;
                self.reg_dmc.dmc_loop = (val >> 6) & 1 == 1;
                self.reg_dmc.frequency = val & 0x0f;
            },
            0x4011 => {
                // -DDD DDDD	Load counter (D)
                self.reg_dmc.value = val & 0x7f;
            },
            0x4012 => {
                // AAAA AAAA	Sample address (A)
                self.reg_dmc.sample_address = val;
            },
            0x4013 => {
                // LLLL LLLL	Sample length (L)
                self.reg_dmc.sample_length = val;
            },
            _ => unreachable!(),
        }
    }
}

impl Memory for Apu {
    fn load_byte(&mut self, addr: u16) -> u8 {
        match addr {
            PULSE1_START ... PULSE1_END => 0, // write only
            PULSE2_START ... PULSE2_END => 0, // write only
            TRIANGLE_START ... TRIANGLE_END => 0, // write only
            NOISE_START ... NOISE_END => 0, // write only
            DMC_START ... DMC_END => 0, // write only
            STATUS => self.read_status(),
            FRAME_COUNTER => 0, // write only
            _ => panic!("Unknown APU register {:04X}", addr),
        }
    }
    fn store_byte(&mut self, addr: u16, val: u8) {
        match addr {
            PULSE1_START ... PULSE1_END => self.reg_pulse1.write(addr, val),
            PULSE2_START ... PULSE2_END => self.reg_pulse2.write(addr, val),
            TRIANGLE_START ... TRIANGLE_END => self.write_triangle(addr, val),
            NOISE_START ... NOISE_END => self.write_noise(addr, val),
            DMC_START ... DMC_END => self.write_dmc(addr, val),
            STATUS => self.write_status(val),
            FRAME_COUNTER => self.write_frame_counter(val),
            _ => panic!("Unknown APU register {:04X}", addr),
        }
    }
}

#[derive(Default)]
struct PulseRegister {
    duty_cycle: u8,
    length_counter_halt: bool,
    constant_volume: bool,
    envelope_volume: u8,
    sweep_enabled: bool,
    sweep_period: u8,
    sweep_negate: bool,
    sweep_shift: u8,
    timer: u16,
    length_counter: u8,
}

impl PulseRegister {
    fn write(&mut self, addr: u16, val: u8) {
        match addr & 3 {
            0 => {
                // DDLC VVVV	Duty (D), envelope loop / length counter halt (L), constant volume (C), volume/envelope (V)
                self.duty_cycle = (val >> 6) & 3;
                self.length_counter_halt = (val >> 5) & 1 == 1;
                self.constant_volume = (val >> 4) & 1 == 1;
                self.envelope_volume = val & 0x0f;
            },
            1 => {
                // EPPP NSSS	Sweep unit: enabled (E), period (P), negate (N), shift (S)
                self.sweep_enabled = (val >> 7) & 1 == 1;
                self.sweep_period = (val >> 4) & 7;
                self.sweep_negate = (val >> 3) & 1 == 1;
                self.sweep_shift = val & 7;
            },
            2 => {
                // TTTT TTTT	Timer low (T)
                self.timer = (self.timer & 0xff00) | val as u16;
            },
            3 => {
                // LLLL LTTT	Length counter load (L), timer high (T)
                self.length_counter = (val >> 3) & 0x1f;
                self.timer = (self.timer & 0x00ff) | ((val as u16 & 7) << 8);
            },
            _ => unreachable!(),
        }
    }
}

#[derive(Default)]
struct TriangleRegister {
    length_counter_halt: bool,
    linear_counter: u8,
    timer: u16,
    length_counter: u8,
}

#[derive(Default)]
struct NoiseRegister {
    length_counter_halt: bool,
    constant_volume: bool,
    envelope_volume: u8,
    noise_loop: bool,
    noise_period: u8,
    length_counter: u8,
}

#[derive(Default)]
struct DmcRegister {
    interrupt_enable: bool,
    dmc_loop: bool,
    frequency: u8,
    value: u8,
    sample_address: u8,
    sample_length: u8,
}

enum FrameMode {
    FourStep,
    FiveStep,
}
