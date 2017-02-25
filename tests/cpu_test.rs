extern crate regex;
extern crate nesrs;

use std::fs::File;
use std::io::{BufReader, BufRead};
use std::fmt;

use nesrs::cpu::Cpu;
use nesrs::memory::Memory;
use nesrs::rom::Rom;

const LOG_FILE_PATH: &'static str = "tests/nestest.log";
const TEST_ROM_PATH: &'static str = "tests/nestest.nes";
const LOG_REGEX: &'static str = 
    r"(?P<PC>[0-9A-F]{4})\s+(?P<OP>[0-9A-F]{2})\s+((?:[A-F0-9]{2}\s+)*)\*?(\w{3})\s+(?P<target>[\w\d$#,=@()\s]+)\s+A:(?P<A>[0-9A-F]{2})\s+X:(?P<X>[0-9A-F]{2})\s+Y:(?P<Y>[0-9A-F]{2})\s+P:(?P<P>[0-9A-F]{2})\s+SP:(?P<SP>[0-9A-F]{2})\s+CYC:\s*(?P<CYC>\d+)\s+SL:(?P<SL>[-\d]+)";

#[test]
fn test_cpu() {
    let rom = Rom::from_file(TEST_ROM_PATH);
    let mut cpu = Cpu::new(Box::new(rom));
    let test = CpuTest::new();
    
    let log = File::open(LOG_FILE_PATH).unwrap();
    let reader = BufReader::new(log);
    
    for line in reader.lines() {
        let expected_state = test.get_state_from_line(&line.unwrap());
        CpuTest::test_state(&cpu, &expected_state);
        cpu.step();
        CpuTest::test_rom_output(&mut cpu);
    }
}

#[cfg(test)]
pub struct CpuTest {
    line_regex: regex::Regex
}

#[cfg(test)]
impl CpuTest {
    fn new() -> CpuTest {
        CpuTest {
            line_regex: regex::Regex::new(LOG_REGEX).unwrap()
        }
    }
    
    fn get_state_from_line(&self, line: &str) -> ExpectedState {
        let captures = self.line_regex.captures(line).unwrap();
        ExpectedState {
            pc: u16::from_str_radix(captures.name("PC").unwrap(), 16).unwrap(),
            opcode: u8::from_str_radix(captures.name("OP").unwrap(), 16).unwrap(),
            a: u8::from_str_radix(captures.name("A").unwrap(), 16).unwrap(),
            x: u8::from_str_radix(captures.name("X").unwrap(), 16).unwrap(),
            y: u8::from_str_radix(captures.name("Y").unwrap(), 16).unwrap(),
            p: u8::from_str_radix(captures.name("P").unwrap(), 16).unwrap(),
            sp: u8::from_str_radix(captures.name("SP").unwrap(), 16).unwrap(),
            cyc: u16::from_str_radix(captures.name("CYC").unwrap(), 10).unwrap(),
            sl: i16::from_str_radix(captures.name("SL").unwrap(), 10).unwrap()
        }
    }
    
    fn test_state(cpu: &Cpu, state: &ExpectedState) {
        let p = (cpu.reg_p.negative as u8)            << 7 |
                (cpu.reg_p.overflow as u8)            << 6 |
                (cpu.reg_p.bit5 as u8)                << 5 |
                (cpu.reg_p.break_command as u8)       << 4 |
                (cpu.reg_p.decimal_mode as u8)        << 3 |
                (cpu.reg_p.interrupt_disable as u8)   << 2 |
                (cpu.reg_p.zero as u8)                << 1 |
                (cpu.reg_p.carry as u8)               << 0;
        
        let test =  state.pc == cpu.reg_pc  &&
                    state.a  == cpu.reg_a   &&
                    state.x  == cpu.reg_x   &&
                    state.y  == cpu.reg_y   &&
                    state.p  == p           &&
                    state.sp == cpu.reg_sp;
                    
        // compare against the log output
        assert!(test, "Expected:\n{}\nActual:\n{:?}", state, cpu);
    }
    
    fn test_rom_output(cpu: &mut Cpu) {
        // the test rom puts its results in memory locations 0x02 and 0x03
        let result = cpu.memory_interface.load_byte(0x02);
        assert!(result == 0, "Test set 1 failed {:02X}\n{:?}", result, cpu);
        let result = cpu.memory_interface.load_byte(0x03);
        assert!(result == 0, "Test set 2 failed {:02X}\n{:?}", result, cpu);
    }
}

#[cfg(test)]
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

#[cfg(test)]
impl fmt::Display for ExpectedState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04X} {:02X} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{} SL:{}", self.pc, self.opcode, self.a, self.x, self.y, self.p, self.sp, self.cyc, self.sl)
    }
}