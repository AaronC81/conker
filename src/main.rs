use std::process::exit;

use concurrent_lang::run_code;

fn main() {
    let input =
"
task Bouncer
    while true
        a <- ?c
        a -> c

task Counter
    0 -> Bouncer
    while true
        x <- Bouncer
        _ <- ?c
        (x + 1) -> c
        (x + 1) -> Bouncer

task Main
    null -> Counter
    x <- Counter
    x
";

    for (task, result) in run_code(input).unwrap().into_iter() {
        if result.is_err() {
            exit(1);
        }
    }
}
