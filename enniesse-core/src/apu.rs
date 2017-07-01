use mapper::Mapper;
use memory::Memory;
use std::rc::Rc;
use std::cell::RefCell;

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

const PULSE_SEQUENCE: [[u8; 8]; 4] = [
    [0, 1, 0, 0, 0, 0, 0, 0],
	[0, 1, 1, 0, 0, 0, 0, 0],
	[0, 1, 1, 1, 1, 0, 0, 0],
	[1, 0, 0, 1, 1, 1, 1, 1],
];

const LENGTH_SEQUENCE: [u8; 32] = [
    10,254, 20,  2, 40,  4, 80,  6, 160,  8, 60, 10, 14, 12, 26, 14,
    12, 16, 24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30
];

const TRIANGLE_SEQUENCE: [u8; 32] = [
    15, 14, 13, 12, 11, 10,  9,  8,  7,  6,  5,  4,  3,  2,  1,  0,
     0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14, 15
];

// NTSC
const NOISE_TABLE: [u16; 16] = [
    4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068
];

// NTSC
const DMC_RATES: [u16; 16] = [
    428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106,  84,  72,  54
];

pub struct Apu {
    cycle: u64,

    pulse1: PulseChannel,
    pulse2: PulseChannel,
    triangle: TriangleChannel,
    noise: NoiseChannel,
    dmc: DmcChannel,

    frame_mode: FrameMode,
    
    pub frame_interrupt: bool,
    pub dmc_interrupt: bool,

    mapper: Rc<RefCell<Box<Mapper>>>,
}

impl Apu {
    pub fn new(mapper: Rc<RefCell<Box<Mapper>>>) -> Apu {
        Apu {
            cycle: 0,

            pulse1: PulseChannel::new(1),
            pulse2: PulseChannel::new(2),
            triangle: TriangleChannel::default(),
            noise: NoiseChannel::new(),
            dmc: DmcChannel::default(),

            frame_mode: FrameMode::FourStep,
            frame_interrupt: false,

            dmc_interrupt: false,

            mapper: mapper,
        }
    }

    pub fn step(&mut self) {
        // step each channel, get and mix output
        
        self.cycle += 1;
    }

    fn read_status(&mut self) -> u8 {
        let status = (self.dmc_interrupt as u8) << 7
                    | (self.frame_interrupt as u8) << 6
                    | ((self.dmc.bytes_remaining > 0) as u8) << 4
                    | ((self.noise.length_counter.counter > 0) as u8) << 3
                    | ((self.triangle.length_counter.counter > 0) as u8) << 2
                    | ((self.pulse2.length_counter.counter > 0) as u8) << 1
                    | ((self.pulse1.length_counter.counter > 0) as u8);
        
        // reading clears frame interrupt flag
        self.frame_interrupt = false;

        status
    }

    fn write_status(&mut self, val: u8) {
        self.dmc.enabled = (val >> 4) & 1 == 1;
        self.noise.enabled = (val >> 3) & 1 == 1;
        self.triangle.enabled = (val >> 2) & 1 == 1;
        self.pulse2.enabled = (val >> 1) & 1 == 1;
        self.pulse1.enabled = val & 1 == 1;

        if !self.dmc.enabled {
            self.dmc.bytes_remaining = 0;
        } else if self.dmc.bytes_remaining == 0 {
            self.dmc.restart_sample();
        }

        if !self.noise.enabled {
            self.noise.length_counter.counter = 0;
        }

        if !self.triangle.enabled {
            self.triangle.length_counter.counter = 0;
        }

        if !self.pulse2.enabled {
            self.pulse2.length_counter.counter = 0;
        }

        if !self.pulse1.enabled {
            self.pulse1.length_counter.counter = 0;
        }

        self.dmc_interrupt = false;
    }

    fn write_frame_counter(&mut self, val: u8) {
        if (val >> 7) & 1 == 0 {
            self.frame_mode = FrameMode::FourStep;
        } else {
            self.frame_mode = FrameMode::FiveStep;
        }

        self.frame_interrupt = (val >> 6) & 1 != 1;
    }

    // channel steps

    // Pulse
    fn step_pulse(&mut self) {
        let pulse1 = &mut self.pulse1;
        let pulse2 = &mut self.pulse2;

        let clock1 = pulse1.timer.tick();
        if clock1 {
            pulse1.sequence_index = (pulse1.sequence_index + 1) % 8;
        }

        let clock2 = pulse2.timer.tick();
        if clock2 {
            pulse2.sequence_index = (pulse2.sequence_index + 1) % 8;
        }
    }

