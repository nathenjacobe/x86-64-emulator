//! top-level machine that owns a CPU and memory and exposes stepping

use crate::cpu::Cpu;
use crate::error::{EmulatorError, Result};
use crate::instruction::{self, Instruction};
use crate::memory::Memory;

/// what one call to Machine::step did
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StepInfo {
    /// address the instruction was decoded from
    pub address: u64,
    /// the decoded instruction
    pub instruction: Instruction,
    /// value of rip after execution
    pub next_rip: u64,
}

/// a CPU plus its memory
#[derive(Debug, Clone, Default)]
pub struct Machine {
    /// processor state
    pub cpu: Cpu,
    /// attached memory
    pub memory: Memory,
}

impl Machine {
    /// a machine with the default address space
    pub fn new() -> Self {
        Self::default()
    }

    /// a machine with an address space of size bytes
    pub fn with_memory_size(size: usize) -> Self {
        Self {
            cpu: Cpu::new(),
            memory: Memory::new(size),
        }
    }

    /// writes code into memory at address
    pub fn load(&mut self, address: u64, code: &[u8]) -> Result<()> {
        self.memory.load(address, code)
    }

    /// decodes and executes the instruction at rip
    pub fn step(&mut self) -> Result<StepInfo> {
        if self.cpu.halted {
            return Err(EmulatorError::Halted);
        }
        let address = self.cpu.registers.rip();
        let instruction = instruction::decode(&self.memory, address)?;
        self.cpu.execute(&instruction)?;
        Ok(StepInfo {
            address,
            instruction,
            next_rip: self.cpu.registers.rip(),
        })
    }

    /// runs until hlt, returning the number of steps executed
    ///
    /// fails with EmulatorError::StepLimitReached if the CPU has not halted
    /// after max_steps steps
    pub fn run(&mut self, max_steps: u64) -> Result<u64> {
        let mut steps = 0;
        while !self.cpu.halted {
            if steps >= max_steps {
                return Err(EmulatorError::StepLimitReached { limit: max_steps });
            }
            self.step()?;
            steps += 1;
        }
        Ok(steps)
    }

    /// resets the CPU state; memory (and any loaded program) is left intact
    pub fn reset(&mut self) {
        self.cpu = Cpu::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::Reg64;

    /// mov rax, 1; add rax, 2; hlt
    const PROGRAM: &[u8] = &[
        0x48, 0xb8, 0x01, 0, 0, 0, 0, 0, 0, 0, // mov rax, 1
        0x48, 0x83, 0xc0, 0x02, // add rax, 2
        0xf4, // hlt
    ];

    fn machine_with(code: &[u8]) -> Machine {
        let mut machine = Machine::with_memory_size(0x1000);
        machine.load(0, code).unwrap();
        machine
    }

    #[test]
    fn new_machine_starts_at_zero_and_running() {
        let machine = Machine::new();
        assert_eq!(machine.cpu.registers.rip(), 0);
        assert!(!machine.cpu.halted);
    }

    #[test]
    fn step_reports_the_instruction_and_next_rip() {
        let mut machine = machine_with(PROGRAM);
        let info = machine.step().unwrap();
        assert_eq!(info.address, 0);
        assert_eq!(info.instruction.length, 10);
        assert_eq!(info.next_rip, 10);
        assert_eq!(machine.cpu.registers.get(Reg64::Rax), 1);
    }

    #[test]
    fn run_executes_until_halt() {
        let mut machine = machine_with(PROGRAM);
        let steps = machine.run(100).unwrap();
        assert_eq!(steps, 3);
        assert_eq!(machine.cpu.registers.get(Reg64::Rax), 3);
        assert!(machine.cpu.halted);
    }

    #[test]
    fn stepping_a_halted_machine_errors() {
        let mut machine = machine_with(PROGRAM);
        machine.run(100).unwrap();
        assert_eq!(machine.step(), Err(EmulatorError::Halted));
    }

    #[test]
    fn run_stops_at_the_step_limit() {
        let mut machine = machine_with(PROGRAM);
        assert_eq!(
            machine.run(1),
            Err(EmulatorError::StepLimitReached { limit: 1 })
        );
    }

    #[test]
    fn load_reports_out_of_bounds() {
        let mut machine = Machine::with_memory_size(4);
        assert!(machine.load(2, &[0, 0, 0, 0]).is_err());
    }

    #[test]
    fn reset_clears_the_cpu_but_keeps_memory() {
        let mut machine = machine_with(PROGRAM);
        machine.run(100).unwrap();
        machine.reset();
        assert_eq!(machine.cpu.registers.get(Reg64::Rax), 0);
        assert!(!machine.cpu.halted);
        // program is still in memory, so it runs again
        assert_eq!(machine.run(100).unwrap(), 3);
    }
}
