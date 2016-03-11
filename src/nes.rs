use super::cpu::Cpu;
use super::rom::Rom;
use super::ppu;
use super::display::Display;

use sdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

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
        let sdl = sdl2::init().unwrap();
        let mut event_pump = sdl.event_pump().unwrap();
        
        let mut display = Display::new(&sdl, ppu::SCREEN_WIDTH as u32, ppu::SCREEN_HEIGHT as u32);
        
        self.cpu.reset();
        
        'main: loop {
            //self.cpu.trace_state();
            self.cpu.run_instruction();
            
            if self.cpu.cycle >= ppu::CPU_CYCLES_PER_SCANLINE as usize {
                let result = self.cpu.memory_interface.ppu.run();
                
                if result.vblank {
                    self.cpu.nmi();
                }
                
                if result.mapper_irq {
                    self.cpu.irq();
                }
                
                if result.render_frame {
                    display.render(&*self.cpu.memory_interface.ppu.display_buffer);
                }
                
                self.cpu.cycle = self.cpu.cycle % ppu::CPU_CYCLES_PER_SCANLINE as usize;
                
                for event in event_pump.poll_iter() {
                    match event {
                        Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                            break 'main;
                        },
                        _ => {}
                    }
                }
            }
        }
    }
}