    // Triangle
    fn step_triangle(&mut self) {
        let triangle = &mut self.triangle;

        let clock = triangle.timer.tick();
        if clock && triangle.length_counter.counter > 0 && triangle.linear_counter > 0 {
            triangle.sequence_index = (triangle.sequence_index + 1) % 32;
        }
    }

    // Noise
    fn step_noise(&mut self) {
        let noise = &mut self.noise;

        let clock = noise.timer.tick();

        if clock {
            let bit = if noise.mode_flag { 6 } else { 1 };

            // feedback is the XOR of bit 0 and either bit6 or bit1, depending on the mode
            let feedback = (noise.shift_register & 1) ^ ((noise.shift_register >> bit) & 1);

            // shift right
            noise.shift_register >>= 1;
            
            // bit 14 is set to the feedback calculated above
            noise.shift_register |= feedback << 14;
        }
    }
    
    // DMC
    fn step_dmc(&mut self) {
        self.step_dmc_memory_reader();

        let clock = self.dmc.timer.tick();
        if clock {
            self.step_dmc_output();
        }
    }

    fn step_dmc_memory_reader(&mut self) {
        let dmc = &mut self.dmc;

        if dmc.sample_buffer == 0 && dmc.bytes_remaining > 0 {
            // TODO: CPU stalls for up to 4 cycles
            
            dmc.sample_buffer = self.mapper.borrow_mut().load_byte_prg(dmc.current_address);
            
            dmc.current_address += 1;
            // address wraps around to 0x8000
            if dmc.current_address == 0 {
                dmc.current_address = 0x8000;
            }

            dmc.bytes_remaining -= 1;
            if dmc.bytes_remaining == 0 {
                if dmc.dmc_loop {
                    dmc.restart_sample();
                } else if dmc.interrupt_enable {
                    self.dmc_interrupt = true;
                }
            }
        }
    }

    fn step_dmc_output(&mut self) {
        let dmc = &mut self.dmc;

        if dmc.output_bits_remaining == 0 {
            dmc.output_bits_remaining = 8;
            if dmc.sample_buffer != 0 {
                dmc.output_shift_register = dmc.sample_buffer;
                dmc.sample_buffer = 0;
            }
        }

        if dmc.output_shift_register & 1 == 1 {
            if dmc.output_level <= 125 {
                dmc.output_level += 2;
            }
        } else {
            if dmc.output_level >= 2 {
                dmc.output_level -= 2;
            }
        }

        dmc.output_shift_register >>= 1;
        dmc.output_bits_remaining -= 1;
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
            PULSE1_START ... PULSE1_END => self.pulse1.write(addr, val),
            PULSE2_START ... PULSE2_END => self.pulse2.write(addr, val),
            TRIANGLE_START ... TRIANGLE_END => self.triangle.write(addr, val),
            NOISE_START ... NOISE_END => self.noise.write(addr, val),
            DMC_START ... DMC_END => self.dmc.write(addr, val),
            STATUS => self.write_status(val),
            FRAME_COUNTER => self.write_frame_counter(val),
            _ => panic!("Unknown APU register {:04X}", addr),
        }
    }
}

struct PulseChannel {
    channel: u8,
    enabled: bool,
    duty_cycle: u8,
    length_counter: LengthCounter,
    envelope: Envelope,
    sweep_enabled: bool,
    sweep_period: u8,
    sweep_negate: bool,
    sweep_shift: u8,
    sweep_reload: bool,
    sweep_counter: u8,
    timer: Timer,
    sequence_index: u8,
}

impl PulseChannel {
    fn new(channel: u8) -> PulseChannel {
        PulseChannel {
            channel: channel,
            enabled: false,
            duty_cycle: 0,
            length_counter: LengthCounter::default(),
            envelope: Envelope::default(),
            sweep_enabled: false,
            sweep_period: 0,
            sweep_negate: false,
            sweep_shift: 0,
            sweep_reload: false,
            sweep_counter: 0,
            timer: Timer::default(),
            sequence_index: 0,
        }
    }

