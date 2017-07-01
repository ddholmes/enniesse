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

    pub fn step(&mut self) -> (u16, bool) {
        //self.cpu.trace_state();
        let cycle_start = self.cpu.cycle;
        self.cpu.step();
        let cycle_end = self.cpu.cycle;

        let mut render = false;

        if cycle_end >= ppu::CPU_CYCLES_PER_SCANLINE {
            let result = self.cpu.memory_interface.ppu.run(false);
            
            if result.vblank {
                self.cpu.nmi();
            }
            
            if result.mapper_irq {
                self.cpu.irq();
            }

            render = result.render_frame;
            
            self.cpu.cycle = cycle_end % ppu::CPU_CYCLES_PER_SCANLINE;
        } else if cycle_end >= (ppu::SCREEN_WIDTH as u16 / 3) && self.cpu.memory_interface.ppu.cycle == 0 {
            // 3 ppu cycles per cpu cycle, so 256 ppu cycles / 3 (~85 cpu cycles) for the visible pixels
            self.cpu.memory_interface.ppu.run(true);
        }

        for _ in 0 .. cycle_end - cycle_start {
            self.cpu.memory_interface.apu.step();

            if self.cpu.memory_interface.apu.dmc_interrupt || self.cpu.memory_interface.apu.frame_interrupt {
                self.cpu.irq();
            }
        }

        (cycle_end, render)
    }
}