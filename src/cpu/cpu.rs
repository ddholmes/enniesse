use super::super::memory::{Memory, MemoryInterface};
use super::super::rom::Rom;
use super::addressing_modes::*;
use super::opcode;

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
    
    pub memory_interface: MemoryInterface,
    
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
            memory_interface: MemoryInterface::new(rom),
            current_instruction: 0
        }
    }
    
    pub fn reset(&mut self) {
        // TODO: accurately model reset
        
        // TODO: uncomment to start in the appropriate place
        self.reg_pc = self.memory_interface.load_word(RESET_VECTOR);
    }
    
    pub fn run_instruction(&mut self) {
        let opcode = self.load_byte_from_pc();
        
        self.execute_instruction(opcode);
    }
    
    pub fn execute_instruction(&mut self, opcode: u8) {
        match opcode {
            0x01 => { let mode = MemoryAddressingMode::indirect_x(self); self.ora(mode); },
            0x03 => { let mode = MemoryAddressingMode::indirect_x(self); self.slo(mode); }, // unofficial
            0x04 => { let mode = ImmediateAddressingMode; self.nop_with_read(mode); }, // unofficial
            0x05 => { let mode = MemoryAddressingMode::zero_page(self); self.ora(mode); },
            0x06 => { let mode = MemoryAddressingMode::zero_page(self); self.asl(mode); },
            0x07 => { let mode = MemoryAddressingMode::zero_page(self); self.slo(mode); }, // unofficial
            0x08 => { self.php(); },
            0x09 => { let mode = ImmediateAddressingMode; self.ora(mode); },
            0x0a => { let mode = AccumulatorAddressingMode; self.asl(mode); },
            0x0c => { let mode = MemoryAddressingMode::absolute(self); self.nop_with_read(mode); }, // unofficial
            0x0d => { let mode = MemoryAddressingMode::absolute(self); self.ora(mode); },
            0x0e => { let mode = MemoryAddressingMode::absolute(self); self.asl(mode); },
            0x0f => { let mode = MemoryAddressingMode::absolute(self); self.slo(mode); }, // unofficial
            0x10 => { self.bpl(); },
            0x11 => { let mode = MemoryAddressingMode::indirect_y(self); self.ora(mode); },
            0x13 => { let mode = MemoryAddressingMode::indirect_y(self); self.slo(mode); }, // unofficial
            0x14 => { let mode = MemoryAddressingMode::indirect_x(self); self.nop_with_read(mode); }, // unofficial
            0x15 => { let mode = MemoryAddressingMode::zero_page_x(self); self.ora(mode); },
            0x16 => { let mode = MemoryAddressingMode::zero_page_x(self); self.asl(mode); },
            0x17 => { let mode = MemoryAddressingMode::zero_page_x(self); self.slo(mode); }, // unofficial
            0x18 => { self.clc(); },
            0x19 => { let mode = MemoryAddressingMode::absolute_y(self); self.ora(mode); },
            0x1a => { self.nop(); } // unofficial
            0x1b => { let mode = MemoryAddressingMode::absolute_y(self); self.slo(mode); }, // unofficial
            0x1c => { let mode = MemoryAddressingMode::absolute_x(self); self.nop_with_read(mode); }, // unofficial
            0x1d => { let mode = MemoryAddressingMode::absolute_x(self); self.ora(mode); },
            0x1e => { let mode = MemoryAddressingMode::absolute_x(self); self.asl(mode); },
            0x1f => { let mode = MemoryAddressingMode::absolute_x(self); self.slo(mode); }, // unofficial
            0x20 => { self.jsr(); },
            0x21 => { let mode = MemoryAddressingMode::indirect_x(self); self.and(mode); },
            0x23 => { let mode = MemoryAddressingMode::indirect_x(self); self.rla(mode); }, // unofficial
            0x24 => { let mode = MemoryAddressingMode::zero_page(self); self.bit(mode); },
            0x25 => { let mode = MemoryAddressingMode::zero_page(self); self.and(mode); },
            0x26 => { let mode = MemoryAddressingMode::zero_page(self); self.rol(mode); },
            0x27 => { let mode = MemoryAddressingMode::zero_page(self); self.rla(mode); }, // unofficial
            0x28 => { self.plp(); },
            0x29 => { let mode = ImmediateAddressingMode; self.and(mode); },
            0x2a => { let mode = AccumulatorAddressingMode; self.rol(mode); },
            0x2c => { let mode = MemoryAddressingMode::absolute(self); self.bit(mode); },
            0x2d => { let mode = MemoryAddressingMode::absolute(self); self.and(mode); },
            0x2e => { let mode = MemoryAddressingMode::absolute(self); self.rol(mode); },
            0x2f => { let mode = MemoryAddressingMode::absolute(self); self.rla(mode); }, // unofficial
            0x30 => { self.bmi(); },
            0x31 => { let mode = MemoryAddressingMode::indirect_y(self); self.and(mode); },
            0x33 => { let mode = MemoryAddressingMode::indirect_y(self); self.rla(mode); }, // unofficial
            0x34 => { let mode = MemoryAddressingMode::indirect_x(self); self.nop_with_read(mode); }, // unofficial
            0x35 => { let mode = MemoryAddressingMode::zero_page_x(self); self.and(mode); },
            0x36 => { let mode = MemoryAddressingMode::zero_page_x(self); self.rol(mode); },
            0x37 => { let mode = MemoryAddressingMode::zero_page_x(self); self.rla(mode); }, // unofficial
            0x38 => { self.sec(); },
            0x39 => { let mode = MemoryAddressingMode::absolute_y(self); self.and(mode); },
            0x3a => { self.nop(); } // unofficial
            0x3b => { let mode = MemoryAddressingMode::absolute_y(self); self.rla(mode); }, // unofficial
            0x3c => { let mode = MemoryAddressingMode::absolute_x(self); self.nop_with_read(mode); }, // unofficial
            0x3d => { let mode = MemoryAddressingMode::absolute_x(self); self.and(mode); },
            0x3e => { let mode = MemoryAddressingMode::absolute_x(self); self.rol(mode); },
            0x3f => { let mode = MemoryAddressingMode::absolute_x(self); self.rla(mode); }, // unofficial
            0x40 => { self.rti(); },
            0x41 => { let mode = MemoryAddressingMode::indirect_x(self); self.eor(mode); },
            0x43 => { let mode = MemoryAddressingMode::indirect_x(self); self.sre(mode); }, // unofficial
            0x44 => { let mode = ImmediateAddressingMode; self.nop_with_read(mode); }, // unofficial
            0x45 => { let mode = MemoryAddressingMode::zero_page(self); self.eor(mode); },
            0x46 => { let mode = MemoryAddressingMode::zero_page(self); self.lsr(mode); },
            0x47 => { let mode = MemoryAddressingMode::zero_page(self); self.sre(mode); }, // unofficial
            0x48 => { self.pha(); },
            0x49 => { let mode = ImmediateAddressingMode; self.eor(mode); },
            0x4a => { let mode = AccumulatorAddressingMode; self.lsr(mode); },
            0x4c => { self.jmp(); },
            0x4d => { let mode = MemoryAddressingMode::absolute(self); self.eor(mode); },
            0x4e => { let mode = MemoryAddressingMode::absolute(self); self.lsr(mode); },
            0x4f => { let mode = MemoryAddressingMode::absolute(self); self.sre(mode); }, // unofficial
            0x50 => { self.bvc(); },
            0x51 => { let mode = MemoryAddressingMode::indirect_y(self); self.eor(mode); },
            0x53 => { let mode = MemoryAddressingMode::indirect_y(self); self.sre(mode); }, // unofficial
            0x54 => { let mode = MemoryAddressingMode::indirect_x(self); self.nop_with_read(mode); }, // unofficial
            0x55 => { let mode = MemoryAddressingMode::zero_page_x(self); self.eor(mode); },
            0x56 => { let mode = MemoryAddressingMode::zero_page_x(self); self.lsr(mode); },
            0x57 => { let mode = MemoryAddressingMode::zero_page_x(self); self.sre(mode); }, // unofficial
            0x59 => { let mode = MemoryAddressingMode::absolute_y(self); self.eor(mode); },
            0x5a => { self.nop(); } // unofficial
            0x5b => { let mode = MemoryAddressingMode::absolute_y(self); self.sre(mode); }, // unofficial
            0x5c => { let mode = MemoryAddressingMode::absolute_x(self); self.nop_with_read(mode); }, // unofficial
            0x5d => { let mode = MemoryAddressingMode::absolute_x(self); self.eor(mode); },
            0x5e => { let mode = MemoryAddressingMode::absolute_x(self); self.lsr(mode); },
            0x5f => { let mode = MemoryAddressingMode::absolute_x(self); self.sre(mode); }, // unofficial
            0x60 => { self.rts(); },
            0x61 => { let mode = MemoryAddressingMode::indirect_x(self); self.adc(mode); },
            0x63 => { let mode = MemoryAddressingMode::indirect_x(self); self.rra(mode); }, // unofficial
            0x64 => { let mode = ImmediateAddressingMode; self.nop_with_read(mode); }, // unofficial
            0x65 => { let mode = MemoryAddressingMode::zero_page(self); self.adc(mode); },
            0x66 => { let mode = MemoryAddressingMode::zero_page(self); self.ror(mode); },
            0x67 => { let mode = MemoryAddressingMode::zero_page(self); self.rra(mode); }, // unofficial
            0x68 => { self.pla(); },
            0x69 => { let mode = ImmediateAddressingMode; self.adc(mode); },
            0x6a => { let mode = AccumulatorAddressingMode; self.ror(mode); },
            0x6c => { self.jmp_indirect(); },
            0x6d => { let mode = MemoryAddressingMode::absolute(self); self.adc(mode); },
            0x6e => { let mode = MemoryAddressingMode::absolute(self); self.ror(mode); },
            0x6f => { let mode = MemoryAddressingMode::absolute(self); self.rra(mode); }, // unofficial
            0x70 => { self.bvs(); },
            0x71 => { let mode = MemoryAddressingMode::indirect_y(self); self.adc(mode); },
            0x73 => { let mode = MemoryAddressingMode::indirect_y(self); self.rra(mode); }, // unofficial
            0x74 => { let mode = MemoryAddressingMode::indirect_x(self); self.nop_with_read(mode); }, // unofficial
            0x75 => { let mode = MemoryAddressingMode::zero_page_x(self); self.adc(mode); },
            0x76 => { let mode = MemoryAddressingMode::zero_page_x(self); self.ror(mode); },
            0x77 => { let mode = MemoryAddressingMode::zero_page_x(self); self.rra(mode); }, // unofficial
            0x78 => { self.sei(); },
            0x79 => { let mode = MemoryAddressingMode::absolute_y(self); self.adc(mode); },
            0x7a => { self.nop(); } // unofficial
            0x7b => { let mode = MemoryAddressingMode::absolute_y(self); self.rra(mode); }, // unofficial
            0x7c => { let mode = MemoryAddressingMode::absolute_x(self); self.nop_with_read(mode); }, // unofficial
            0x7d => { let mode = MemoryAddressingMode::absolute_x(self); self.adc(mode); },
            0x7e => { let mode = MemoryAddressingMode::absolute_x(self); self.ror(mode); },
            0x7f => { let mode = MemoryAddressingMode::absolute_x(self); self.rra(mode); }, // unofficial
            0x80 => { let mode = ImmediateAddressingMode; self.nop_with_read(mode); }, // unofficial
            0x81 => { let mode = MemoryAddressingMode::indirect_x(self); self.sta(mode); },
            0x83 => { let mode = MemoryAddressingMode::indirect_x(self); self.sax(mode); }, // unofficial
            0x84 => { let mode = MemoryAddressingMode::zero_page(self); self.sty(mode); },
            0x85 => { let mode = MemoryAddressingMode::zero_page(self); self.sta(mode); },
            0x86 => { let mode = MemoryAddressingMode::zero_page(self); self.stx(mode); },
            0x87 => { let mode = MemoryAddressingMode::zero_page(self); self.sax(mode); }, // unofficial
            0x88 => { self.dey(); },
            0x8a => { self.txa(); },
            0x8c => { let mode = MemoryAddressingMode::absolute(self); self.sty(mode); },
            0x8d => { let mode = MemoryAddressingMode::absolute(self); self.sta(mode); },
            0x8e => { let mode = MemoryAddressingMode::absolute(self); self.stx(mode); },
            0x8f => { let mode = MemoryAddressingMode::absolute(self); self.sax(mode); }, // unofficial
            0x90 => { self.bcc(); },
            0x91 => { let mode = MemoryAddressingMode::indirect_y(self); self.sta(mode); },
            0x94 => { let mode = MemoryAddressingMode::zero_page_x(self); self.sty(mode); },
            0x95 => { let mode = MemoryAddressingMode::zero_page_x(self); self.sta(mode); },
            0x96 => { let mode = MemoryAddressingMode::zero_page_y(self); self.stx(mode); },
            0x97 => { let mode = MemoryAddressingMode::zero_page_y(self); self.sax(mode); }, // unofficial
            0x98 => { self.tya(); },
            0x99 => { let mode = MemoryAddressingMode::absolute_y(self); self.sta(mode); },
            0x9a => { self.txs(); },
            0x9d => { let mode = MemoryAddressingMode::absolute_x(self); self.sta(mode); },
            0xa0 => { let mode = ImmediateAddressingMode; self.ldy(mode); },
            0xa1 => { let mode = MemoryAddressingMode::indirect_x(self); self.lda(mode); },
            0xa2 => { let mode = ImmediateAddressingMode; self.ldx(mode); },
            0xa3 => { let mode = MemoryAddressingMode::indirect_x(self); self.lax(mode); }, // unofficial
            0xa4 => { let mode = MemoryAddressingMode::zero_page(self); self.ldy(mode); },
            0xa5 => { let mode = MemoryAddressingMode::zero_page(self); self.lda(mode); },
            0xa6 => { let mode = MemoryAddressingMode::zero_page(self); self.ldx(mode); },
            0xa7 => { let mode = MemoryAddressingMode::zero_page(self); self.lax(mode); }, // unofficial
            0xa8 => { self.tay(); },
            0xa9 => { let mode = ImmediateAddressingMode; self.lda(mode); },
            0xaa => { self.tax(); },
            0xac => { let mode = MemoryAddressingMode::absolute(self); self.ldy(mode); },
            0xad => { let mode = MemoryAddressingMode::absolute(self); self.lda(mode); },
            0xae => { let mode = MemoryAddressingMode::absolute(self); self.ldx(mode); },
            0xaf => { let mode = MemoryAddressingMode::absolute(self); self.lax(mode); }, // unofficial
            0xb0 => { self.bcs(); },
            0xb1 => { let mode = MemoryAddressingMode::indirect_y(self); self.lda(mode); },
            0xb3 => { let mode = MemoryAddressingMode::indirect_y(self); self.lax(mode); }, // unofficial
            0xb4 => { let mode = MemoryAddressingMode::zero_page_x(self); self.ldy(mode); },
            0xb5 => { let mode = MemoryAddressingMode::zero_page_x(self); self.lda(mode); },
            0xb6 => { let mode = MemoryAddressingMode::zero_page_y(self); self.ldx(mode); },
            0xb7 => { let mode = MemoryAddressingMode::zero_page_y(self); self.lax(mode); }, // unofficial
            0xb8 => { self.clv(); },
            0xb9 => { let mode = MemoryAddressingMode::absolute_y(self); self.lda(mode); },
            0xba => { self.tsx(); },
            0xbc => { let mode = MemoryAddressingMode::absolute_x(self); self.ldy(mode); },
            0xbd => { let mode = MemoryAddressingMode::absolute_x(self); self.lda(mode); },
            0xbe => { let mode = MemoryAddressingMode::absolute_y(self); self.ldx(mode); },
            0xbf => { let mode = MemoryAddressingMode::absolute_y(self); self.lax(mode); }, // unofficial
            0xc0 => { let mode = ImmediateAddressingMode; self.cpy(mode); },
            0xc1 => { let mode = MemoryAddressingMode::indirect_x(self); self.cmp(mode); },
            0xc3 => { let mode = MemoryAddressingMode::indirect_x(self); self.dcp(mode); }, // unofficial
            0xc4 => { let mode = MemoryAddressingMode::zero_page(self); self.cpy(mode); },
            0xc5 => { let mode = MemoryAddressingMode::zero_page(self); self.cmp(mode); },
            0xc6 => { let mode = MemoryAddressingMode::zero_page(self); self.dec(mode); },
            0xc7 => { let mode = MemoryAddressingMode::zero_page(self); self.dcp(mode); }, // unofficial
            0xc8 => { self.iny(); },
            0xc9 => { let mode = ImmediateAddressingMode; self.cmp(mode); },
            0xca => { self.dex(); },
            0xcc => { let mode = MemoryAddressingMode::absolute(self); self.cpy(mode); },
            0xcd => { let mode = MemoryAddressingMode::absolute(self); self.cmp(mode); },
            0xce => { let mode = MemoryAddressingMode::absolute(self); self.dec(mode); },
            0xcf => { let mode = MemoryAddressingMode::absolute(self); self.dcp(mode); }, // unofficial
            0xd0 => { self.bne(); },
            0xd1 => { let mode = MemoryAddressingMode::indirect_y(self); self.cmp(mode); },
            0xd3 => { let mode = MemoryAddressingMode::indirect_y(self); self.dcp(mode); }, // unofficial
            0xd4 => { let mode = MemoryAddressingMode::indirect_x(self); self.nop_with_read(mode); }, // unofficial
            0xd5 => { let mode = MemoryAddressingMode::zero_page_x(self); self.cmp(mode); },
            0xd6 => { let mode = MemoryAddressingMode::zero_page_x(self); self.dec(mode); },
            0xd7 => { let mode = MemoryAddressingMode::zero_page_x(self); self.dcp(mode); }, // unofficial
            0xd8 => { self.cld(); },
            0xd9 => { let mode = MemoryAddressingMode::absolute_y(self); self.cmp(mode); },
            0xda => { self.nop(); } // unofficial
            0xdb => { let mode = MemoryAddressingMode::absolute_y(self); self.dcp(mode); }, // unofficial
            0xdc => { let mode = MemoryAddressingMode::absolute_x(self); self.nop_with_read(mode); }, // unofficial
            0xdd => { let mode = MemoryAddressingMode::absolute_x(self); self.cmp(mode); },
            0xde => { let mode = MemoryAddressingMode::absolute_x(self); self.dec(mode); },
            0xdf => { let mode = MemoryAddressingMode::absolute_x(self); self.dcp(mode); }, // unofficial
            0xe0 => { let mode = ImmediateAddressingMode; self.cpx(mode); },
            0xe1 => { let mode = MemoryAddressingMode::indirect_x(self); self.sbc(mode); },
            0xe3 => { let mode = MemoryAddressingMode::indirect_x(self); self.isc(mode); }, // unofficial
            0xe4 => { let mode = MemoryAddressingMode::zero_page(self); self.cpx(mode); },
            0xe5 => { let mode = MemoryAddressingMode::zero_page(self); self.sbc(mode); },
            0xe6 => { let mode = MemoryAddressingMode::zero_page(self); self.inc(mode); },
            0xe7 => { let mode = MemoryAddressingMode::zero_page(self); self.isc(mode); }, // unofficial
            0xe8 => { self.inx(); },
            0xe9 => { let mode = ImmediateAddressingMode; self.sbc(mode); },
            0xea => { self.nop(); },
            0xeb => { let mode = ImmediateAddressingMode; self.sbc(mode); }, // unofficial
            0xec => { let mode = MemoryAddressingMode::absolute(self); self.cpx(mode); },
            0xed => { let mode = MemoryAddressingMode::absolute(self); self.sbc(mode); },
            0xee => { let mode = MemoryAddressingMode::absolute(self); self.inc(mode); },
            0xef => { let mode = MemoryAddressingMode::absolute(self); self.isc(mode); }, // unofficial
            0xf0 => { self.beq(); },
            0xf1 => { let mode = MemoryAddressingMode::indirect_y(self); self.sbc(mode); },
            0xf3 => { let mode = MemoryAddressingMode::indirect_y(self); self.isc(mode); }, // unofficial
            0xf4 => { let mode = MemoryAddressingMode::indirect_x(self); self.nop_with_read(mode); }, // unofficial
            0xf5 => { let mode = MemoryAddressingMode::zero_page_x(self); self.sbc(mode); },
            0xf6 => { let mode = MemoryAddressingMode::zero_page_x(self); self.inc(mode); },
            0xf7 => { let mode = MemoryAddressingMode::zero_page_x(self); self.isc(mode); }, // unofficial
            0xf8 => { self.sed(); },
            0xf9 => { let mode = MemoryAddressingMode::absolute_y(self); self.sbc(mode); },
            0xfa => { self.nop(); } // unofficial
            0xfb => { let mode = MemoryAddressingMode::absolute_y(self); self.isc(mode); }, // unofficial
            0xfc => { let mode = MemoryAddressingMode::absolute_x(self); self.nop_with_read(mode); }, // unofficial
            0xfd => { let mode = MemoryAddressingMode::absolute_x(self); self.sbc(mode); },
            0xfe => { let mode = MemoryAddressingMode::absolute_x(self); self.inc(mode); },
            0xff => { let mode = MemoryAddressingMode::absolute_x(self); self.isc(mode); }, // unofficial
            _ => panic!("Unknown opcode: {:02X}", opcode)
        }
    }
    
    pub fn load_byte_from_pc(&mut self) -> u8 {
        let value = self.memory_interface.load_byte(self.reg_pc);
        self.reg_pc += 1;
        
        value
    }
    
    pub fn load_word_from_pc(&mut self) -> u16 {
        let value = self.memory_interface.load_word(self.reg_pc);
        self.reg_pc += 2;
        
        value
    }
    
    fn stack_push_byte(&mut self, val: u8) {
        self.memory_interface.store_byte(STACK_START + self.reg_sp as u16, val);
        self.reg_sp -= 1;
    }
    
    fn stack_push_word(&mut self, val: u16) {
        self.memory_interface.store_word(STACK_START + (self.reg_sp - 1) as u16, val);
        self.reg_sp -= 2;
    }
    
    fn stack_pop_byte(&mut self) -> u8 {
        let val = self.memory_interface.load_byte(STACK_START + (self.reg_sp + 1) as u16);
        self.reg_sp += 1;
        val
    }
    
    fn stack_pop_word(&mut self) -> u16 {
        let val = self.memory_interface.load_word(STACK_START + (self.reg_sp + 1) as u16);
        self.reg_sp += 2;
        val
    }
    
    fn branch(&mut self, condition: bool) {
        let displacement = self.load_byte_from_pc() as i8;
        
        if condition {
            self.reg_pc = (self.reg_pc as i16 + displacement as i16) as u16;
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
    
    fn compare(&mut self, reg: u8, val: u8) {
        let result = reg as i16 - val as i16;
        
        self.reg_p.carry = reg >= val;
        self.set_zero_negative_flags(result as u8);    
    }
    
    fn shift_left(&mut self, set_lsb: bool, val: u8) -> u8 {
        self.reg_p.carry = (val & 0b1000_0000) != 0;
        let mut result = val << 1;
        if set_lsb {
            result = result | 1;
        }
        
        self.set_zero_negative_flags(result);
        result
    }
    
    fn shift_right(&mut self, set_msb: bool, val: u8) -> u8 {
        self.reg_p.carry = (val & 1) != 0;
        let mut result = val >> 1;
        if set_msb {
            result = result | 0b1000_0000;
        }
        
        self.set_zero_negative_flags(result);
        result
    }
    
    pub fn trace_state(&mut self) {
        self.current_instruction = self.memory_interface.load_byte(self.reg_pc);
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
        let lsb: u8 = self.memory_interface.load_byte(addr);
        let msb: u8 = self.memory_interface.load_byte((addr & 0xff00) | ((addr + 1) & 0x00ff));
        
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
        let val = mode.load(self) as u8;
        
        self.compare(reg, val);
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
        let val = mode.load(self);
        
        self.compare(reg, val);
    }
    fn cpx<T:AddressingMode>(&mut self, mode: T) {
        let reg = self.reg_x;
        let val = mode.load(self);
        
        self.compare(reg, val);
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
        let val = mode.load(self);
        let result = self.shift_right(false, val);
        mode.store(self, result);
    }
    fn asl<T:AddressingMode>(&mut self, mode: T) {
        let val = mode.load(self);
        let result = self.shift_left(false, val);
        mode.store(self, result);
    }
    fn ror<T:AddressingMode>(&mut self, mode: T) {
        let carry = self.reg_p.carry;
        let val = mode.load(self);
        let result = self.shift_right(carry, val);
        mode.store(self, result);
    }
    fn rol<T:AddressingMode>(&mut self, mode: T) {
        let carry = self.reg_p.carry;
        let val = mode.load(self);
        let result = self.shift_left(carry, val);
        mode.store(self, result);
    }
    fn sty<T:AddressingMode>(&mut self, mode: T) {
        let reg = self.reg_y;
        mode.store(self, reg);
    }
    fn inc<T:AddressingMode>(&mut self, mode: T) {
        let val = mode.load(self).wrapping_add(1);
        
        self.set_zero_negative_flags(val);
        mode.store(self, val);
    }
    fn dec<T:AddressingMode>(&mut self, mode: T) {
        let val = mode.load(self).wrapping_sub(1);
        self.set_zero_negative_flags(val);
        
        mode.store(self, val);
    }
    
    // unofficial opcodes
    
    // loads from memory but does nothing with it
    fn nop_with_read<T:AddressingMode>(&mut self, mode: T) {
        mode.load(self);
    }
    
    // lda then tax
    fn lax<T:AddressingMode>(&mut self, mode: T) {
        self.lda(mode);
        self.tax();
    }
    
    // store the bitwise and of A and x
    fn sax<T:AddressingMode>(&mut self, mode: T) {
        let val = self.reg_a & self.reg_x;
        mode.store(self, val);
    }
    
    // dec then cmp
    fn dcp<T:AddressingMode>(&mut self, mode: T) {        
        // not calling dec directly to avoid moving/borrowing the mode var
        let val = mode.load(self).wrapping_sub(1);
        self.set_zero_negative_flags(val);
        mode.store(self, val);
        
        self.cmp(mode);
    }
    
    // inc then sbc
    fn isc<T:AddressingMode>(&mut self, mode: T) {
        // not calling inc directly to avoid moving/borrowing the mode var
        let val = mode.load(self).wrapping_add(1);
        
        self.set_zero_negative_flags(val);
        mode.store(self, val);
        
        self.sbc(mode);
    }
    
    // asl then ora
    fn slo<T:AddressingMode>(&mut self, mode: T) {
        let val = mode.load(self);
        let result = self.shift_left(false, val);
        mode.store(self, result);
        
        self.ora(mode);
    }
    
    // rol then and
    fn rla<T:AddressingMode>(&mut self, mode: T) {
        let val = mode.load(self);
        let carry = self.reg_p.carry;
        let result = self.shift_left(carry, val);
        mode.store(self, result);
        
        self.and(mode);
    }
    
    // lsr then eor
    fn sre<T:AddressingMode>(&mut self, mode: T) {
        let val = mode.load(self);
        let result = self.shift_right(false, val);
        mode.store(self, result);
        
        self.eor(mode);
    }
    
    // ror then adc
    fn rra<T:AddressingMode>(&mut self, mode: T) {
        let carry = self.reg_p.carry;
        let val = mode.load(self);
        let result = self.shift_right(carry, val);
        mode.store(self, result);
        
        self.adc(mode);
    }
}

impl fmt::Debug for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04X} {:20} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:",
            self.reg_pc,
            opcode::decode(self.current_instruction),
            self.reg_a,
            self.reg_x,
            self.reg_y,
            self.reg_p.as_u8(),
            self.reg_sp)
    }
}

#[derive(Debug)]
pub struct StatusRegister {
    pub carry: bool,
    pub zero: bool,
    pub interrupt_disable: bool,
    pub decimal_mode: bool,
    pub break_command: bool,
    pub bit5: bool,
    pub overflow: bool,
    pub negative: bool
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