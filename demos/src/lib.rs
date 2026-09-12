//! example programs that can be loaded into the emulator

use emulator::{Machine, Reg64};

/// a demo program: raw x86-64 bytes plus what they should compute
#[derive(Debug, Clone, Copy)]
pub struct Demo {
    /// short identifier used on the command line
    pub name: &'static str,
    /// human readable summary
    pub description: &'static str,
    /// bytes; loaded at address 0
    pub code: &'static [u8],
    /// expected value of rax once the program halts
    pub expected_rax: u64,
}

/// rax = 5; rax += 3; rbx = 2; rax -= rbx -> rax = 6
const ARITHMETIC: &[u8] = &[
    0x48, 0xb8, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // mov rax, 5
    0x48, 0x83, 0xc0, 0x03, // add rax, 3
    0x48, 0xbb, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // mov rbx, 2
    0x48, 0x29, 0xd8, // sub rax, rbx
    0xf4, // hlt
];

/// rax = 0; rcx = 5; do { rax += rcx; rcx -= 1 } while (rcx != 0) -> rax = 15
const LOOP: &[u8] = &[
    0x48, 0xb8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // mov rax, 0
    0x48, 0xb9, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // mov rcx, 5
    0x48, 0x01, 0xc8, // add rax, rcx
    0x48, 0xff, 0xc9, // dec rcx
    0x75, 0xf8, // jne -8 (back to add rax, rcx)
    0xf4, // hlt
];

/// rax = 1; if (rax == 1) skip; rax = 0 -> rax = 1
const CONDITIONAL: &[u8] = &[
    0x48, 0xb8, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // mov rax, 1
    0x48, 0x83, 0xf8, 0x01, // cmp rax, 1
    0x74, 0x0a, // je +10 (skip the zeroing move)
    0x48, 0xb8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // mov rax, 0
    0xf4, // hlt
];

/// rax = 42; rax ^= rax -> rax = 0
const ZERO: &[u8] = &[
    0x48, 0xb8, 0x2a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // mov rax, 42
    0x48, 0x31, 0xc0, // xor rax, rax
    0xf4, // hlt
];

static DEMOS: &[Demo] = &[
    Demo {
        name: "arithmetic",
        description: "5 + 3 - 2",
        code: ARITHMETIC,
        expected_rax: 6,
    },
    Demo {
        name: "loop",
        description: "sum 5 + 4 + 3 + 2 + 1",
        code: LOOP,
        expected_rax: 15,
    },
    Demo {
        name: "conditional",
        description: "je skips an overwrite when cmp is equal",
        code: CONDITIONAL,
        expected_rax: 1,
    },
    Demo {
        name: "zero",
        description: "xor rax, rax zeroes the accumulator",
        code: ZERO,
        expected_rax: 0,
    },
];

/// every built-in demo, in display order
pub fn demos() -> &'static [Demo] {
    DEMOS
}

/// the demo shown when none is named
pub fn default_demo() -> &'static Demo {
    &DEMOS[0]
}

/// looks up a demo by name
pub fn find(name: &str) -> Option<&'static Demo> {
    demos().iter().find(|demo| demo.name == name)
}

/// comma-separated list of demo names, for error messages
pub fn names() -> String {
    demos()
        .iter()
        .map(|demo| demo.name)
        .collect::<Vec<_>>()
        .join(", ")
}

/// builds a machine with the demo loaded at address 0
pub fn machine(demo: &Demo) -> Machine {
    let mut machine = Machine::with_memory_size(1 << 16);
    machine
        .load(0, demo.code)
        .expect("demo program fits in the address space");
    machine
}

/// formats the CPU registers and flags for display
pub fn format_cpu(machine: &Machine) -> String {
    let mut out = String::new();
    for (reg, value) in machine.cpu.registers.iter() {
        out.push_str(&format!("{reg:>4} = {value:#018x}\n"));
    }
    out.push_str(&format!(" rip = {:#018x}\n", machine.cpu.registers.rip()));
    out.push_str(&format!("flags: {}\n", machine.cpu.flags));
    out
}

/// convenience for the current rax
pub fn rax(machine: &Machine) -> u64 {
    machine.cpu.registers.get(Reg64::Rax)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_names_are_unique_and_findable() {
        for demo in demos() {
            assert_eq!(find(demo.name).map(|d| d.name), Some(demo.name));
        }
        assert!(find("does-not-exist").is_none());
    }

    #[test]
    fn default_demo_is_the_first_one() {
        assert_eq!(default_demo().name, demos()[0].name);
    }
}
