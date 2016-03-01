use super::super::memory::MemoryMap;
use super::super::rom::Rom;
use super::opcode::Opcode;

use std::fmt;
use num::FromPrimitive;

const NMI_VECTOR: u16 = 0xfffa;
const RESET_VECTOR: u16 = 0xfffc;
const BRK_VECTOR: u16 = 0xfffe;

const STACK_START: u16 = 0x0100;

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
                self.set_zero_negative_flags(val);
                self.reg_x = val;
            },
            Opcode::Stx => {
                let reg = self.reg_x;
                self.store_reg(reg);
            },
            Opcode::Jsr => {
                let addr = self.load_word_from_pc();
                let pc = self.reg_pc;
                self.stack_push_word(pc - 1);
                self.reg_pc = addr;
            },
            Opcode::Nop => {},
            Opcode::Sec => {
                self.reg_p.carry = true;
            },
            Opcode::Bcs => {
                let condition = self.reg_p.carry;
                self.branch(condition);
            },
            Opcode::Clc => {
                self.reg_p.carry = false;
            },
            Opcode::Bcc => {
                let condition = !self.reg_p.carry;
                self.branch(condition);
            },
            Opcode::Lda => {
                let val = self.load_byte_from_pc();
                self.set_zero_negative_flags(val);
                self.reg_a = val;
            },
            Opcode::Beq => {
                let condition = self.reg_p.zero;
                self.branch(condition);
            },
            Opcode::Bne => {
                let condition = !self.reg_p.zero;
                self.branch(condition);
            },
            Opcode::Sta => {
                let reg = self.reg_a;
                self.store_reg(reg);
            },
            Opcode::Bit => {
                let addr = self.load_byte_from_pc() as u16;
                let val = self.memory_map.load_byte(addr);
                let a = self.reg_a;
                
                self.reg_p.zero = (val & a) == 0;
                self.reg_p.negative = (val & 0b1000_0000) != 0;
                self.reg_p.overflow = (val & 0b0100_0000) != 0;
            },
            Opcode::Bvs => {
                let condition = self.reg_p.overflow;
                self.branch(condition);
            },
            Opcode::Bvc => {
                let condition = !self.reg_p.overflow;
                self.branch(condition);
            },
            Opcode::Bpl => {
                let condition = !self.reg_p.negative;
                self.branch(condition);
            },
            Opcode::Rts => {
                let addr = self.stack_pop_word();
                
                self.reg_pc = addr + 1;
            },
            Opcode::Sei => {
                self.reg_p.interrupt_disable = true;
            },
            Opcode::Sed => {
                self.reg_p.decimal_mode = true;
            },
            Opcode::Php => {
                let status = self.reg_p.as_u8();
                // sets the break flag
                self.stack_push_byte(status | (1 << 4));
            },
            Opcode::Pla => {
                let val = self.stack_pop_byte();
                self.set_zero_negative_flags(val);
                self.reg_a = val;
            },
            Opcode::And => {
                let val = self.load_byte_from_pc();
                
                let result = val & self.reg_a;
                self.set_zero_negative_flags(result);
                self.reg_a = result;
            },
            Opcode::Cmp => {
                let val = self.load_byte_from_pc();
                
                let result = self.reg_a as u32 - val as u32;
                
                self.reg_p.carry = self.reg_a >= val;
                self.set_zero_negative_flags(result as u8);
            },
            Opcode::Cld => {
                self.reg_p.decimal_mode = false;
            },
            Opcode::Pha => {
                let a = self.reg_a;
                self.stack_push_byte(a);
            },
            Opcode::Plp => {
                let val = self.stack_pop_byte();
                self.set_flags(val);
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
    
    fn stack_push_byte(&mut self, val: u8) {
        self.memory_map.store_byte(STACK_START + self.reg_sp as u16, val);
        self.reg_sp -= 1;
    }
    
    fn stack_push_word(&mut self, val: u16) {
        self.memory_map.store_word(STACK_START + (self.reg_sp - 1) as u16, val);
        self.reg_sp -= 2;
    }
    
    fn stack_pop_byte(&mut self) -> u8 {
        let val = self.memory_map.load_byte(STACK_START + (self.reg_sp + 1) as u16);
        self.reg_sp += 1;
        val
    }
    
    fn stack_pop_word(&mut self) -> u16 {
        let val = self.memory_map.load_word(STACK_START + (self.reg_sp + 1) as u16);
        self.reg_sp += 2;
        val
    }
    
    fn branch(&mut self, condition: bool) {
        let displacement = self.load_byte_from_pc();
        
        if condition {
            self.reg_pc += displacement as u16;
        }
    }
    
    fn set_zero_negative_flags(&mut self, val: u8) {
        self.reg_p.negative = (val & 0b1000_0000) != 0;
        self.reg_p.zero = val == 0;
    }
    
    fn store_reg(&mut self, reg: u8) {
        let target = self.load_byte_from_pc();
        self.memory_map.store_byte(target as u16, reg);
    }
    
    fn set_flags(&mut self, flags: u8) {
        let new_flags = StatusRegister::from(flags);
        
        self.reg_p.carry = new_flags.carry;
        self.reg_p.zero = new_flags.zero;
        self.reg_p.interrupt_disable = new_flags.interrupt_disable;
        self.reg_p.decimal_mode = new_flags.decimal_mode;
        // ignores bit 4 and 5
        self.reg_p.overflow = new_flags.overflow;
        self.reg_p.negative = new_flags.negative;
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