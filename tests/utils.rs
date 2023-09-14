use conker::{interpreter::{Value, InterpreterError}, run_code};

pub fn run_one_task(input: &str) -> Result<Value, InterpreterError> {
    run_code(input).unwrap().into_iter().next().unwrap().1
}

pub fn run_one_expression(input: &str) -> Result<Value, InterpreterError> {
    run_one_task(&format!("task X\n    {input}\n"))
}
