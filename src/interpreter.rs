use std::{collections::HashMap, fmt::Display};

use crossbeam_channel::{Sender, Receiver, SendError};

use crate::node::{Node, NodeKind, BinaryOperator};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TaskID(String);

impl Display for TaskID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone)]
struct InterpreterError {
    message: String,
}

impl InterpreterError {
    fn new(s: impl Into<String>) -> Self {
        Self { message: s.into() }
    }
}

impl<T> From<SendError<T>> for InterpreterError {
    fn from(value: SendError<T>) -> Self {
        InterpreterError::new(format!("send error: {value}"))
    }
}

#[derive(Debug)]
pub struct Globals {
    tasks: HashMap<String, TaskID>,
}

#[derive(Debug)]
pub struct TaskState {
    name: String,
    id: TaskID,
    body: Vec<Node>,

    locals: HashMap<String, Value>,

    receivers: HashMap<TaskID, Receiver<Value>>,
    senders: HashMap<TaskID, Sender<Value>>,
}

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Integer(i64),
    TaskReference(TaskID),
}

impl Value {
    fn get_integer(&self) -> Result<i64, InterpreterError> {
        match self {
            Value::Integer(i) => Ok(*i),
            _ => Err(InterpreterError::new("expected an integer"))
        }
    }

    fn get_task_id<'a>(&'a self, globals: &'a Globals) -> Result<TaskID, InterpreterError> {
        match self {
            Value::TaskReference(id) => Ok(id.clone()),
            _ => Err(InterpreterError::new("expected a task")),
        }
    }
}

impl TaskState {
    fn evaluate(&mut self, node: &Node, globals: &Globals) -> Result<Value, InterpreterError> {
        match &node.kind {
            NodeKind::IntegerLiteral(i)
                => Ok(Value::Integer(*i)),
            NodeKind::Identifier(name)
                => self.resolve(&name, globals),
            
            NodeKind::BinaryOperation { left, op, right } => {
                let left = self.evaluate(&left, globals)?.get_integer()?;
                let right = self.evaluate(&right, globals)?.get_integer()?;

                Ok(Value::Integer(match op {
                    BinaryOperator::Add         => left + right,
                    BinaryOperator::Subtract    => left - right,
                    BinaryOperator::Multiply    => left * right,
                    BinaryOperator::Divide      => left / right,
                }))
            }
            
            NodeKind::Send { value, channel } => {
                let value = self.evaluate(&value, globals)?;

                // Resolve the channel, and get its sender
                let channel = self.evaluate(&channel, globals)?;
                let other_task_id = channel.get_task_id(globals)?;
                let task_sender = self.get_sender_for_task(&other_task_id)?;

                // Actually perform send
                task_sender.send(value)?;

                Ok(Value::Null)
            },

            NodeKind::Receive { value, channel, bind_channel } => {
                todo!()
            }
        }
    }

    fn resolve(&self, name: &str, globals: &Globals) -> Result<Value, InterpreterError> {
        // Try locals
        if let Some(val) = self.locals.get(name) {
            return Ok(val.clone());
        }

        // Else, try tasks
        if let Some(val) = globals.tasks.get(name) {
            return Ok(Value::TaskReference(val.clone()));
        }
    
        // Give up!
        Err(InterpreterError::new(format!("could not find `{name}`")))
    }

    fn get_sender_for_task(&self, id: &TaskID) -> Result<&Sender<Value>, InterpreterError> {
        self.senders.get(id)
            .ok_or_else(|| InterpreterError::new(format!("no sender for task ID {id}")))
    }
}