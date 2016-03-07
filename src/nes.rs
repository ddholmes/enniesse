use super::cpu::Cpu;
use super::rom::Rom;
use super::ppu;

#[derive(Debug)]
pub struct Nes {
    cpu: Cpu
}

impl Nes {
    pub fn new(rom: Box<Rom>) -> Nes {
        let cpu = Cpu::new(rom);
        
        Nes {
            cpu: cpu
        }
    }
    
    pub fn power_on(&mut self) {
        self.cpu.reset();
        
        loop {
            self.cpu.trace_state();
            self.cpu.run_instruction();
            
            if self.cpu.cycle >= ppu::CPU_CYCLES_PER_SCANLINE as usize {
                let result = self.cpu.memory_interface.ppu.run();
                
                if result.vblank {
                    self.cpu.nmi();
                }
                
                if result.mapper_irq {
                    self.cpu.irq();
                }
                
                self.cpu.cycle = self.cpu.cycle % ppu::CPU_CYCLES_PER_SCANLINE as usize;
            }
        }
    }
}