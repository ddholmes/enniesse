use memory::Memory;

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
}

impl Memory for Input {
    fn load_byte(&mut self, addr: u16) -> u8 {
        0
    }
    fn store_byte(&mut self, addr: u16, val: u8) {
        
    }
}

#[derive(Default)]
struct ControllerState {
    a: bool,
    b: bool,
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    select: bool,
    start: bool
}