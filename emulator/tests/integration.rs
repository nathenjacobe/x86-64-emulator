//! end-to-end tests that run small hand-assembled programs through a Machine

use emulator::{EmulatorError, Machine, Reg64};

fn machine_with(code: &[u8]) -> Machine {
    let mut machine = Machine::with_memory_size(1 << 16);
    machine.load(0, code).unwrap();
    machine
}

fn run(code: &[u8]) -> Machine {
    let mut machine = machine_with(code);
    machine.run(10_000).unwrap();
    machine
}

#[test]
fn adds_two_immediates() {
    let code = [
        0x48, 0xb8, 0x02, 0, 0, 0, 0, 0, 0, 0, // mov rax, 2
        0x48, 0x83, 0xc0, 0x03, // add rax, 3
        0xf4, // hlt
    ];
    let machine = run(&code);
    assert_eq!(machine.cpu.registers.get(Reg64::Rax), 5);
    assert!(machine.cpu.halted);
}

#[test]
fn counts_down_to_zero_and_sums() {
    // rax = 0; rcx = 4; do { rax += rcx; rcx--; } while (rcx != 0)  => rax = 10
    let code = [
        0x48, 0xb8, 0x00, 0, 0, 0, 0, 0, 0, 0, // mov rax, 0
        0x48, 0xb9, 0x04, 0, 0, 0, 0, 0, 0, 0, // mov rcx, 4
        0x48, 0x01, 0xc8, // add rax, rcx
        0x48, 0xff, 0xc9, // dec rcx
        0x75, 0xf8, // jne -8
        0xf4, // hlt
    ];
    let machine = run(&code);
    assert_eq!(machine.cpu.registers.get(Reg64::Rax), 10);
    assert_eq!(machine.cpu.registers.get(Reg64::Rcx), 0);
    assert!(machine.cpu.flags.zero);
}

#[test]
fn step_reports_instructions_in_order() {
    let code = [
        0x48, 0xb8, 0x01, 0, 0, 0, 0, 0, 0, 0, // mov rax, 1
        0x48, 0xff, 0xc0, // inc rax
        0xf4, // hlt
    ];
    let mut machine = machine_with(&code);

    let steps: Vec<_> = std::iter::from_fn(|| machine.step().ok()).collect();
    assert_eq!(steps.len(), 3);
    assert_eq!(steps[0].instruction.mnemonic, emulator::Mnemonic::Mov);
    assert_eq!(steps[1].instruction.mnemonic, emulator::Mnemonic::Inc);
    assert_eq!(steps[2].instruction.mnemonic, emulator::Mnemonic::Hlt);
    assert_eq!(steps[0].address, 0);
    assert_eq!(steps[0].next_rip, 10);
    assert_eq!(machine.cpu.registers.get(Reg64::Rax), 2);
}

#[test]
fn unimplemented_opcodes_surface_as_errors() {
    let mut machine = machine_with(&[0x0f]);
    assert_eq!(
        machine.run(10),
        Err(EmulatorError::UnimplementedOpcode {
            opcode: 0x0f,
            address: 0
        })
    );
}

#[test]
fn infinite_loop_hits_the_step_limit() {
    // jmp -2 targets itself
    let mut machine = machine_with(&[0xeb, 0xfe]);
    assert_eq!(
        machine.run(100),
        Err(EmulatorError::StepLimitReached { limit: 100 })
    );
}

#[test]
fn running_off_the_end_of_memory_is_an_error() {
    let mut machine = Machine::with_memory_size(2);
    machine.load(0, &[0x90]).unwrap();
    // nop executes, then decoding at address 1 reads one byte (0x00) and
    // fails on that unimplemented opcode
    let err = machine.run(10).unwrap_err();
    assert!(matches!(err, EmulatorError::UnimplementedOpcode { .. }));
}
