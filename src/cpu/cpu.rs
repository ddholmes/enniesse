use super::super::memory::MemoryMap;
use super::super::rom::Rom;

const NMI_VECTOR: u16 = 0xfffa;
const RESET_VECTOR: u16 = 0xfffc;
const BRK_VECTOR: u16 = 0xfffe;

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
    
    memory_map: MemoryMap
}

impl Cpu {
    pub fn new(rom: Box<Rom>) -> Cpu {
        Cpu {
            reg_a: 0,
            reg_x: 0,
            reg_y: 0,
            reg_pc: 0xc000,
            reg_sp: 0xfd,
            reg_p: StatusRegister::from(0x24),
            memory_map: MemoryMap::new(rom)
        }
    }
    
    pub fn reset(&mut self) {
        //self.reg_sp -= 3;
        //self.reg_p.interrupt_disable = true;
        self.reg_pc = self.memory_map.load_word(RESET_VECTOR);
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