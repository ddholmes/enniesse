use super::rom;

use std::fmt;

const RAM_SIZE: usize = 2048;

pub struct Memory {
    pub ram: [u8; RAM_SIZE],
    pub rom: rom::Rom
}

impl Memory {
    pub fn new(rom: rom::Rom) -> Memory {
        Memory {
            ram: [0; RAM_SIZE],
            rom: rom
        }
    }
}

impl fmt::Debug for Memory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Debug not implemented for Memory")
    }
}