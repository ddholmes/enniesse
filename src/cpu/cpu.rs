use super::super::memory::MemoryMap;
use super::super::rom::Rom;
use super::opcode::Opcode;
use super::instruction::Instruction;
use super::instruction::AddressingMode;

use std::fmt;
use num::FromPrimitive;

const NMI_VECTOR: u16 = 0xfffa;
const RESET_VECTOR: u16 = 0xfffc;
const BRK_VECTOR: u16 = 0xfffe;

const STACK_START: u16 = 0x0100;

pub struct Cpu {
    // accumulator
    pub reg_a: u8,
    // index X
    pub reg_x: u8,
    // index Y
    pub reg_y: u8,
    // program counter
    pub reg_pc: u16,
    // stack pointer
    pub reg_sp: u8,
    // status register
    pub reg_p: StatusRegister,
    
    pub memory_map: MemoryMap,
    
    current_instruction: u8,
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
        let instruction = self.load_instruction();
        
        self.execute_instruction(instruction);
    }
    
    fn execute_instruction(&mut self, instruction: Instruction) {
        match instruction.opcode {
            Opcode::Jmp => {
                self.reg_pc = self.load_word_from_pc();
            },
            Opcode::Ldx => {
                let val = instruction.addressing_mode.unwrap().load(self);
                self.set_zero_negative_flags(val);
                self.reg_x = val;
            },
            Opcode::Stx => {
                let reg = self.reg_x;
                
                instruction.addressing_mode.unwrap().store(self, reg);
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
                let val = instruction.addressing_mode.unwrap().load(self);
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
                
                instruction.addressing_mode.unwrap().store(self, reg);
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
                let reg = self.reg_a;
                self.compare(reg);
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
            },
            Opcode::Bmi => {
                let condition = self.reg_p.negative;
                self.branch(condition);
            },
            Opcode::Ora => {
                let val = self.load_byte_from_pc();
                
                let result = val | self.reg_a;
                self.set_zero_negative_flags(result);
                self.reg_a = result;
            },
            Opcode::Clv => {
                self.reg_p.overflow = false;
            },
            Opcode::Eor => {
                let val = self.load_byte_from_pc();
                
                let result = val ^ self.reg_a;
                self.set_zero_negative_flags(result);
                self.reg_a = result;
            },
            Opcode::Adc => {
                let val = self.load_byte_from_pc();
                
                let mut result = self.reg_a as u32 + val as u32;
                if self.reg_p.carry {
                    result += 1;
                }
                
                self.reg_p.carry = (result & 0x0100) != 0;
                let result = result as u8;
                self.reg_p.overflow = (self.reg_a ^ val) & 0b1000_0000 == 0 && (self.reg_a ^ result) & 0b1000_0000 == 0b1000_0000;
                self.set_zero_negative_flags(result);
                self.reg_a = result;
            },
            Opcode::Ldy => {
                let val = self.load_byte_from_pc();
                self.set_zero_negative_flags(val);
                self.reg_y = val;
            },
            Opcode::Cpy => {
                let reg = self.reg_y;
                self.compare(reg);
            },
            Opcode::Cpx => {
                let reg = self.reg_x;
                self.compare(reg);
            },
            Opcode::Sbc => {
                let val = self.load_byte_from_pc();
                
                let mut result = self.reg_a as i16 - val as i16;
                if !self.reg_p.carry {
                    result -= 1;
                }
                
                self.reg_p.carry = (result & 0x0100) == 0;
                let result = result as u8;
                self.reg_p.overflow = (self.reg_a ^ result) & 0b1000_0000 != 0 && (self.reg_a ^ val) & 0b1000_0000 == 0b1000_0000;
                self.set_zero_negative_flags(result);
                self.reg_a = result;
            },
            Opcode::Iny => {
                let result = self.reg_y.wrapping_add(1);
                self.set_zero_negative_flags(result);
                self.reg_y = result;
            },
            Opcode::Inx => {
                let result = self.reg_x.wrapping_add(1);
                self.set_zero_negative_flags(result);
                self.reg_x = result;
            },
            Opcode::Dey => {
                let result = self.reg_y.wrapping_sub(1);
                self.set_zero_negative_flags(result);
                self.reg_y = result;
            }
            Opcode::Dex => {
                let result = self.reg_x.wrapping_sub(1);
                self.set_zero_negative_flags(result);
                self.reg_x = result;
            },
            Opcode::Tay => {
                let result = self.reg_a;
                self.set_zero_negative_flags(result);
                self.reg_y = result;
            },
            Opcode::Tax => {
                let result = self.reg_a;
                self.set_zero_negative_flags(result);
                self.reg_x = result;
            },
            Opcode::Tya => {
                let result = self.reg_y;
                self.set_zero_negative_flags(result);
                self.reg_a = result;
            },
            Opcode::Txa => {
                let result = self.reg_x;
                self.set_zero_negative_flags(result);
                self.reg_a = result;
            },
            Opcode::Tsx => {
                let result = self.reg_sp;
                self.set_zero_negative_flags(result);
                self.reg_x = result;
            },
            Opcode::Txs => {
                let result = self.reg_x;
                self.reg_sp = result;
            },
            Opcode::Rti => {
                let flags = self.stack_pop_byte();
                self.set_flags(flags);
                self.reg_pc = self.stack_pop_word();
            },
            Opcode::Lsr => {
                let mode = instruction.addressing_mode.unwrap();
                let val = mode.load(self);
                
                self.reg_p.carry = (val & 1) != 0;
                let result = val >> 1;
                self.set_zero_negative_flags(result);
                mode.store(self, result);
            }
        }
    }
    
    pub fn load_byte_from_pc(&mut self) -> u8 {
        let value = self.memory_map.load_byte(self.reg_pc);
        self.reg_pc += 1;
        
        value
    }
    
    pub fn load_word_from_pc(&mut self) -> u16 {
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
    
    fn compare(&mut self, reg: u8) {
        let val = self.load_byte_from_pc();
        
        let result = reg as i16 - val as i16;
        
        self.reg_p.carry = reg >= val;
        self.set_zero_negative_flags(result as u8);    
    }
    
    pub fn trace_state(&mut self) {
        self.current_instruction = self.memory_map.load_byte(self.reg_pc);
        println!("{:?}", self);
    }
    
    fn load_instruction(&mut self) -> Instruction {
        let opcode = self.load_byte_from_pc();
        
        Instruction::decode(opcode)
    }
}

impl fmt::Debug for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04X} {:02X} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:",
            self.reg_pc,
            self.current_instruction,
            self.reg_a,
            self.reg_x,
            self.reg_y,
            self.reg_p.as_u8(),
            self.reg_sp)
    }
}

#[derive(Debug)]
pub struct StatusRegister {
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
    pub fn as_u8(&self) -> u8 {
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