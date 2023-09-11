use std::{thread, collections::HashMap};

use interpreter::{TaskState, TaskID, Globals};
use node::{Node, NodeKind};

use crate::{node::BinaryOperator, tokenizer::Tokenizer};

mod node;
mod interpreter;
mod parser;
mod tokenizer;

fn main() {
    let input =
"
a
b
c
    d
        e
            f
    g
h
    i

";
    let input_chars: Vec<_> = input.chars().collect();
    let mut tokenizer = Tokenizer::new(&input_chars);
    tokenizer.tokenize();
    
    println!("Tokens: {:#?}", tokenizer.tokens);
    println!("Errors: {:#?}", tokenizer.errors);

    return;


    // Set up a pair of threads
    // TODO: consider 1 to emulate TIS-100 behaviour
    let (send_a_to_b, recv_from_a_in_b) = crossbeam_channel::bounded(0);
    let (send_b_to_a, recv_from_b_in_a) = crossbeam_channel::bounded(0);

    let task_a_id = TaskID(1);
    let task_b_id = TaskID(2);

    let mut task_a = TaskState {
        name: "A".to_string(),
        id: task_a_id,
        locals: HashMap::new(),

        receivers: HashMap::from([(task_b_id, recv_from_b_in_a)]),
        senders: HashMap::from([(task_b_id, send_a_to_b)]),
    };
    let task_a_body = Node {
        kind: NodeKind::Body(vec![
            Node {
                kind: NodeKind::Send {
                    value: Box::new(Node { kind: NodeKind::IntegerLiteral(123) }),
                    channel: Box::new(Node { kind: NodeKind::Identifier("B".to_string()) }),
                }
            },
            Node {
                kind: NodeKind::Send {
                    value: Box::new(Node { kind: NodeKind::IntegerLiteral(456) }),
                    channel: Box::new(Node { kind: NodeKind::Identifier("B".to_string()) }),
                }
            },
        ]),
    };

    let mut task_b = TaskState {
        name: "B".to_string(),
        id: task_b_id,
        locals: HashMap::new(),

        receivers: HashMap::from([(task_a_id, recv_from_a_in_b)]),
        senders: HashMap::from([(task_a_id, send_b_to_a)]),
    };
    let task_b_body = Node {
        kind: NodeKind::Body(vec![
            // Receive two numbers from the *same channel*
            Node {
                kind: NodeKind::Receive {
                    value: Box::new(Node { kind: NodeKind::Identifier("val1".to_string()) }),
                    channel: Box::new(Node { kind: NodeKind::Identifier("x".to_string()) }),
                    bind_channel: true,
                }
            },
            Node {
                kind: NodeKind::Receive {
                    value: Box::new(Node { kind: NodeKind::Identifier("val2".to_string()) }),
                    channel: Box::new(Node { kind: NodeKind::Identifier("x".to_string()) }),
                    bind_channel: false,
                }
            },

            // Add them
            Node {
                kind: NodeKind::BinaryOperation {
                    left: Box::new(Node { kind: NodeKind::Identifier("val1".to_string()) }),
                    op: BinaryOperator::Add, 
                    right: Box::new(Node { kind: NodeKind::Identifier("val2".to_string()) }),
                },
            }
        ])
    };

    let globals = Globals {
        tasks: HashMap::from([
            ("A".to_string(), task_a_id),
            ("B".to_string(), task_b_id),
        ]),
    };

    let globals_a = globals.clone();
    let handle_a = thread::spawn(move || task_a.evaluate(&task_a_body, &globals_a));
    let globals_b = globals.clone();
    let handle_b = thread::spawn(move || task_b.evaluate(&task_b_body, &globals_b));

    println!("a: {:?}", handle_a.join());
    println!("b: {:?}", handle_b.join());
}
