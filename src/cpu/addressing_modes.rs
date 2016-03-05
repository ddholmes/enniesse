use super::Cpu;
use super::super::memory::Memory;

pub trait AddressingMode {
    fn load(&self, cpu: &mut Cpu) -> u8;
    fn store(&self, cpu: &mut Cpu, value: u8);
}

pub struct ImmediateAddressingMode;
impl AddressingMode for ImmediateAddressingMode {
    fn load(&self, cpu: &mut Cpu) -> u8 {
        cpu.load_byte_from_pc()
    }
    fn store(&self, _: &mut Cpu, _: u8) {
        panic!("Store not supported for immediate addressing mode.");
    }
}

pub struct AccumulatorAddressingMode;
impl AddressingMode for AccumulatorAddressingMode {
    fn load(&self, cpu: &mut Cpu) -> u8 {
        cpu.reg_a
    }
    fn store(&self, cpu: &mut Cpu, value: u8) {
        cpu.reg_a = value;
    }
}

pub struct MemoryAddressingMode {
    address: u16
}
impl AddressingMode for MemoryAddressingMode {
    fn load(&self, cpu: &mut Cpu) -> u8 {
        cpu.memory_interface.load_byte(self.address)
    }
    fn store(&self, cpu: &mut Cpu, value: u8) {
        cpu.memory_interface.store_byte(self.address, value);
    }
}

impl MemoryAddressingMode {
    pub fn zero_page(cpu: &mut Cpu) -> MemoryAddressingMode {
        MemoryAddressingMode { address: cpu.load_byte_from_pc() as u16 }
    }
    
    pub fn zero_page_x(cpu: &mut Cpu) -> MemoryAddressingMode {
        MemoryAddressingMode { address: cpu.load_byte_from_pc().wrapping_add(cpu.reg_x) as u16 }
    }
    
    pub fn zero_page_y(cpu: &mut Cpu) -> MemoryAddressingMode {
        MemoryAddressingMode { address: cpu.load_byte_from_pc().wrapping_add(cpu.reg_y) as u16 }
    }
    
    pub fn absolute(cpu: &mut Cpu) -> MemoryAddressingMode {
        MemoryAddressingMode { address: cpu.load_word_from_pc() as u16 }
    }
    
    pub fn absolute_x(cpu: &mut Cpu) -> MemoryAddressingMode {
        MemoryAddressingMode { address: cpu.load_word_from_pc() + cpu.reg_x as u16 }
    }
    
    pub fn absolute_y(cpu: &mut Cpu) -> MemoryAddressingMode {
        MemoryAddressingMode { address: cpu.load_word_from_pc().wrapping_add(cpu.reg_y as u16) as u16 }
    }
    
    pub fn indirect_x(cpu: &mut Cpu) -> MemoryAddressingMode {
        let val = cpu.load_byte_from_pc();
        let x = cpu.reg_x;
        
        MemoryAddressingMode { address: cpu.memory_interface.load_word_zero_page(val.wrapping_add(x)) }
    }
    
    pub fn indirect_y(cpu: &mut Cpu) -> MemoryAddressingMode {
        let val = cpu.load_byte_from_pc();
        let y = cpu.reg_y as u16;
        
        MemoryAddressingMode { address: cpu.memory_interface.load_word_zero_page(val).wrapping_add(y) }
    }
}