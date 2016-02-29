use std::fmt;
use std::io::Read;

const FILE_HEADER: [u8; 4] = *b"NES\x1a";

pub struct Rom {
    // number of 16k pages
    pub prg_rom_size: u8,
    // number of 8k pages
    pub chr_rom_size: u8,
    pub mapper: u8,
    
    pub prg_rom: Box<[u8]>,
    pub chr_rom: Box<[u8]>
}

// TODO: load from reader rather than a buffer?
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
        
        let prg_rom_bytes: usize = prg_rom_size as usize * 16384;
        let chr_rom_bytes: usize = chr_rom_size as usize * 8192;
        
        let prg_rom_end = 16 + prg_rom_bytes;
        let chr_rom_end = prg_rom_end + chr_rom_bytes;
        
        // TODO: error handling
        let mut prg_rom = Vec::<u8>::new();
        let mut rom_data = &value[16..prg_rom_end]; 
        rom_data.read_to_end(&mut prg_rom).unwrap();
        
        let mut chr_rom = Vec::<u8>::new();
        let mut rom_data = &value[prg_rom_end..chr_rom_end];
        rom_data.read_to_end(&mut chr_rom).unwrap();
        
        Rom {
            prg_rom_size: prg_rom_size,
            chr_rom_size: chr_rom_size,
            mapper: mapper_upper | (mapper_lower >> 4),
            prg_rom: prg_rom.into_boxed_slice(),
            chr_rom: chr_rom.into_boxed_slice()
        }
    }
}

impl fmt::Debug for Rom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Debug not implemented for Rom")
    }
}