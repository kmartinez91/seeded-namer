use std::env;
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use seeded_namer::{name_sequence_with_style, Style};

fn main() -> ExitCode {
    let raw_args: Vec<String> = env::args().skip(1).collect();

    // Pull `--style <name>` out of the argument list wherever it appears,
    // leaving the rest as positional seed/count arguments.
    let mut style = Style::Default;
    let mut args: Vec<String> = Vec::with_capacity(raw_args.len());
    let mut i = 0;
    while i < raw_args.len() {
        if raw_args[i] == "--style" {
            let value = match raw_args.get(i + 1) {
                Some(v) => v,
                None => {
                    eprintln!("--style requires a value (default, harsh, soft, sci-fi)");
                    return ExitCode::FAILURE;
                }
            };
            style = match value.parse::<Style>() {
                Ok(style) => style,
                Err(e) => {
                    eprintln!("{}", e);
                    return ExitCode::FAILURE;
                }
            };
            i += 2;
        } else {
            args.push(raw_args[i].clone());
            i += 1;
        }
    }

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

    for name in name_sequence_with_style(seed, count, style) {
        println!("{}", name);
    }

    ExitCode::SUCCESS
}
