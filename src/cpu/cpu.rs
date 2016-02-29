use super::super::memory::MemoryMap;
use super::super::rom::Rom;
use super::opcode::Opcode;

use std::fmt;
use num::FromPrimitive;

const NMI_VECTOR: u16 = 0xfffa;
const RESET_VECTOR: u16 = 0xfffc;
const BRK_VECTOR: u16 = 0xfffe;

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
    
    memory_map: MemoryMap,
    
    current_instruction: u8
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
            memory_map: MemoryMap::new(rom),
            current_instruction: 0
        }
    }
    
    pub fn reset(&mut self) {
        // TODO: accurately model reset
        
        // TODO: uncomment to start in the appropriate place
        //self.reg_pc = self.memory_map.load_word(RESET_VECTOR);
    }
    
    pub fn run_instruction(&mut self) {
        self.print_state();
        
        let opcode = Self::get_opcode(self.load_byte_from_pc());
        
        self.execute_instruction(opcode);
    }
    
    fn execute_instruction(&mut self, opcode: Opcode) {
        match opcode {
            Opcode::Jmp => {
                self.reg_pc = self.load_word_from_pc();
            },
            Opcode::Ldx => {
                let val = self.load_byte_from_pc();
                self.reg_p.negative = (val & 0b1000_000) != 0;
                self.reg_p.zero = (val == 0);
                self.reg_x = val;
            },
            Opcode::Stx => {
                let target = self.load_byte_from_pc();
                // TODO: actually store
                //self.memory_map.store_byte(self.reg_x);
            }
        }
    }
    
    fn load_byte_from_pc(&mut self) -> u8 {
        let value = self.memory_map.load_byte(self.reg_pc);
        self.reg_pc += 1;
        
        value
    }
    
    fn load_word_from_pc(&mut self) -> u16 {
        let value = self.memory_map.load_word(self.reg_pc);
        self.reg_pc += 2;
        
        value
    }
    
    fn print_state(&mut self) {
        self.current_instruction = self.memory_map.load_byte(self.reg_pc);
        println!("{:?}", self);
    }
    
    fn get_opcode(opcode: u8) -> Opcode {
        Opcode::from_u8(opcode).unwrap_or_else(|| panic!("Unknown opcode: {:X}", opcode))
    }
}

impl fmt::Debug for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04X} {:?} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:",
            self.reg_pc,
            Cpu::get_opcode(self.current_instruction),
            self.reg_a,
            self.reg_x,
            self.reg_y,
            self.reg_p.as_u8(),
            self.reg_sp)
    }
}

#[derive(Debug)]
struct StatusRegister {
    carry: bool,
    zero: bool,
    interrupt_disable: bool,
    decimal_mode: bool,
    break_command: bool,
    bit5: bool,
    overflow: bool,
    negative: bool
}

impl StatusRegister {
    fn as_u8(&self) -> u8 {
        (self.negative as u8)            << 7 |
        (self.overflow as u8)            << 6 |
        (self.bit5 as u8)                << 5 |
        (self.break_command as u8)       << 4 |
        (self.decimal_mode as u8)        << 3 |
        (self.interrupt_disable as u8)   << 2 |
        (self.zero as u8)                << 1 |
        (self.carry as u8)               << 0
    }
}

impl From<u8> for StatusRegister {
    fn from(value: u8) -> StatusRegister {
        StatusRegister {
            carry:              (value & (1 << 0)) != 0,
            zero:               (value & (1 << 1)) != 0,
            interrupt_disable:  (value & (1 << 2)) != 0,
            decimal_mode:       (value & (1 << 3)) != 0,
            break_command:      (value & (1 << 4)) != 0,
            bit5:               (value & (1 << 5)) != 0,
            overflow:           (value & (1 << 6)) != 0,
            negative:           (value & (1 << 7)) != 0
        }
    }
}