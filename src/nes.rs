use super::cpu::Cpu;
use super::rom::Rom;

use super::cpu::CpuTest;

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
        self.cpu.reset();
        
        let mut test = CpuTest::new();
        
        loop {
            self.trace_cpu(&mut test);
            self.cpu.run_instruction();
        }
    }
    
    fn trace_cpu(&mut self, test: &mut CpuTest) {
        self.cpu.trace_state();
        test.test_cpu_state(&self.cpu);
    }
}