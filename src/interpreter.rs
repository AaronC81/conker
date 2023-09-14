use std::{collections::{HashMap, BTreeMap}, fmt::Display};

use crossbeam_channel::{Sender, Receiver, SendError, Select, RecvError};

use crate::node::{Node, NodeKind, BinaryOperator};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TaskID(pub usize);

impl Display for TaskID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterpreterError {
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

impl From<RecvError> for InterpreterError {
    fn from(value: RecvError) -> Self {
        InterpreterError::new(format!("receive error: {value}"))
    }
}

#[derive(Debug, Clone)]
pub struct Globals {
    pub tasks: HashMap<String, TaskID>,
}

#[derive(Clone, Debug)]
pub struct TaskState {
    pub name: String,
    pub id: TaskID,

    pub locals: HashMap<String, Value>,

    pub receivers: HashMap<TaskID, Receiver<Value>>,
    pub senders: HashMap<TaskID, Sender<Value>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Null,
    Integer(i64),
    Boolean(bool),
    TaskReference(TaskID),
}

impl Value {
    fn is_truthy(&self) -> bool {
        match self {
            Self::Boolean(false) => false,
            Self::Null => false,

            _ => true,
        }
    }

    fn get_integer(&self) -> Result<i64, InterpreterError> {
        match self {
            Value::Integer(i) => Ok(*i),
            _ => Err(InterpreterError::new("expected an integer"))
        }
    }

    fn get_task_id<'a>(&'a self) -> Result<TaskID, InterpreterError> {
        match self {
            Value::TaskReference(id) => Ok(id.clone()),
            _ => Err(InterpreterError::new("expected a task")),
        }
    }
}

impl TaskState {
    pub fn evaluate(&mut self, node: &Node, globals: &Globals) -> Result<Value, InterpreterError> {
        match &node.kind {
            NodeKind::Body(v) => {
                let mut result = Value::Null;
                for i in v {
                    result = self.evaluate(i, globals)?;
                }
                Ok(result)
            }

            NodeKind::IntegerLiteral(i)
                => Ok(Value::Integer(*i)),
            NodeKind::BooleanLiteral(b)
                => Ok(Value::Boolean(*b)),
            NodeKind::NullLiteral
                => Ok(Value::Null),
            NodeKind::Identifier(name)
                => self.resolve(&name, globals),
            
            NodeKind::BinaryOperation { left, op, right } => {
                let left = self.evaluate(&left, globals)?.get_integer()?;
                let right = self.evaluate(&right, globals)?.get_integer()?;

                Ok(match op {
                    BinaryOperator::Add         => Value::Integer(left + right),
                    BinaryOperator::Subtract    => Value::Integer(left - right),
                    BinaryOperator::Multiply    => Value::Integer(left * right),
                    BinaryOperator::Divide      => Value::Integer(left / right),

                    BinaryOperator::Equals      => Value::Boolean(left == right),
                    BinaryOperator::LessThan    => Value::Boolean(left < right),
                    BinaryOperator::GreaterThan => Value::Boolean(left > right),
                })
            }

            NodeKind::If { condition, if_true } => {
                let condition = self.evaluate(&condition, globals)?;

                if condition.is_truthy() {
                    self.evaluate(&if_true, globals)
                } else {
                    Ok(Value::Null)
                }
            }
            
            NodeKind::Send { value, channel } => {
                let value = self.evaluate(&value, globals)?;

                // Resolve the channel, and get its sender
                let channel = self.evaluate(&channel, globals)?;
                let other_task_id = channel.get_task_id()?;
                let task_sender = self.get_sender_to_task(&other_task_id)?;

                // Actually perform send
                task_sender.send(value)?;

                Ok(Value::Null)
            },

            NodeKind::Receive { value, channel, bind_channel } => {
                if *bind_channel {
                    // Receive from anything using select
                    let ids_and_receivers: Vec<_> = self.receivers.iter().collect();
                    let mut selector = Select::new();
                    for (_, chan) in &ids_and_receivers {
                        selector.recv(chan);
                    }
                    let selected = selector.select();
                    
                    // Figure out which channel we received from
                    let (received_from, received_on_chan) = ids_and_receivers[selected.index()];

                    // Fetch sent value and result variable
                    let received_value = selected.recv(received_on_chan)?;
                    let NodeKind::Identifier(value_local) = &value.kind else {
                        return Err(InterpreterError::new("expected identifier for result of assign"))
                    };

                    // Get channel variable
                    let NodeKind::Identifier(receiver_local) = &channel.kind else {
                        return Err(InterpreterError::new("expected identifier to assign to as binding channel receiver"))
                    };

                    // Assign value and channel
                    self.create_or_assign_local(&receiver_local, Value::TaskReference(received_from.clone()));
                    self.create_or_assign_local(&value_local, received_value);

                    Ok(Value::Null)
                } else {
                    // Look up channel to receive on
                    let receiving_from_val = self.evaluate(&channel, globals)?;
                    let Value::TaskReference(id) = receiving_from_val else {
                        return Err(InterpreterError::new("tried to receive from non-channel"))
                    };

                    // Get receiver
                    let receiver = self.get_receiver_from_task(&id)?;

                    // Fetch sent value and assign into result variable
                    let received_value = receiver.recv()?;
                    let NodeKind::Identifier(value_local) = &value.kind else {
                        return Err(InterpreterError::new("expected identifier for result of assign"))
                    };
                    self.create_or_assign_local(&value_local, received_value);

                    Ok(Value::Null)
                }
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

    fn create_or_assign_local(&mut self, name: &str, value: Value) {
        if let Some(local) = self.locals.get_mut(name) {
            *local = value;
        } else {
            self.locals.insert(name.to_string(), value);
        }
    }

    fn get_sender_to_task(&self, id: &TaskID) -> Result<&Sender<Value>, InterpreterError> {
        self.senders.get(id)
            .ok_or_else(|| InterpreterError::new(format!("no sender for task ID {id}")))
    }

    fn get_receiver_from_task(&self, id: &TaskID) -> Result<&Receiver<Value>, InterpreterError> {
        self.receivers.get(id)
            .ok_or_else(|| InterpreterError::new(format!("no receiver for task ID {id}")))
    }
}
