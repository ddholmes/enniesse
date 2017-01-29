use super::cpu::Cpu;
use super::rom::Rom;
use super::ppu;
use input::Button;

use minifb::{Key, WindowOptions, Window, Scale};

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
        let mut window = Window::new("nesrs", ppu::SCREEN_WIDTH, ppu::SCREEN_HEIGHT,
                                WindowOptions { 
                                    borderless: false,
                                    title: true,
                                    resize: false,
                                    scale: Scale::X2,
                                }).unwrap_or_else(|e| {
                                    panic!("{}", e);
                                });
        
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
                    let mut buffer: Vec<u32> = vec![0; ppu::SCREEN_WIDTH * ppu::SCREEN_HEIGHT * 3];

                    for i in 0 .. ppu::SCREEN_WIDTH * ppu::SCREEN_HEIGHT {
                        buffer[i] = (self.cpu.memory_interface.ppu.display_buffer[i * 3] as u32) << 16 |
                                    (self.cpu.memory_interface.ppu.display_buffer[i * 3 + 1] as u32) << 8 |
                                    self.cpu.memory_interface.ppu.display_buffer[i * 3 + 2] as u32;
                    }
                    window.update_with_buffer(&buffer);
                }
                
                self.cpu.cycle = self.cpu.cycle % ppu::CPU_CYCLES_PER_SCANLINE as usize;

                if !window.is_open() || window.is_key_down(Key::Escape) {
                    break 'main;
                }

                self.read_keys(&window);
            }
        }
    }

    fn read_keys(&mut self, window: &Window) {
        self.cpu.memory_interface.input.handle_input(Button::A, window.is_key_down(Key::Z));
        self.cpu.memory_interface.input.handle_input(Button::B, window.is_key_down(Key::X));
        self.cpu.memory_interface.input.handle_input(Button::Select, window.is_key_down(Key::RightShift));
        self.cpu.memory_interface.input.handle_input(Button::Start, window.is_key_down(Key::Enter));
        self.cpu.memory_interface.input.handle_input(Button::Up, window.is_key_down(Key::Up));
        self.cpu.memory_interface.input.handle_input(Button::Down, window.is_key_down(Key::Down));
        self.cpu.memory_interface.input.handle_input(Button::Left, window.is_key_down(Key::Left));
        self.cpu.memory_interface.input.handle_input(Button::Right, window.is_key_down(Key::Right));
    }
}