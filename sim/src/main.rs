use std::{env, fs, process::ExitCode};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let [_, path] = args.as_slice() else {
        eprintln!("usage: sim <program.bin>");
        return ExitCode::from(2);
    };

    let program = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("sim: {path}: {e}");
            return ExitCode::FAILURE;
        }
    };
    let (mut cpu, mut bus) = match sim::boot(&program) {
        Ok(machine) => machine,
        Err(e) => {
            eprintln!("sim: {path}: {e}");
            return ExitCode::FAILURE;
        }
    };

    let (trap, retired) = sim::run(&mut cpu, &mut bus);
    println!("stopped after {retired} instructions: {trap:x?}");
    for (i, val) in cpu.regs().iter().enumerate() {
        let sep = if i % 4 == 3 { "\n" } else { "  " };
        print!("x{i:<2} = {val:#010x}{sep}");
    }
    ExitCode::SUCCESS
}
