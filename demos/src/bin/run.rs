//! loads a demo program, runs it to completion, and prints the final state
//!
//! usage: cargo run -p demos --bin run [demo-name]

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
    match machine.run(10_000) {
        Ok(steps) => {
            println!("demo: {} - {}", demo.name, demo.description);
            println!("ran {steps} step(s)\n");
            print!("{}", demos::format_cpu(&machine));
            println!("\nrax = {}", demos::rax(&machine));
        }
        Err(err) => {
            eprintln!("error: {err}");
            std::process::exit(1);
        }
    }
}
