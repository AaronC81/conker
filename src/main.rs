use std::process::exit;

use concurrent_lang::run_code;

fn main() {
    let input =
"
task A
    123
";

    for (task, result) in run_code(input).unwrap().into_iter() {
        if result.is_err() {
            exit(1);
        }
    }
}
