use super::opcode::Opcode;
use super::opcode::Opcode::*;
use super::cpu::Cpu;

pub struct Instruction {
    pub opcode: Opcode, 
    pub addressing_mode: Option<Box<AddressingMode>>
}

impl Instruction {
    fn new(opcode: Opcode, addressing_mode: Option<Box<AddressingMode>>) -> Instruction {
        Instruction {
            opcode: opcode,
            addressing_mode: addressing_mode
        }
    }
    
    pub fn decode(opcode: u8) -> Instruction {
        match opcode {
            0x08 => Instruction::new(Php, None),
            0x09 => Instruction::new(Ora, Some(Box::new(ImmediateAddressingMode))),
            0x10 => Instruction::new(Bpl, Some(Box::new(ImmediateAddressingMode))),
            0x18 => Instruction::new(Clc, None),
            0x20 => Instruction::new(Jsr, Some(Box::new(AbsoluteAddressingMode))),
            0x24 => Instruction::new(Bit, Some(Box::new(ZeroPageAddressingMode))),
            0x28 => Instruction::new(Plp, None),
            0x29 => Instruction::new(And, Some(Box::new(ImmediateAddressingMode))),
            0x30 => Instruction::new(Bmi, Some(Box::new(ImmediateAddressingMode))),
            0x38 => Instruction::new(Sec, None),
            0x48 => Instruction::new(Pha, None),
            0x49 => Instruction::new(Eor, Some(Box::new(ImmediateAddressingMode))),
            0x4c => Instruction::new(Jmp, Some(Box::new(AbsoluteAddressingMode))),
            0x50 => Instruction::new(Bvc, Some(Box::new(ImmediateAddressingMode))),
            0x60 => Instruction::new(Rts, None),
            0x68 => Instruction::new(Pla, None),
            0x69 => Instruction::new(Adc, Some(Box::new(ImmediateAddressingMode))),
            0x70 => Instruction::new(Bvs, Some(Box::new(ImmediateAddressingMode))),
            0x78 => Instruction::new(Sei, None),
            0x85 => Instruction::new(Sta, Some(Box::new(ZeroPageAddressingMode))),
            0x86 => Instruction::new(Stx, Some(Box::new(ZeroPageAddressingMode))),
            0x88 => Instruction::new(Dey, None),
            0x8a => Instruction::new(Txa, None),
            0x90 => Instruction::new(Bcc, Some(Box::new(ImmediateAddressingMode))),
            0x98 => Instruction::new(Tya, None),
            0xa0 => Instruction::new(Ldy, Some(Box::new(ImmediateAddressingMode))),
            0xa2 => Instruction::new(Ldx, Some(Box::new(ImmediateAddressingMode))),
            0xa8 => Instruction::new(Tay, None),
            0xa9 => Instruction::new(Lda, Some(Box::new(ImmediateAddressingMode))),
            0xaa => Instruction::new(Tax, None),
            0xb0 => Instruction::new(Bcs, Some(Box::new(ImmediateAddressingMode))),
            0xb8 => Instruction::new(Clv, None),
            0xba => Instruction::new(Tsx, None),
            0xc0 => Instruction::new(Cpy, Some(Box::new(ImmediateAddressingMode))),
            0xc8 => Instruction::new(Iny, None),
            0xc9 => Instruction::new(Cmp, Some(Box::new(ImmediateAddressingMode))),
            0xca => Instruction::new(Dex, None),
            0xd0 => Instruction::new(Bne, Some(Box::new(ImmediateAddressingMode))),
            0xd8 => Instruction::new(Cld, None),
            0xe0 => Instruction::new(Cpx, Some(Box::new(ImmediateAddressingMode))),
            0xe8 => Instruction::new(Inx, None),
            0xe9 => Instruction::new(Sbc, Some(Box::new(ImmediateAddressingMode))),
            0xea => Instruction::new(Nop, None),
            0xf0 => Instruction::new(Beq, Some(Box::new(ImmediateAddressingMode))),
            0xf8 => Instruction::new(Sed, None),
            _ => panic!("Unknown opcode: {:X}", opcode)
        }
    }
}

