use std::env;
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use seeded_namer::name_sequence;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    let seed = match args.first() {
        Some(s) => match s.parse::<u64>() {
            Ok(seed) => seed,
            Err(_) => {
                eprintln!("seed must be a non-negative integer, got {:?}", s);
                return ExitCode::FAILURE;
            }
        },
        // No seed given: fall back to the current time so the tool is still
        // usable interactively. This is the only place non-determinism is
        // allowed to enter; everything downstream of `seed` is pure.
        None => SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0),
    };

    let count = match args.get(1) {
        Some(s) => match s.parse::<usize>() {
            Ok(count) => count,
            Err(_) => {
                eprintln!("count must be a non-negative integer, got {:?}", s);
                return ExitCode::FAILURE;
            }
        },
        None => 1,
    };

    for name in name_sequence(seed, count) {
        println!("{}", name);
    }

    ExitCode::SUCCESS
}
