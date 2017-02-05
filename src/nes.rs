use cpu::Cpu;
use rom::Rom;
use ppu;

#[derive(Debug)]
pub struct Nes {
    pub cpu: Cpu
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
    }

    pub fn step(&mut self) -> (usize, bool) {
        //self.cpu.trace_state();
        self.cpu.run_instruction();
        
        let cycles = self.cpu.cycle;
        let mut render = false;

        // 256 ppu cycles (~85 cpu cycles) for the visible pixels
        if cycles >= 85 && self.cpu.memory_interface.ppu.cycle == 0 {
            self.cpu.memory_interface.ppu.run(true);
        }
        if cycles >= ppu::CPU_CYCLES_PER_SCANLINE as usize {
            let result = self.cpu.memory_interface.ppu.run(false);
            
            if result.vblank {
                self.cpu.nmi();
            }
            
            if result.mapper_irq {
                self.cpu.irq();
            }

            render = result.render_frame;
            
            self.cpu.cycle = cycles % ppu::CPU_CYCLES_PER_SCANLINE as usize;
        }

        (cycles, render)
    }
}