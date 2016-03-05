use super::super::memory::MemoryMap;
use super::super::rom::Rom;
use super::opcode::Opcode;
use super::addressing_modes::*;

use std::fmt;

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
        let opcode = self.load_byte_from_pc();
        
        self.execute_instruction(opcode);
    }
    
    pub fn execute_instruction(&mut self, opcode: u8) {
        match opcode {
            0x01 => { let mode = MemoryAddressingMode::indirect_x(self); self.ora(mode); },
            0x05 => { let mode = MemoryAddressingMode::zero_page(self); self.ora(mode); },
            0x08 => { self.php(); },
            0x09 => { let mode = ImmediateAddressingMode; self.ora(mode); },
            0x0a => { let mode = AccumulatorAddressingMode; self.asl(mode); },
            0x10 => { self.bpl(); },
            0x18 => { self.clc(); },
            0x20 => { self.jsr(); },
            0x21 => { let mode = MemoryAddressingMode::indirect_x(self); self.and(mode); },
            0x24 => { let mode = MemoryAddressingMode::zero_page(self); self.bit(mode); },
            0x25 => { let mode = MemoryAddressingMode::zero_page(self); self.and(mode); },
            0x28 => { self.plp(); },
            0x29 => { let mode = ImmediateAddressingMode; self.and(mode); },
            0x2a => { let mode = AccumulatorAddressingMode; self.rol(mode); },
            0x30 => { self.bmi(); },
            0x38 => { self.sec(); },
            0x40 => { self.rti(); },
            0x41 => { let mode = MemoryAddressingMode::indirect_x(self); self.eor(mode); },
            0x45 => { let mode = MemoryAddressingMode::zero_page(self); self.eor(mode); },
            0x46 => { let mode = MemoryAddressingMode::zero_page(self); self.lsr(mode); },
            0x48 => { self.pha(); },
            0x49 => { let mode = ImmediateAddressingMode; self.eor(mode); },
            0x4a => { let mode = AccumulatorAddressingMode; self.lsr(mode); },
            0x4c => { self.jmp(); },
            0x50 => { self.bvc(); },
            0x60 => { self.rts(); },
            0x61 => { let mode = MemoryAddressingMode::indirect_x(self); self.adc(mode); },
            0x65 => { let mode = MemoryAddressingMode::zero_page(self); self.adc(mode); },
            0x68 => { self.pla(); },
            0x69 => { let mode = ImmediateAddressingMode; self.adc(mode); },
            0x6a => { let mode = AccumulatorAddressingMode; self.ror(mode); },
            0x70 => { self.bvs(); },
            0x78 => { self.sei(); },
            0x81 => { let mode = MemoryAddressingMode::indirect_x(self); self.sta(mode); },
            0x84 => { let mode = MemoryAddressingMode::zero_page(self); self.sty(mode); },
            0x85 => { let mode = MemoryAddressingMode::zero_page(self); self.sta(mode); },
            0x86 => { let mode = MemoryAddressingMode::zero_page(self); self.stx(mode); },
            0x88 => { self.dey(); },
            0x8a => { self.txa(); },
            0x8d => { let mode = MemoryAddressingMode::absolute(self); self.sta(mode); },
            0x8e => { let mode = MemoryAddressingMode::absolute(self); self.stx(mode); },
            0x90 => { self.bcc(); },
            0x98 => { self.tya(); },
            0x9a => { self.txs(); },
            0xa0 => { let mode = ImmediateAddressingMode; self.ldy(mode); },
            0xa1 => { let mode = MemoryAddressingMode::indirect_x(self); self.lda(mode); },
            0xa2 => { let mode = ImmediateAddressingMode; self.ldx(mode); },
            0xa4 => { let mode = MemoryAddressingMode::zero_page(self); self.ldy(mode); },
            0xa5 => { let mode = MemoryAddressingMode::zero_page(self); self.lda(mode); },
            0xa6 => { let mode = MemoryAddressingMode::zero_page(self); self.ldx(mode); },
            0xa8 => { self.tay(); },
            0xa9 => { let mode = ImmediateAddressingMode; self.lda(mode); },
            0xad => { let mode = MemoryAddressingMode::absolute(self); self.lda(mode); },
            0xae => { let mode = MemoryAddressingMode::absolute(self); self.ldx(mode); },
            0xaa => { self.tax(); },
            0xb0 => { self.bcs(); },
            0xb8 => { self.clv(); },
            0xba => { self.tsx(); },
            0xc0 => { let mode = ImmediateAddressingMode; self.cpy(mode); },
            0xc1 => { let mode = MemoryAddressingMode::indirect_x(self); self.cmp(mode); },
            0xc4 => { let mode = MemoryAddressingMode::zero_page(self); self.cpy(mode); },
            0xc5 => { let mode = MemoryAddressingMode::zero_page(self); self.cmp(mode); },
            0xc8 => { self.iny(); },
            0xc9 => { let mode = ImmediateAddressingMode; self.cmp(mode); },
            0xca => { self.dex(); },
            0xd0 => { self.bne(); },
            0xd8 => { self.cld(); },
            0xe0 => { let mode = ImmediateAddressingMode; self.cpx(mode); },
            0xe1 => { let mode = MemoryAddressingMode::indirect_x(self); self.sbc(mode); },
            0xe4 => { let mode = MemoryAddressingMode::zero_page(self); self.cpx(mode); },
            0xe5 => { let mode = MemoryAddressingMode::zero_page(self); self.sbc(mode); },
            0xe8 => { self.inx(); },
            0xe9 => { let mode = ImmediateAddressingMode; self.sbc(mode); },
            0xea => { self.nop(); },
            0xf0 => { self.beq(); },
            0xf8 => { self.sed(); },
            _ => panic!("Unknown opcode: {:02X}", opcode)
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
    
    fn compare<T:AddressingMode>(&mut self, reg: u8, mode: T) {
        let val = mode.load(self);
        
        let result = reg as i16 - val as i16;
        
        self.reg_p.carry = reg >= val;
        self.set_zero_negative_flags(result as u8);    
    }
    
    fn shift_left<T:AddressingMode>(&mut self, set_lsb: bool, mode: T) {
        let val = mode.load(self);
        self.reg_p.carry = (val & 0b1000_0000) != 0;
        let mut result = val << 1;
        if set_lsb {
            result = result | 1;
        }
        
        self.set_zero_negative_flags(result);
        mode.store(self, result);
    }
    
    fn shift_right<T:AddressingMode>(&mut self, set_msb: bool, mode: T) {
        let val = mode.load(self);
        self.reg_p.carry = (val & 1) != 0;
        let mut result = val >> 1;
        if set_msb {
            result = result | 0b1000_0000;
        }
        
        self.set_zero_negative_flags(result);
        mode.store(self, result);
    }
    
    pub fn trace_state(&mut self) {
        self.current_instruction = self.memory_map.load_byte(self.reg_pc);
        println!("{:?}", self);
    }
    
    
    // instructions
    fn jmp(&mut self) {
        self.reg_pc = self.load_word_from_pc();
    }
    // special case to handle the cpu bug rather than being an addressing mode
    fn jmp_indirect(&mut self) {
        let addr = self.load_word_from_pc();
        
        // cpu bug: if the jmp vector goes to xxff, the msb is pulled from xx00
        let lsb: u8 = self.memory_map.load_byte(addr);
        let msb: u8 = self.memory_map.load_byte((addr & 0xff00) | ((addr + 1) & 0x00ff));
        
        self.reg_pc = (msb as u16) << 8 | lsb as u16;
    }
    fn ldx<T:AddressingMode>(&mut self, mode: T) {
        let val = mode.load(self);
        self.set_zero_negative_flags(val);
        self.reg_x = val;
    }
    fn stx<T:AddressingMode>(&mut self, mode: T) {
        let reg = self.reg_x;
        
        mode.store(self, reg);
    }
    fn jsr(&mut self) {
        let addr = self.load_word_from_pc();
        let pc = self.reg_pc;
        self.stack_push_word(pc - 1);
        self.reg_pc = addr;
    }
    fn nop(&mut self) {}
    fn sec(&mut self) {
        self.reg_p.carry = true;
    }
    fn bcs(&mut self) {
        let condition = self.reg_p.carry;
        self.branch(condition);
    }
    fn clc(&mut self) {
        self.reg_p.carry = false;
    }
    fn bcc(&mut self) {
        let condition = !self.reg_p.carry;
        self.branch(condition);
    }
    fn lda<T:AddressingMode>(&mut self, mode: T) {
        let val = mode.load(self);
        self.set_zero_negative_flags(val);
        self.reg_a = val;
    }
    fn beq(&mut self) {
        let condition = self.reg_p.zero;
        self.branch(condition);
    }
    fn bne(&mut self) {
        let condition = !self.reg_p.zero;
        self.branch(condition);
    }
    fn sta<T:AddressingMode>(&mut self, mode: T) {
        let reg = self.reg_a;
        
        mode.store(self, reg);
    }
    fn bit<T:AddressingMode>(&mut self, mode: T) {
        let val = mode.load(self);
        let a = self.reg_a;
        
        self.reg_p.zero = (val & a) == 0;
        self.reg_p.negative = (val & 0b1000_0000) != 0;
        self.reg_p.overflow = (val & 0b0100_0000) != 0;
    }
    fn bvs(&mut self) {
        let condition = self.reg_p.overflow;
        self.branch(condition);
    }
    fn bvc(&mut self) {
        let condition = !self.reg_p.overflow;
        self.branch(condition);
    }
    fn bpl(&mut self) {
        let condition = !self.reg_p.negative;
        self.branch(condition);
    }
    fn rts(&mut self) {
        let addr = self.stack_pop_word();
        
        self.reg_pc = addr + 1;
    }
    fn sei(&mut self) {
        self.reg_p.interrupt_disable = true;
    }
    fn sed(&mut self) {
        self.reg_p.decimal_mode = true;
    }
    fn php(&mut self) {
        let status = self.reg_p.as_u8();
        // sets the break flag
        self.stack_push_byte(status | (1 << 4));
    }
    fn pla(&mut self) {
        let val = self.stack_pop_byte();
        self.set_zero_negative_flags(val);
        self.reg_a = val;
    }
    fn and<T:AddressingMode>(&mut self, mode: T) {
        let val = mode.load(self);
        
        let result = val & self.reg_a;
        self.set_zero_negative_flags(result);
        self.reg_a = result;
    }
    fn cmp<T:AddressingMode>(&mut self, mode: T) {
        let reg = self.reg_a;
        self.compare(reg, mode);
    }
    fn cld(&mut self) {
        self.reg_p.decimal_mode = false;
    }
    fn pha(&mut self) {
        let a = self.reg_a;
        self.stack_push_byte(a);
    }
    fn plp(&mut self) {
        let val = self.stack_pop_byte();
        self.set_flags(val);
    }
    fn bmi(&mut self) {
        let condition = self.reg_p.negative;
        self.branch(condition);
    }
    fn ora<T:AddressingMode>(&mut self, mode: T) {
        let val = mode.load(self);
        
        let result = val | self.reg_a;
        self.set_zero_negative_flags(result);
        self.reg_a = result;
    }
    fn clv(&mut self) {
        self.reg_p.overflow = false;
    }
    fn eor<T:AddressingMode>(&mut self, mode: T) {
        let val = mode.load(self);
        
        let result = val ^ self.reg_a;
        self.set_zero_negative_flags(result);
        self.reg_a = result;
    }
    fn adc<T:AddressingMode>(&mut self, mode: T) {
        let val = mode.load(self);
        
        let mut result = self.reg_a as u32 + val as u32;
        if self.reg_p.carry {
            result += 1;
        }
        
        self.reg_p.carry = (result & 0x0100) != 0;
        let result = result as u8;
        self.reg_p.overflow = (self.reg_a ^ val) & 0b1000_0000 == 0 && (self.reg_a ^ result) & 0b1000_0000 == 0b1000_0000;
        self.set_zero_negative_flags(result);
        self.reg_a = result;
    }
    fn ldy<T:AddressingMode>(&mut self, mode: T) {
        let val = mode.load(self);
        self.set_zero_negative_flags(val);
        self.reg_y = val;
    }
    fn cpy<T:AddressingMode>(&mut self, mode: T) {
        let reg = self.reg_y;
        self.compare(reg, mode);
    }
    fn cpx<T:AddressingMode>(&mut self, mode: T) {
        let reg = self.reg_x;
        self.compare(reg, mode);
    }
    fn sbc<T:AddressingMode>(&mut self, mode: T) {
        let val = mode.load(self);
        
        let mut result = self.reg_a as i16 - val as i16;
        if !self.reg_p.carry {
            result -= 1;
        }
        
        self.reg_p.carry = (result & 0x0100) == 0;
        let result = result as u8;
        self.reg_p.overflow = (self.reg_a ^ result) & 0b1000_0000 != 0 && (self.reg_a ^ val) & 0b1000_0000 == 0b1000_0000;
        self.set_zero_negative_flags(result);
        self.reg_a = result;
    }
    fn iny(&mut self) {
        let result = self.reg_y.wrapping_add(1);
        self.set_zero_negative_flags(result);
        self.reg_y = result;
    }
    fn inx(&mut self) {
        let result = self.reg_x.wrapping_add(1);
        self.set_zero_negative_flags(result);
        self.reg_x = result;
    }
    fn dey(&mut self) {
        let result = self.reg_y.wrapping_sub(1);
        self.set_zero_negative_flags(result);
        self.reg_y = result;
    }
    fn dex(&mut self) {
        let result = self.reg_x.wrapping_sub(1);
        self.set_zero_negative_flags(result);
        self.reg_x = result;
    }
    fn tay(&mut self) {
        let result = self.reg_a;
        self.set_zero_negative_flags(result);
        self.reg_y = result;
    }
    fn tax(&mut self) {
        let result = self.reg_a;
        self.set_zero_negative_flags(result);
        self.reg_x = result;
    }
    fn tya(&mut self) {
        let result = self.reg_y;
        self.set_zero_negative_flags(result);
        self.reg_a = result;
    }
    fn txa(&mut self) {
        let result = self.reg_x;
        self.set_zero_negative_flags(result);
        self.reg_a = result;
    }
    fn tsx(&mut self) {
        let result = self.reg_sp;
        self.set_zero_negative_flags(result);
        self.reg_x = result;
    }
    fn txs(&mut self) {
        let result = self.reg_x;
        self.reg_sp = result;
    }
    fn rti(&mut self) {
        let flags = self.stack_pop_byte();
        self.set_flags(flags);
        self.reg_pc = self.stack_pop_word();
    }
    fn lsr<T:AddressingMode>(&mut self, mode: T) {
        self.shift_right(false, mode);
    }
    fn asl<T:AddressingMode>(&mut self, mode: T) {
        self.shift_left(false, mode);
    }
    fn ror<T:AddressingMode>(&mut self, mode: T) {
        let carry = self.reg_p.carry;
        self.shift_right(carry, mode);
    }
    fn rol<T:AddressingMode>(&mut self, mode: T) {
        let carry = self.reg_p.carry;
        self.shift_left(carry, mode);
    }
    fn sty<T:AddressingMode>(&mut self, mode: T) {
        let reg = self.reg_y;
        mode.store(self, reg);
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