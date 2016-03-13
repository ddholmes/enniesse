use super::ppu;

use sdl2::Sdl;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Renderer, Texture, TextureAccess};

const BUFFER_SIZE: usize = ppu::SCREEN_WIDTH * ppu::SCREEN_HEIGHT * 3;

pub struct Display<'a> {
    width: u32,
    height: u32,
    
    pub renderer: Box<Renderer<'a>>,
    pub display_texture: Box<Texture>
}

impl<'a> Display<'a> {
    pub fn new(sdl: &Sdl, width: u32, height: u32) -> Display<'a> {
        let video = sdl.video().unwrap();
        
        let window = video.window("nesrs", width, height)
                            .position_centered()
                            .build()
                            .unwrap();
                        
        let renderer = window.renderer().present_vsync().build().unwrap();
        
        let texture = renderer.create_texture(PixelFormatEnum::BGR24, TextureAccess::Streaming,
                        width, height).unwrap();
        
        Display {
            width: width,
            height: height,
            renderer: Box::new(renderer),
            display_texture: Box::new(texture)
        }
    }
    
    pub fn render(&mut self, display_buffer: &[u8; BUFFER_SIZE]) {
        self.display_texture.update(None, display_buffer, ppu::SCREEN_WIDTH * 3);
        
        self.renderer.clear();
        self.renderer.copy(&self.display_texture, None, None);
        self.renderer.present();
    }
}