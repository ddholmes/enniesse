use regex::Regex;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::fmt;

use super::cpu::Cpu;

const LOG_FILE: &'static str = "src/cpu/nestest.log";
const LOG_REGEX: &'static str = 
    r"(?P<PC>[0-9A-F]{4})\s+(?P<OP>[0-9A-F]{2})\s+((?:[A-F0-9]{2}\s+)*)\*?(\w{3})\s+(?P<target>[\w\d$#,=@()\s]+)\s+A:(?P<A>[0-9A-F]{2})\s+X:(?P<X>[0-9A-F]{2})\s+Y:(?P<Y>[0-9A-F]{2})\s+P:(?P<P>[0-9A-F]{2})\s+SP:(?P<SP>[0-9A-F]{2})\s+CYC:\s*(?P<CYC>\d+)\s+SL:(?P<SL>[-\d]+)";

pub struct CpuTest {
    expected_state: Vec<ExpectedState>,
    current_instruction_index: u32
}

impl CpuTest {
    pub fn new() -> CpuTest {
        let mut state_buf = Vec::<ExpectedState>::new();
        
        Self::load_log(&mut state_buf);
        
        CpuTest {
            expected_state: state_buf,
            current_instruction_index: 0
        }
    }
    
    pub fn test_cpu_state(&mut self, cpu: &Cpu) {
        let state = &self.expected_state[self.current_instruction_index as usize];
        
        let test =  state.pc == cpu.reg_pc        &&
                    state.a  == cpu.reg_a         &&
                    state.x  == cpu.reg_x         &&
                    state.y  == cpu.reg_y         &&
                    state.p  == cpu.reg_p.as_u8() &&
                    state.sp == cpu.reg_sp;
                    
        if !test {
            panic!("Expected: {}", state);
        }
        
        self.current_instruction_index += 1;
        println!("OK {}/{}", self.current_instruction_index, self.expected_state.len());
    }
    
    fn load_log(state_buf: &mut Vec<ExpectedState>) {
        let log = File::open(LOG_FILE).unwrap();
        let reader = BufReader::new(log);
        
        let regex = Regex::new(LOG_REGEX).unwrap();
        
        for line in reader.lines() {
            let line: &str = &line.unwrap();
            let captures = regex.captures(line).unwrap();
            
            let mut state = ExpectedState::default();
            state.pc = u16::from_str_radix(captures.name("PC").unwrap(), 16).unwrap();
            state.opcode = u8::from_str_radix(captures.name("OP").unwrap(), 16).unwrap();
            state.a = u8::from_str_radix(captures.name("A").unwrap(), 16).unwrap();
            state.x = u8::from_str_radix(captures.name("X").unwrap(), 16).unwrap();
            state.y = u8::from_str_radix(captures.name("Y").unwrap(), 16).unwrap();
            state.p = u8::from_str_radix(captures.name("P").unwrap(), 16).unwrap();
            state.sp = u8::from_str_radix(captures.name("SP").unwrap(), 16).unwrap();
            state.cyc = u16::from_str_radix(captures.name("CYC").unwrap(), 10).unwrap();
            state.sl = i16::from_str_radix(captures.name("SL").unwrap(), 10).unwrap();
            
            state_buf.push(state);
        }
    }
}

#[derive(Default)]
struct ExpectedState {
    pc: u16,
    opcode: u8,
    a: u8,
    x: u8,
    y: u8,
    p: u8,
    sp: u8,
    cyc: u16,
    sl: i16
}

impl fmt::Display for ExpectedState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04X} {:02X} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{} SL:{}", self.pc, self.opcode, self.a, self.x, self.y, self.p, self.sp, self.cyc, self.sl)
    }
}