    fn write(&mut self, addr: u16, val: u8) {
        match addr & 3 {
            0 => {
                // DDLC VVVV	Duty (D), envelope loop / length counter halt (L), constant volume (C), volume/envelope (V)
                self.duty_cycle = (val >> 6) & 3;
                self.length_counter.halt = (val >> 5) & 1 == 1;

                self.envelope.write(val);
            },
            1 => {
                // EPPP NSSS	Sweep unit: enabled (E), period (P), negate (N), shift (S)
                self.sweep_enabled = (val >> 7) & 1 == 1;
                // sweep period is P + 1
                self.sweep_period = ((val >> 4) & 7) + 1;
                self.sweep_negate = (val >> 3) & 1 == 1;
                self.sweep_shift = val & 7;
            },
            2 => {
                // TTTT TTTT	Timer low (T)
                self.timer.write_low(val);
            },
            3 => {
                // LLLL LTTT	Length counter load (L), timer high (T)
                if self.enabled {
                    self.length_counter.load(val);
                }

                self.timer.write_high(val & 7);

                self.envelope.start_flag = true;
                self.sequence_index = 0;
            },
            _ => unreachable!(),
        }
    }

    fn clock_sweep(&mut self) {
        if self.sweep_reload {
            if self.sweep_counter == 0 && self.sweep_enabled {
                self.sweep_timer();
            }
            
            self.sweep_counter = self.sweep_period;
            self.sweep_reload = false;
        } else if self.sweep_counter > 0 {
            self.sweep_counter -= 1;
        } else if self.sweep_enabled {
            self.sweep_counter = self.sweep_period;
            self.sweep_timer();
        }
    }

    fn sweep_timer(&mut self) {
        let mut change = self.timer.period >> self.sweep_shift;

        if self.sweep_negate {
            self.timer.period -= change;
            // pulse1 adds the one's complement, which subtracts one more
            if self.channel == 1 {
                self.timer.period -= 1;
            }
        } else {
            self.timer.period += change;
        }
    }

    fn output(&self) -> u8 {
        if !self.enabled
            || self.length_counter.counter == 0
            || self.timer.period < 8 || self.timer.period > 0x7ff
            || PULSE_SEQUENCE[self.duty_cycle as usize][self.sequence_index as usize] == 0 {
            return 0;
        }

        self.envelope.volume()
    }
}

#[derive(Default)]
struct TriangleChannel {
    enabled: bool,
    length_counter: LengthCounter,
    linear_counter: u8,
    linear_counter_control_flag: bool,
    linear_counter_reload_flag: bool,
    linear_counter_reload_value: u8,
    timer: Timer,
    sequence_index: u8,
}

impl TriangleChannel {
    fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x4008 => {
                // CRRR RRRR	Length counter halt / linear counter control (C), linear counter load (R)
                self.length_counter.halt = (val >> 7) & 1 == 1;
                self.linear_counter_reload_value = val & 0x7f;
                self.linear_counter_control_flag = self.length_counter.halt;
            },
            0x4009 => {}, // unused
            0x400a => {
                // TTTT TTTT	Timer low (T)
                self.timer.write_low(val);
            },
            0x400b => {
                // LLLL LTTT	Length counter load (L), timer high (T)
                if self.enabled {
                    self.length_counter.load(val);
                }

                self.timer.write_high(val & 7);

                self.linear_counter_reload_flag = true;
            },
            _ => unreachable!(),
        }
    }

    fn step_linear_counter(&mut self) {
        if self.linear_counter_reload_flag {
            self.linear_counter = self.linear_counter_reload_value;
        } else if self.linear_counter > 0 {
            self.linear_counter -= 1;
        }

        if !self.linear_counter_control_flag {
            self.linear_counter_reload_flag = false;
        }
    }

    fn output(&self) -> u8 {
        if !self.enabled {
            return 0;
        }

        TRIANGLE_SEQUENCE[self.sequence_index as usize]
    }
}

struct NoiseChannel {
    enabled: bool,
    length_counter: LengthCounter,
    envelope: Envelope,
    timer: Timer,
    mode_flag: bool,
    shift_register: u16,
}

impl NoiseChannel {
    fn new() -> NoiseChannel {
        NoiseChannel {
            enabled: false,
            length_counter: LengthCounter::default(),
            envelope: Envelope::default(),
            timer: Timer::default(),
            mode_flag: false,
            shift_register: 1,
        }
    }

    fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x400c => {
                // --LC VVVV	Envelope loop / length counter halt (L), constant volume (C), volume/envelope (V)
                self.length_counter.halt = (val >> 5) & 1 == 1;
                
                self.envelope.write(val);
            },
            0x400d => {}, // unused
            0x400e => {
                // L--- PPPP	Loop noise (L), noise period (P)
                self.mode_flag = (val >> 7) & 1 == 1;
                self.timer.period = NOISE_TABLE[val as usize & 0x0f];
            },
            0x400f => {
                // LLLL L---	Length counter load (L)
                if self.enabled {
                    self.length_counter.load(val);
                }

                self.envelope.start_flag = true;
            },
            _ => unreachable!(),
        }
    }

    fn output(&self) -> u8 {
        if !self.enabled
            || self.shift_register & 1 == 1
            || self.length_counter.counter == 0 {
            return 0;
        }

        self.envelope.volume()
    }
}

#[derive(Default)]
struct DmcChannel {
    enabled: bool,
    interrupt_enable: bool,
    dmc_loop: bool,
    sample_address: u16,
    sample_length: u16,
    sample_buffer: u8,
    current_address: u16,
    bytes_remaining: u16,
    output_level: u8,
    output_shift_register: u8,
    output_bits_remaining: u8,
    timer: Timer,
}

impl DmcChannel {
    fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x4010 => {
                // IL-- RRRR	IRQ enable (I), loop (L), frequency (R)
                self.interrupt_enable = (val >> 7) & 1 == 1;
                self.dmc_loop = (val >> 6) & 1 == 1;
                self.timer.period = DMC_RATES[val as usize & 0x0f];
            },
            0x4011 => {
                // -DDD DDDD	Load counter (D)
                self.output_level = val & 0x7f;
            },
            0x4012 => {
                // AAAA AAAA	Sample address (A)
                // Sample address = %11AAAAAA.AA000000
                self.sample_address = 0xc000 + (val as u16 * 64);
            },
            0x4013 => {
                // LLLL LLLL	Sample length (L)
                // Sample length = %LLLL.LLLL0001
                self.sample_length = (val as u16 * 16) + 1;
            },
            _ => unreachable!(),
        }
    }

    fn restart_sample(&mut self) {
        self.current_address = self.sample_address;
        self.bytes_remaining = self.sample_length;
    }

    fn output(&self) -> u8 {
        self.output_level
    }
}

#[derive(Default)]
struct Envelope {
    start_flag: bool,
    loop_flag: bool,
    use_constant_volume: bool,
    divider_counter: u8,
    constant_volume: u8,
    decay_level: u8,
}

impl Envelope {
    fn write(&mut self, val: u8) {
        self.loop_flag = (val >> 5) & 1 == 1;
        self.use_constant_volume = (val >> 4) & 1 == 1;
        self.constant_volume = val & 0x0f;
    }
    
    fn clock(&mut self) {
        if self.start_flag {
            self.start_flag = false;
            self.decay_level = 15;
            self.divider_counter = self.constant_volume;
        } else if self.divider_counter == 0 {
            self.divider_counter = self.constant_volume;

            if self.decay_level > 0 {
                self.decay_level -= 1;
            } else if self.loop_flag {
                self.decay_level = 15;
            }
        } else {
            self.divider_counter -= 1;
        }
    }

    fn volume(&self) -> u8 {
        if self.use_constant_volume {
            self.constant_volume
        } else {
            self.decay_level
        }
    }
}

#[derive(Default)]
struct LengthCounter {
    halt: bool,
    counter: u8,
}

impl LengthCounter {
    fn clock(&mut self) {
        if !self.halt && self.counter > 0 {
            self.counter -= 1;
        }
    }

    fn load(&mut self, val: u8) {
        self.counter = LENGTH_SEQUENCE[(val >> 3) as usize];
    }
}

#[derive(Default)]
struct Timer {
    period: u16,
    value: u16,
}

impl Timer {
    // returns true if the timer should generate a clock
    fn tick(&mut self) -> bool {
        if self.value == 0 {
            self.value = self.period;
            return true;
        }

        self.value -= 1;
        false
    }

    fn write_low(&mut self, val: u8) {
        self.value = (self.value & 0xff00) | val as u16;
    }

    fn write_high(&mut self, val: u8) {
        self.value = (self.value & 0x00ff) | ((val as u16) << 8);
    }
}

enum FrameMode {
    FourStep,
    FiveStep,
}
