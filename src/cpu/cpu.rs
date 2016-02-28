use super::super::memory;
use super::super::rom;

#[derive(Debug)]
pub struct Cpu {
    // accumulator
    reg_a: u8,
    // index X
    reg_x: u8,
    // index Y
    reg_y: u8,
    // program counter
    reg_pc: u16,
    // stack pointer
    reg_sp: u8,
    // status register
    reg_p: StatusRegister,
    
    memory: memory::Memory
}

impl Cpu {
    pub fn new(rom: rom::Rom) -> Cpu {
        Cpu {
            reg_a: 0,
            reg_x: 0,
            reg_y: 0,
            reg_pc: 0,
            reg_sp: 0xfd,
            reg_p: StatusRegister::from(0x34),
            memory: memory::Memory::new(rom)
        }
    }
    
    pub fn reset(&mut self) {
        self.reg_sp -= 3;
        self.reg_p.interrupt_disable = true;
        self.reg_pc = 0;
    }
}

#[derive(Debug)]
struct StatusRegister {
    carry: bool,
    zero: bool,
    interrupt_disable: bool,
    decimal_mode: bool,
    break_command: bool,
    overflow: bool,
    negative: bool
}

impl From<u8> for StatusRegister {
    fn from(value: u8) -> StatusRegister {
        StatusRegister {
            carry:              (value & (1 << 0)) != 0,
            zero:               (value & (1 << 1)) != 0,
            interrupt_disable:  (value & (1 << 2)) != 0,
            decimal_mode:       (value & (1 << 3)) != 0,
            break_command:      (value & (1 << 4)) != 0,
            // bit 5 unused
            overflow:           (value & (1 << 6)) != 0,
            negative:           (value & (1 << 7)) != 0
        }
    }
}