pub trait AddressingMode {
    fn load(&self, cpu: &mut Cpu) -> u8;
    fn store(&self, cpu: &mut Cpu, value: u8);
}

struct ImmediateAddressingMode;
impl AddressingMode for ImmediateAddressingMode {
    fn load(&self, cpu: &mut Cpu) -> u8 {
        cpu.load_byte_from_pc()
    }
    fn store(&self, _: &mut Cpu, _: u8) {
        panic!("Store not supported for ImmediateAddressingMode");
    }
}

struct AccumulatorAddressingMode;
impl AddressingMode for AccumulatorAddressingMode {
    fn load(&self, cpu: &mut Cpu) -> u8 {
        cpu.reg_a
    }
    fn store(&self, cpu: &mut Cpu, value: u8) {
        cpu.reg_a = value;
    }
}

trait MemoryAddressingMode : AddressingMode {
    fn load_address(&self, cpu: &mut Cpu) -> u16;
}

impl<T> AddressingMode for T where T: MemoryAddressingMode {
    fn load(&self, cpu: &mut Cpu) -> u8 {
        let addr = self.load_address(cpu);
        cpu.memory_map.load_byte(addr)
    }
    fn store(&self, cpu: &mut Cpu, value: u8) {
        let addr = self.load_address(cpu);
        cpu.memory_map.store_byte(addr, value);
    }
}

struct ZeroPageAddressingMode;
impl MemoryAddressingMode for ZeroPageAddressingMode {
    fn load_address(&self, cpu: &mut Cpu) -> u16 {
        cpu.load_byte_from_pc() as u16
    }
}

struct ZeroPageXAddressingMode;
impl MemoryAddressingMode for ZeroPageXAddressingMode {
    fn load_address(&self, cpu: &mut Cpu) -> u16 {
        (cpu.load_byte_from_pc() + cpu.reg_x) as u16
    }
}

struct ZeroPageYAddressingMode;
impl MemoryAddressingMode for ZeroPageYAddressingMode {
    fn load_address(&self, cpu: &mut Cpu) -> u16 {
        (cpu.load_byte_from_pc() + cpu.reg_y) as u16
    }
}

struct AbsoluteAddressingMode;
impl MemoryAddressingMode for AbsoluteAddressingMode {
    fn load_address(&self, cpu: &mut Cpu) -> u16 {
        cpu.load_word_from_pc() as u16
    }
}

struct AbsoluteXAddressingMode;
impl MemoryAddressingMode for AbsoluteXAddressingMode {
    fn load_address(&self, cpu: &mut Cpu) -> u16 {
        cpu.load_word_from_pc() + cpu.reg_x as u16
    }
}

struct AbsoluteYAddressingMode;
impl MemoryAddressingMode for AbsoluteYAddressingMode {
    fn load_address(&self, cpu: &mut Cpu) -> u16 {
        cpu.load_word_from_pc() + cpu.reg_y as u16
    }
}

struct IndexedIndirectAddressingMode;
impl MemoryAddressingMode for IndexedIndirectAddressingMode {
    fn load_address(&self, cpu: &mut Cpu) -> u16 {
        let val = cpu.load_byte_from_pc();
        let x = cpu.reg_x;
        
        cpu.memory_map.load_word_zero_page(val + x)
    }
}

struct IndirectIndexedAddressingMode;
impl MemoryAddressingMode for IndirectIndexedAddressingMode {
    fn load_address(&self, cpu: &mut Cpu) -> u16 {
        let val = cpu.load_byte_from_pc();
        let y = cpu.reg_y as u16;
        
        cpu.memory_map.load_word_zero_page(val) + y
    }
}