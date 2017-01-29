use minifb::{Key, WindowOptions, Window, Scale};

use nes::*;
use input::Button;
use ppu;
use rom::*;

pub struct Emu {
    window: Window,
    pub nes: Nes,
}

impl Emu {
    pub fn new(rom: Rom) -> Emu {
        Emu {
            window: Window::new("nesrs", ppu::SCREEN_WIDTH, ppu::SCREEN_HEIGHT,
                                WindowOptions { 
                                    borderless: false,
                                    title: true,
                                    resize: false,
                                    scale: Scale::X2,
                                }).unwrap_or_else(|e| {
                                    panic!("{}", e);
                                }),
            nes: Nes::new(Box::new(rom)),
        }
    }

    pub fn start(&mut self) {
        self.nes.power_on();

        let mut buffer: Vec<u32> = vec![0; ppu::SCREEN_WIDTH * ppu::SCREEN_HEIGHT * 3];
        while self.window.is_open() && !self.window.is_key_down(Key::Escape) {
            let (_, render) = self.nes.step();

            if render {
                for i in 0 .. ppu::SCREEN_WIDTH * ppu::SCREEN_HEIGHT {
                    buffer[i] = (self.nes.cpu.memory_interface.ppu.display_buffer[i * 3] as u32) << 16 |
                                (self.nes.cpu.memory_interface.ppu.display_buffer[i * 3 + 1] as u32) << 8 |
                                self.nes.cpu.memory_interface.ppu.display_buffer[i * 3 + 2] as u32;
                }
                self.window.update_with_buffer(&buffer);
            }

            self.read_keys();
        }
    }

    fn read_keys(&mut self) {
        self.nes.cpu.memory_interface.input.handle_input(Button::A, self.window.is_key_down(Key::Z));
        self.nes.cpu.memory_interface.input.handle_input(Button::B, self.window.is_key_down(Key::X));
        self.nes.cpu.memory_interface.input.handle_input(Button::Select, self.window.is_key_down(Key::RightShift));
        self.nes.cpu.memory_interface.input.handle_input(Button::Start, self.window.is_key_down(Key::Enter));
        self.nes.cpu.memory_interface.input.handle_input(Button::Up, self.window.is_key_down(Key::Up));
        self.nes.cpu.memory_interface.input.handle_input(Button::Down, self.window.is_key_down(Key::Down));
        self.nes.cpu.memory_interface.input.handle_input(Button::Left, self.window.is_key_down(Key::Left));
        self.nes.cpu.memory_interface.input.handle_input(Button::Right, self.window.is_key_down(Key::Right));
    }
}