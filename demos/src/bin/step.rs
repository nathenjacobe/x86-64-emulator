//! loads a demo program and single-steps through it, printing CPU state
//!
//! usage: cargo run -p demos --bin step [demo-name]

use emulator::EmulatorError;

fn main() {
    let name = std::env::args().nth(1);
    let demo = match name.as_deref() {
        Some(name) => match demos::find(name) {
            Some(demo) => demo,
            None => {
                eprintln!("unknown demo {name:?}; available: {}", demos::names());
                std::process::exit(1);
            }
        },
        None => demos::default_demo(),
    };

    let mut machine = demos::machine(demo);
    println!("demo: {} - {}\n", demo.name, demo.description);

    loop {
        match machine.step() {
            Ok(info) => {
                println!("{:#06x}  {}", info.address, info.instruction);
                print!("{}", demos::format_cpu(&machine));
                println!();
            }
            Err(EmulatorError::Halted) => break,
            Err(err) => {
                eprintln!("error: {err}");
                std::process::exit(1);
            }
        }
    }

    println!("halted with rax = {}", demos::rax(&machine));
}
