use super::cpu;
use super::rom;

#[derive(Debug)]
pub struct Nes {
    cpu: cpu::Cpu
}

impl Nes {
    pub fn new(rom: rom::Rom) -> Nes {
        let cpu = cpu::Cpu::new(rom);
        
        Nes {
            cpu: cpu
        }
    }
    
    pub fn power_on(&mut self) {
        self.cpu.reset();
    }
}