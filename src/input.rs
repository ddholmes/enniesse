use memory::Memory;

use minifb::Key;

const CONTROLLER1_ADDR: u16 = 0x4016;
const CONTROLLER2_ADDR: u16 = 0x4017;

pub enum Button {
    A,
    B,
    Select,
    Start,
    Up,
    Down,
    Left,
    Right
}

pub struct Input {
    controller1: ControllerState,
    controller2: ControllerState
}

impl Input {
    pub fn new() -> Input {
        Input {
            controller1: ControllerState::default(),
            controller2: ControllerState::default()
        }
    }
    
    pub fn handle_input(&mut self, button: Button, pressed: bool) {
        match button {
            Button::A       => self.controller1.a      = pressed,
            Button::B       => self.controller1.b      = pressed,
            Button::Select  => self.controller1.select = pressed,
            Button::Start   => self.controller1.start  = pressed,
            Button::Up      => self.controller1.up     = pressed,
            Button::Down    => self.controller1.down   = pressed,
            Button::Left    => self.controller1.left   = pressed,
            Button::Right   => self.controller1.right  = pressed,
        }
    }
}

impl Memory for Input {
    fn load_byte(&mut self, addr: u16) -> u8 {
        match addr {
            CONTROLLER1_ADDR => 0x40 | self.controller1.get_button_state(), 
            CONTROLLER2_ADDR => 0x40 | self.controller2.get_button_state(),
            _ => 0
        }
    }
    fn store_byte(&mut self, addr: u16, val: u8) {
        match addr {
            CONTROLLER1_ADDR => self.controller1.check_reset(val),
            CONTROLLER2_ADDR => self.controller2.check_reset(val),
            _ => {}
        }
    }
}

#[derive(Default)]
struct ControllerState {
    a: bool,
    b: bool,
    select: bool,
    start: bool,
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    
    // button states are read one at a time in the order above
    next_button_read: u8,
    read_reset: bool
}

impl ControllerState {
    fn get_button_state(&mut self) -> u8 {        
        let result = match self.next_button_read {
            0 => self.a as u8,
            1 => self.b as u8,
            2 => self.select as u8,
            3 => self.start as u8,
            4 => self.up as u8,
            5 => self.down as u8,
            6 => self.left as u8,
            7 => self.right as u8,
            _ => 0
        };
        
        self.next_button_read = (self.next_button_read + 1) & 7;
        
        result
    }
    
    fn check_reset(&mut self, val: u8) {
        // writing a 1 then a 0 will reset the read state
        if val == 1 {
            self.read_reset = true;
        } else if val == 0 && self.read_reset {
            self.next_button_read = 0;
            self.read_reset = false;
        }
    }
}