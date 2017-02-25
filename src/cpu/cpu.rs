use super::super::memory;
use super::super::memory::{Memory, MemoryInterface};
use super::super::rom::Rom;
use super::addressing_mode;
use super::addressing_mode::AddressingMode;
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
    
    pub cycle: usize,
    
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
            cycle: 0,
            memory_interface: MemoryInterface::new(rom),
            current_instruction: 0
        }
    }
    
    pub fn reset(&mut self) {
        // TODO: accurately model reset
        
        self.reg_pc = self.load_word(RESET_VECTOR);
    }
    
    pub fn nmi(&mut self) {
        let pc = self.reg_pc;
        let flags = self.reg_p.as_u8();
        
        self.stack_push_word(pc);
        self.stack_push_byte(flags);
        
        self.reg_pc = self.load_word(NMI_VECTOR);
    }
    
    pub fn irq(&mut self) {
        if self.reg_p.interrupt_disable {
            return;
        }
        
        let pc = self.reg_pc;
        let flags = self.reg_p.as_u8();
        
        self.stack_push_word(pc);
        self.stack_push_byte(flags);
        
        self.reg_pc = self.load_word(BRK_VECTOR);
    }
    
    pub fn step(&mut self) {
        let opcode = self.load_byte_from_pc();
        
        macro_rules! instruction {
            ($i:ident, $c:expr) => {{
                self.$i();
                self.cycle += $c;
            }};
            ($i:ident, $am:path, $c: expr) => {{
                let mode = $am(self);
                self.$i(mode);
                self.cycle += $c;
            }};
        }
        // instruction macro format: (instruction, addressingmode [optional])
        match opcode {
            0x00 => { instruction!(brk, 7); }
            0x01 => { instruction!(ora, addressing_mode::indirect_x, 6); },
            0x03 => { instruction!(slo, addressing_mode::indirect_x, 8); }, // unofficial
            0x04 => { instruction!(nop_with_read, addressing_mode::immediate, 3); }, // unofficial
            0x05 => { instruction!(ora, addressing_mode::zero_page, 3); },
            0x06 => { instruction!(asl, addressing_mode::zero_page, 5); },
            0x07 => { instruction!(slo, addressing_mode::zero_page, 5); }, // unofficial
            0x08 => { instruction!(php, 3); },
            0x09 => { instruction!(ora, addressing_mode::immediate, 2); },
            0x0a => { instruction!(asl, addressing_mode::accumulator, 2); },
            0x0b => { instruction!(anc, addressing_mode::immediate, 2); }, // unofficial
            0x0c => { instruction!(nop_with_read, addressing_mode::absolute, 4); }, // unofficial
            0x0d => { instruction!(ora, addressing_mode::absolute, 4); },
            0x0e => { instruction!(asl, addressing_mode::absolute, 6); },
            0x0f => { instruction!(slo, addressing_mode::absolute, 6); }, // unofficial
            0x10 => { instruction!(bpl, 2); },
            0x11 => { instruction!(ora, addressing_mode::indirect_y, 5); },
            0x13 => { instruction!(slo, addressing_mode::indirect_y, 8); }, // unofficial
            0x14 => { instruction!(nop_with_read, addressing_mode::indirect_x, 4); }, // unofficial
            0x15 => { instruction!(ora, addressing_mode::zero_page_x, 4); },
            0x16 => { instruction!(asl, addressing_mode::zero_page_x, 6); },
            0x17 => { instruction!(slo, addressing_mode::zero_page_x, 6); }, // unofficial
            0x18 => { instruction!(clc, 2); },
            0x19 => { instruction!(ora, addressing_mode::absolute_y, 4); },
            0x1a => { instruction!(nop, 2); } // unofficial
            0x1b => { instruction!(slo, addressing_mode::absolute_y, 7); }, // unofficial
            0x1c => { instruction!(nop_with_read, addressing_mode::absolute_x, 4); }, // unofficial
            0x1d => { instruction!(ora, addressing_mode::absolute_x, 4); },
            0x1e => { instruction!(asl, addressing_mode::absolute_x, 7); },
            0x1f => { instruction!(slo, addressing_mode::absolute_x, 7); }, // unofficial
            0x20 => { instruction!(jsr, 6); },
            0x21 => { instruction!(and, addressing_mode::indirect_x, 6); },
            0x23 => { instruction!(rla, addressing_mode::indirect_x, 8); }, // unofficial
            0x24 => { instruction!(bit, addressing_mode::zero_page, 3); },
            0x25 => { instruction!(and, addressing_mode::zero_page, 3); },
            0x26 => { instruction!(rol, addressing_mode::zero_page, 5); },
            0x27 => { instruction!(rla, addressing_mode::zero_page, 5); }, // unofficial
            0x28 => { instruction!(plp, 4); },
            0x29 => { instruction!(and, addressing_mode::immediate, 2); },
            0x2a => { instruction!(rol, addressing_mode::accumulator, 2); },
            0x2b => { instruction!(anc, addressing_mode::immediate, 2); }, // unofficial
            0x2c => { instruction!(bit, addressing_mode::absolute, 4); },
            0x2d => { instruction!(and, addressing_mode::absolute, 4); },
            0x2e => { instruction!(rol, addressing_mode::absolute, 6); },
            0x2f => { instruction!(rla, addressing_mode::absolute, 6); }, // unofficial
            0x30 => { instruction!(bmi, 2); },
            0x31 => { instruction!(and, addressing_mode::indirect_y, 5); },
            0x33 => { instruction!(rla, addressing_mode::indirect_y, 8); }, // unofficial
            0x34 => { instruction!(nop_with_read, addressing_mode::indirect_x, 4); }, // unofficial
            0x35 => { instruction!(and, addressing_mode::zero_page_x, 4); },
            0x36 => { instruction!(rol, addressing_mode::zero_page_x, 6); },
            0x37 => { instruction!(rla, addressing_mode::zero_page_x, 6); }, // unofficial
            0x38 => { instruction!(sec, 2); },
            0x39 => { instruction!(and, addressing_mode::absolute_y, 4); },
            0x3a => { instruction!(nop, 2); } // unofficial
            0x3b => { instruction!(rla, addressing_mode::absolute_y, 7); }, // unofficial
            0x3c => { instruction!(nop_with_read, addressing_mode::absolute_x, 4); }, // unofficial
            0x3d => { instruction!(and, addressing_mode::absolute_x, 4); },
            0x3e => { instruction!(rol, addressing_mode::absolute_x, 7); },
            0x3f => { instruction!(rla, addressing_mode::absolute_x, 7); }, // unofficial
            0x40 => { instruction!(rti, 6); },
            0x41 => { instruction!(eor, addressing_mode::indirect_x, 6); },
            0x43 => { instruction!(sre, addressing_mode::indirect_x, 8); }, // unofficial
            0x44 => { instruction!(nop_with_read, addressing_mode::immediate, 3); }, // unofficial
            0x45 => { instruction!(eor, addressing_mode::zero_page, 3); },
            0x46 => { instruction!(lsr, addressing_mode::zero_page, 5); },
            0x47 => { instruction!(sre, addressing_mode::zero_page, 5); }, // unofficial
            0x48 => { instruction!(pha, 3); },
            0x49 => { instruction!(eor, addressing_mode::immediate, 2); },
            0x4a => { instruction!(lsr, addressing_mode::accumulator, 2); },
            0x4b => { instruction!(alr, addressing_mode::immediate, 2); }, // unofficial
            0x4c => { instruction!(jmp, 3); },
            0x4d => { instruction!(eor, addressing_mode::absolute, 4); },
            0x4e => { instruction!(lsr, addressing_mode::absolute, 6); },
            0x4f => { instruction!(sre, addressing_mode::absolute, 6); }, // unofficial
            0x50 => { instruction!(bvc, 2); },
            0x51 => { instruction!(eor, addressing_mode::indirect_y, 5); },
            0x53 => { instruction!(sre, addressing_mode::indirect_y, 8); }, // unofficial
            0x54 => { instruction!(nop_with_read, addressing_mode::indirect_x, 4); }, // unofficial
            0x55 => { instruction!(eor, addressing_mode::zero_page_x, 4); },
            0x56 => { instruction!(lsr, addressing_mode::zero_page_x, 6); },
            0x57 => { instruction!(sre, addressing_mode::zero_page_x, 6); }, // unofficial
            0x58 => { instruction!(cli, 2); },
            0x59 => { instruction!(eor, addressing_mode::absolute_y, 4); },
            0x5a => { instruction!(nop, 2); } // unofficial
            0x5b => { instruction!(sre, addressing_mode::absolute_y, 7); }, // unofficial
            0x5c => { instruction!(nop_with_read, addressing_mode::absolute_x, 4); }, // unofficial
            0x5d => { instruction!(eor, addressing_mode::absolute_x, 4); },
            0x5e => { instruction!(lsr, addressing_mode::absolute_x, 7); },
            0x5f => { instruction!(sre, addressing_mode::absolute_x, 7); }, // unofficial
            0x60 => { instruction!(rts, 6); },
            0x61 => { instruction!(adc, addressing_mode::indirect_x, 6); },
            0x63 => { instruction!(rra, addressing_mode::indirect_x, 8); }, // unofficial
            0x64 => { instruction!(nop_with_read, addressing_mode::immediate, 3); }, // unofficial
            0x65 => { instruction!(adc, addressing_mode::zero_page, 3); },
            0x66 => { instruction!(ror, addressing_mode::zero_page, 5); },
            0x67 => { instruction!(rra, addressing_mode::zero_page, 5); }, // unofficial
            0x68 => { instruction!(pla, 4); },
            0x69 => { instruction!(adc, addressing_mode::immediate, 2); },
            0x6a => { instruction!(ror, addressing_mode::accumulator, 2); },
            0x6b => { instruction!(arr, 2); }, // unofficial
            0x6c => { instruction!(jmp_indirect, 5); },
            0x6d => { instruction!(adc, addressing_mode::absolute, 4); },
            0x6e => { instruction!(ror, addressing_mode::absolute, 6); },
            0x6f => { instruction!(rra, addressing_mode::absolute, 6); }, // unofficial
            0x70 => { instruction!(bvs, 2); },
            0x71 => { instruction!(adc, addressing_mode::indirect_y, 5); },
            0x73 => { instruction!(rra, addressing_mode::indirect_y, 8); }, // unofficial
            0x74 => { instruction!(nop_with_read, addressing_mode::indirect_x, 4); }, // unofficial
            0x75 => { instruction!(adc, addressing_mode::zero_page_x, 4); },
            0x76 => { instruction!(ror, addressing_mode::zero_page_x, 6); },
            0x77 => { instruction!(rra, addressing_mode::zero_page_x, 6); }, // unofficial
            0x78 => { instruction!(sei, 2); },
            0x79 => { instruction!(adc, addressing_mode::absolute_y, 4); },
            0x7a => { instruction!(nop, 2); } // unofficial
            0x7b => { instruction!(rra, addressing_mode::absolute_y, 7); }, // unofficial
            0x7c => { instruction!(nop_with_read, addressing_mode::absolute_x, 4); }, // unofficial
            0x7d => { instruction!(adc, addressing_mode::absolute_x, 4); },
            0x7e => { instruction!(ror, addressing_mode::absolute_x, 7); },
            0x7f => { instruction!(rra, addressing_mode::absolute_x, 7); }, // unofficial
            0x80 => { instruction!(nop_with_read, addressing_mode::immediate, 2); }, // unofficial
            0x81 => { instruction!(sta, addressing_mode::indirect_x, 6); },
            0x82 => { instruction!(nop_with_read, addressing_mode::immediate, 2); }, // unofficial
            0x83 => { instruction!(sax, addressing_mode::indirect_x, 6); }, // unofficial
            0x84 => { instruction!(sty, addressing_mode::zero_page, 3); },
            0x85 => { instruction!(sta, addressing_mode::zero_page, 3); },
            0x86 => { instruction!(stx, addressing_mode::zero_page, 3); },
            0x87 => { instruction!(sax, addressing_mode::zero_page, 3); }, // unofficial
            0x88 => { instruction!(dey, 2); },
            0x89 => { instruction!(nop_with_read, addressing_mode::immediate, 2); }, // unofficial
            0x8a => { instruction!(txa, 2); },
            0x8c => { instruction!(sty, addressing_mode::absolute ,4); },
            0x8d => { instruction!(sta, addressing_mode::absolute, 4); },
            0x8e => { instruction!(stx, addressing_mode::absolute, 4); },
            0x8f => { instruction!(sax, addressing_mode::absolute, 4); }, // unofficial
            0x90 => { instruction!(bcc, 2); },
            0x91 => { instruction!(sta, addressing_mode::indirect_y, 6); },
            0x94 => { instruction!(sty, addressing_mode::zero_page_x, 4); },
            0x95 => { instruction!(sta, addressing_mode::zero_page_x, 4); },
            0x96 => { instruction!(stx, addressing_mode::zero_page_y, 4); },
            0x97 => { instruction!(sax, addressing_mode::zero_page_y, 4); }, // unofficial
            0x98 => { instruction!(tya, 2); },
            0x99 => { instruction!(sta, addressing_mode::absolute_y, 5); },
            0x9a => { instruction!(txs, 2); },
            0x9c => { instruction!(nop, 2); }, // wrong, but not implementing
            0x9d => { instruction!(sta, addressing_mode::absolute_x, 5); },
            0x9e => { instruction!(nop, 2); }, // wrong, but not implementing
            0xa0 => { instruction!(ldy, addressing_mode::immediate, 2); },
            0xa1 => { instruction!(lda, addressing_mode::indirect_x, 6); },
            0xa2 => { instruction!(ldx, addressing_mode::immediate, 2); },
            0xa3 => { instruction!(lax, addressing_mode::indirect_x, 6); }, // unofficial
            0xa4 => { instruction!(ldy, addressing_mode::zero_page, 3); },
            0xa5 => { instruction!(lda, addressing_mode::zero_page, 3); },
            0xa6 => { instruction!(ldx, addressing_mode::zero_page, 3); },
            0xa7 => { instruction!(lax, addressing_mode::zero_page, 3); }, // unofficial
            0xa8 => { instruction!(tay, 2); },
            0xa9 => { instruction!(lda, addressing_mode::immediate, 2); },
            0xaa => { instruction!(tax, 2); },
            0xab => { instruction!(lax, addressing_mode::immediate, 2); }, // unofficial
            0xac => { instruction!(ldy, addressing_mode::absolute, 4); },
            0xad => { instruction!(lda, addressing_mode::absolute, 4); },
            0xae => { instruction!(ldx, addressing_mode::absolute, 4); },
            0xaf => { instruction!(lax, addressing_mode::absolute, 4); }, // unofficial
            0xb0 => { instruction!(bcs, 2); },
            0xb1 => { instruction!(lda, addressing_mode::indirect_y, 5); },
            0xb3 => { instruction!(lax, addressing_mode::indirect_y, 5); }, // unofficial
            0xb4 => { instruction!(ldy, addressing_mode::zero_page_x, 4); },
            0xb5 => { instruction!(lda, addressing_mode::zero_page_x, 4); },
            0xb6 => { instruction!(ldx, addressing_mode::zero_page_y, 4); },
            0xb7 => { instruction!(lax, addressing_mode::zero_page_y, 4); }, // unofficial
            0xb8 => { instruction!(clv, 2); },
            0xb9 => { instruction!(lda, addressing_mode::absolute_y, 4); },
            0xba => { instruction!(tsx, 2); },
            0xbc => { instruction!(ldy, addressing_mode::absolute_x, 4); },
            0xbd => { instruction!(lda, addressing_mode::absolute_x, 4); },
            0xbe => { instruction!(ldx, addressing_mode::absolute_y, 4); },
            0xbf => { instruction!(lax, addressing_mode::absolute_y, 4); }, // unofficial
            0xc0 => { instruction!(cpy, addressing_mode::immediate, 2); },
            0xc1 => { instruction!(cmp, addressing_mode::indirect_x, 6); },
            0xc2 => { instruction!(nop_with_read, addressing_mode::immediate, 2); }, // unofficial
            0xc3 => { instruction!(dcp, addressing_mode::indirect_x, 8); }, // unofficial
            0xc4 => { instruction!(cpy, addressing_mode::zero_page, 3); },
            0xc5 => { instruction!(cmp, addressing_mode::zero_page, 3); },
            0xc6 => { instruction!(dec, addressing_mode::zero_page, 5); },
            0xc7 => { instruction!(dcp, addressing_mode::zero_page, 5); }, // unofficial
            0xc8 => { instruction!(iny, 2); },
            0xc9 => { instruction!(cmp, addressing_mode::immediate, 2); },
            0xca => { instruction!(dex, 2); },
            0xcb => { instruction!(axs, 2); }, // unofficial
            0xcc => { instruction!(cpy, addressing_mode::absolute, 4); },
            0xcd => { instruction!(cmp, addressing_mode::absolute, 4); },
            0xce => { instruction!(dec, addressing_mode::absolute, 6); },
            0xcf => { instruction!(dcp, addressing_mode::absolute, 6); }, // unofficial
            0xd0 => { instruction!(bne, 2); },
            0xd1 => { instruction!(cmp, addressing_mode::indirect_y, 5); },
            0xd3 => { instruction!(dcp, addressing_mode::indirect_y, 8); }, // unofficial
            0xd4 => { instruction!(nop_with_read, addressing_mode::indirect_x, 4); }, // unofficial
            0xd5 => { instruction!(cmp, addressing_mode::zero_page_x, 4); },
            0xd6 => { instruction!(dec, addressing_mode::zero_page_x, 6); },
            0xd7 => { instruction!(dcp, addressing_mode::zero_page_x, 6); }, // unofficial
            0xd8 => { instruction!(cld, 2); },
            0xd9 => { instruction!(cmp, addressing_mode::absolute_y, 4); },
            0xda => { instruction!(nop, 2); } // unofficial
            0xdb => { instruction!(dcp, addressing_mode::absolute_y, 7); }, // unofficial
            0xdc => { instruction!(nop_with_read, addressing_mode::absolute_x, 4); }, // unofficial
            0xdd => { instruction!(cmp, addressing_mode::absolute_x, 4); },
            0xde => { instruction!(dec, addressing_mode::absolute_x, 7); },
            0xdf => { instruction!(dcp, addressing_mode::absolute_x, 7); }, // unofficial
            0xe0 => { instruction!(cpx, addressing_mode::immediate, 2); },
            0xe1 => { instruction!(sbc, addressing_mode::indirect_x, 6); },
            0xe2 => { instruction!(nop_with_read, addressing_mode::immediate, 2); }, // unofficial
            0xe3 => { instruction!(isc, addressing_mode::indirect_x, 8); }, // unofficial
            0xe4 => { instruction!(cpx, addressing_mode::zero_page, 3); },
            0xe5 => { instruction!(sbc, addressing_mode::zero_page, 3); },
            0xe6 => { instruction!(inc, addressing_mode::zero_page, 5); },
            0xe7 => { instruction!(isc, addressing_mode::zero_page, 5); }, // unofficial
            0xe8 => { instruction!(inx, 2); },
            0xe9 => { instruction!(sbc, addressing_mode::immediate, 2); },
            0xea => { instruction!(nop, 2); },
            0xeb => { instruction!(sbc, addressing_mode::immediate, 2); }, // unofficial
            0xec => { instruction!(cpx, addressing_mode::absolute, 4); },
            0xed => { instruction!(sbc, addressing_mode::absolute, 4); },
            0xee => { instruction!(inc, addressing_mode::absolute, 6); },
            0xef => { instruction!(isc, addressing_mode::absolute, 6); }, // unofficial
            0xf0 => { instruction!(beq, 2); },
            0xf1 => { instruction!(sbc, addressing_mode::indirect_y, 5); },
            0xf3 => { instruction!(isc, addressing_mode::indirect_y, 8); }, // unofficial
            0xf4 => { instruction!(nop_with_read, addressing_mode::indirect_x, 4); }, // unofficial
            0xf5 => { instruction!(sbc, addressing_mode::zero_page_x, 4); },
            0xf6 => { instruction!(inc, addressing_mode::zero_page_x, 6); },
            0xf7 => { instruction!(isc, addressing_mode::zero_page_x, 7); }, // unofficial
            0xf8 => { instruction!(sed, 2); },
            0xf9 => { instruction!(sbc, addressing_mode::absolute_y, 4); },
            0xfa => { instruction!(nop, 2); } // unofficial
            0xfb => { instruction!(isc, addressing_mode::absolute_y, 7); }, // unofficial
            0xfc => { instruction!(nop_with_read, addressing_mode::absolute_x, 4); }, // unofficial
            0xfd => { instruction!(sbc, addressing_mode::absolute_x, 4); },
            0xfe => { instruction!(inc, addressing_mode::absolute_x, 7); },
            0xff => { instruction!(isc, addressing_mode::absolute_x, 7); }, // unofficial
            _ => panic!("Unknown opcode: {:02X}", opcode)
        }
    }
    
    pub fn load_byte_from_pc(&mut self) -> u8 {
        let pc = self.reg_pc;
        let value = self.load_byte(pc);
        self.reg_pc += 1;
        
        value
    }
    
    pub fn load_word_from_pc(&mut self) -> u16 {
        let pc = self.reg_pc;
        let value = self.load_word(pc);
        self.reg_pc += 2;
        
        value
    }
    
    fn ppu_oam_dma(&mut self, val: u8) {
        let start_addr = val as u16 * 0x100;
        let end_addr = start_addr + 256;
        
        // "1 dummy read cycle while waiting for writes to complete"
        self.cycle += 1;
        // "+1 if on an odd CPU cycle"
        if self.cycle % 2 == 1 {
            self.cycle += 1;
        }
        
        for addr in start_addr .. end_addr {
            let val = self.load_byte(addr);
            self.memory_interface.store_byte(memory::PPU_OAM_DATA, val);
            
            self.cycle += 2;
        }
    }
    
    // instruction helpers
    
    fn stack_push_byte(&mut self, val: u8) {
        let stack_addr = STACK_START + self.reg_sp as u16;
        self.store_byte(stack_addr, val);
        self.reg_sp -= 1;
    }
    
    fn stack_push_word(&mut self, val: u16) {
        let stack_addr = STACK_START + (self.reg_sp - 1) as u16;
        self.store_word(stack_addr, val);
        self.reg_sp -= 2;
    }
    
    fn stack_pop_byte(&mut self) -> u8 {
        let stack_addr = STACK_START + (self.reg_sp + 1) as u16;
        let val = self.load_byte(stack_addr);
        self.reg_sp += 1;
        val
    }
    
    fn stack_pop_word(&mut self) -> u16 {
        let stack_addr = STACK_START + (self.reg_sp + 1) as u16;
        let val = self.load_word(stack_addr);
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
        let pc = self.reg_pc;
        self.current_instruction = self.load_byte(pc);
        
        println!("{:?}", self);
    }
    
    
    // instructions
    
    fn brk(&mut self) {
        self.reg_pc += 1;
        let pc = self.reg_pc;
        
        self.reg_p.break_command = true;
        self.reg_p.bit5 = true;
        
        let status = self.reg_p.as_u8();

        self.stack_push_word(pc);
        self.stack_push_byte(status);

        self.reg_p.interrupt_disable = true;
        
        self.reg_pc = self.load_word(BRK_VECTOR);
    }
    
    fn jmp(&mut self) {
        self.reg_pc = self.load_word_from_pc();
    }
    // special case to handle the cpu bug rather than being an addressing mode
    fn jmp_indirect(&mut self) {
        let addr = self.load_word_from_pc();
        
        // cpu bug: if the jmp vector goes to xxff, the msb is pulled from xx00
        let lsb: u8 = self.load_byte(addr);
        let msb: u8 = self.load_byte((addr & 0xff00) | ((addr + 1) & 0x00ff));
        
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
    fn cli(&mut self) {
        self.reg_p.interrupt_disable = false;
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

    // and then lsr A
    fn alr<T:AddressingMode>(&mut self, mode: T) {
        self.and(mode);
        let a_mode = addressing_mode::accumulator(self);
        self.lsr(a_mode);
    }

    // Does AND #i, setting N and Z flags based on the result. Then it copies N (bit 7) to C. 
    fn anc<T:AddressingMode>(&mut self, mode: T) {
        self.and(mode);
        self.reg_p.carry = self.reg_p.negative;
    }

    // Similar to AND #i then ROR A, except sets the flags differently. N and Z are normal, but C is bit 6 and V is bit 6 xor bit 5.
    fn arr(&mut self) {
        // TODO
    }

    // Sets X to {(A AND X) - #value without borrow}, and updates NZC.
    fn axs(&mut self) {
        // TODO
    }
}

impl Memory for Cpu {
    fn load_byte(&mut self, addr: u16) -> u8 {
        self.memory_interface.load_byte(addr)
    }
    fn store_byte(&mut self, addr: u16, val: u8) {
        if addr == memory::PPU_OAM_DMA {
            self.ppu_oam_dma(val);
        } else {
            self.memory_interface.store_byte(addr, val);
        }
    }
}

impl fmt::Debug for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04X} {:20} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{}",
            self.reg_pc,
            opcode::decode(self.current_instruction),
            self.reg_a,
            self.reg_x,
            self.reg_y,
            self.reg_p.as_u8(),
            self.reg_sp,
            self.cycle)
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