use std::{process::exit, env::args, fs};

use concurrent_lang::run_code;

fn main() {
    let args: Vec<_> = args().collect();
    if args.len() != 2 {
        println!("Usage: ... [file]");
        exit(1);
    }
    let file = &args[1];
    let input = fs::read_to_string(file).unwrap();

    for (task, result) in run_code(&input).unwrap().into_iter() {
        if result.is_err() {
            exit(1);
        }
    }
}
