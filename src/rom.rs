use std::str;
use std::fmt;

// the text "NES" and a eof chracter
const FILE_HEADER: [u8; 4] = [0x4e, 0x45, 0x53, 0x1a];

pub struct Rom {
    // number of 16k pages
    pub prg_rom_size: u8,
    // number of 8k pages
    pub chr_rom_size: u8,
    pub mapper: u8,
    pub prg_rom: Box<[u8]>
}

impl From<Box<[u8]>> for Rom {
    fn from(value: Box<[u8]>) -> Rom {
        let header = &value[0..4];
        
        if header != FILE_HEADER {
            panic!("Invalid ROM file. {:?}", header);
        }
        
        let prg_rom_size = value[4];
        let chr_rom_size = value[5];
        let flags6 = value[6];
        let flags7 = value[7];
        
        // TODO: other flags
        let mapper_lower = flags6 & 0b1111_0000;
        let mapper_upper = flags7 & 0b1111_0000;
        
        // TODO: better way?
        let mut vec = Vec::new();
        vec.extend_from_slice(&value[16..]);
        let prg_rom = vec.into_boxed_slice();
        
        Rom {
            prg_rom_size: prg_rom_size,
            chr_rom_size: chr_rom_size,
            mapper: (mapper_upper << 4) ^ mapper_lower,
            prg_rom: prg_rom
        }
    }
}

impl fmt::Debug for Rom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Debug not implemented for Rom")
    }
}