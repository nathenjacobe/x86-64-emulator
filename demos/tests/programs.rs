//! runs every demo program to completion and checks the documented result

use demos::{demos, machine, rax};

#[test]
fn every_demo_halts_with_the_expected_rax() {
    for demo in demos() {
        let mut machine = machine(demo);
        machine
            .run(100_000)
            .unwrap_or_else(|err| panic!("demo {}: {err}", demo.name));
        assert!(machine.cpu.halted, "demo {} did not halt", demo.name);
        assert_eq!(rax(&machine), demo.expected_rax, "demo {}", demo.name);
    }
}

#[test]
fn reset_reruns_the_same_program() {
    for demo in demos() {
        let mut machine = machine(demo);
        machine.run(100_000).unwrap();
        let first = rax(&machine);

        machine.reset();
        assert_eq!(rax(&machine), 0, "demo {} was not reset", demo.name);
        machine.run(100_000).unwrap();
        assert_eq!(rax(&machine), first, "demo {} was not deterministic", demo.name);
    }
}
