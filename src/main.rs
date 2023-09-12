use std::{thread, collections::HashMap};

use interpreter::{TaskState, TaskID, Globals};
use node::{Node, NodeKind};

use crate::{node::{BinaryOperator, ItemKind}, tokenizer::Tokenizer, parser::Parser, runtime::Runtime};

mod node;
mod interpreter;
mod parser;
mod tokenizer;
mod runtime;

fn main() {
    let input =
"
task Emitter1
    2 -> Adder
    4 -> Adder

task Emitter2
    3 -> Adder
    7 -> Adder

task Adder
    a <- ?chan
    b <- chan
    c <- ?chan
    d <- chan
    a + b + c + d
";

    // Tokenize
    let input_chars: Vec<_> = input.chars().collect();
    let mut tokenizer = Tokenizer::new(&input_chars);
    tokenizer.tokenize();
    
    if !tokenizer.errors.is_empty() {
        println!("Errors: {:#?}", tokenizer.errors);
        return;
    }

    // Parse
    let mut parser = Parser::new(&tokenizer.tokens);
    parser.parse_top_level();

    if !parser.errors.is_empty() {
        println!("Errors: {:#?}", parser.errors);
        return;
    }

    // Create a runtime with tasks
    let mut runtime = Runtime::new();
    for item in parser.items {
        match item.kind {
            ItemKind::TaskDefinition { name, body } => runtime.add_task(&name, body),
        }
    }

    // Run!
    runtime.create_task_channels();
    runtime.start();
    runtime.join().unwrap();
}
