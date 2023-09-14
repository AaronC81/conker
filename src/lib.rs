use std::{thread, collections::HashMap, process::exit};

use interpreter::{TaskState, TaskID, Globals, Value, InterpreterError};
use node::{Node, NodeKind};

use crate::{node::{BinaryOperator, ItemKind}, tokenizer::Tokenizer, parser::Parser, runtime::Runtime};

pub mod node;
pub mod interpreter;
pub mod parser;
pub mod tokenizer;
pub mod runtime;

pub fn run_code(input: &str) -> Option<HashMap<String, Result<Value, InterpreterError>>> {
    // Tokenize
    let input_chars: Vec<_> = input.chars().collect();
    let mut tokenizer = Tokenizer::new(&input_chars);
    tokenizer.tokenize();
    
    if !tokenizer.errors.is_empty() {
        println!("Errors: {:#?}", tokenizer.errors);
        return None;
    }

    // Parse
    let mut parser = Parser::new(&tokenizer.tokens);
    parser.parse_top_level();

    if !parser.errors.is_empty() {
        println!("Errors: {:#?}", parser.errors);
        return None;
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
    Some(runtime.join())
}
