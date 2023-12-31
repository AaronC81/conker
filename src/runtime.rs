use std::{collections::HashMap, thread::{JoinHandle, self}, sync::Arc};

use crossbeam_channel::{Receiver, Sender};

use crate::{interpreter::{TaskID, TaskState, Globals, Value, InterpreterError}, node::Node};

pub struct Runtime {
    globals: Globals,
    tasks: Vec<(TaskState, Node)>,

    next_task_id: TaskID,

    result_sender: Sender<(TaskID, String, Result<Value, InterpreterError>)>,
    result_receiver: Receiver<(TaskID, String, Result<Value, InterpreterError>)>,
}

impl Runtime {
    pub fn new() -> Self {
        let (result_sender, result_receiver) = crossbeam_channel::unbounded();

        Self {
            globals: Globals {
                task_values_by_name: HashMap::new(),
                task_descriptions_by_id: HashMap::new(),
            },
            tasks: vec![],
            next_task_id: TaskID(1),

            result_sender,
            result_receiver
        }
    }
    
    pub fn add_task(&mut self, name: &str, body: Node, instances: Option<usize>) {
        let global_value;

        if let Some(instance_count) = instances {
            let mut ids = vec![];
            for i in 0..instance_count {
                let (id, name) = self.add_one_task(name, body.clone(), Some(i));
                ids.push(Value::TaskReference(id, name));
            }
            global_value = Value::Array(ids)
        } else {
            let (id, name) = self.add_one_task(name, body, None);
            global_value = Value::TaskReference(id, name);
        }

        self.globals.task_values_by_name.insert(name.to_string(), global_value);
    }

    pub fn add_one_task(&mut self, name: &str, body: Node, index: Option<usize>) -> (TaskID, String) {
        let id = self.take_task_id();
        let state = TaskState {
            name: name.to_string(),
            id,
            index,

            locals: HashMap::new(),

            receivers: HashMap::new(),
            senders: HashMap::new(),
        };
        let name = state.formatted_name();
        self.globals.task_descriptions_by_id.insert(id, name.clone());
        self.tasks.push((state, body));

        (id, name)
    }

    pub fn start(&mut self) {
        for (task, body) in &mut self.tasks {
            let cloned_globals = self.globals.clone();
            let cloned_body = body.clone();
            let cloned_sender = self.result_sender.clone();
            let formatted_name = task.formatted_name();

            // TODO: cloning task is Bad, probably!
            let mut cloned_task = task.clone();
            
            thread::spawn(move || {
                let result = cloned_task.evaluate(&cloned_body, &cloned_globals);
                cloned_sender.send((cloned_task.id, formatted_name, result))
            });
        }
    }

    pub fn join(&mut self) -> HashMap<String, Result<Value, InterpreterError>> {
        let mut results = HashMap::new();

        // Wait for a number of results equal to the number of tasks
        // TODO: what about panics?
        for _ in 0..self.tasks.len() {
            let (id, name, result) = self.result_receiver.recv().unwrap();

            match result {
                Ok(ref value) => println!("Task {name} terminated with tail value {value:?}"),
                Err(ref e) => println!("Task {name} encountered an error: {e:?}")
            }

            results.insert(name.to_string(), result);
        }

        results
    }

    pub fn create_task_channels(&mut self) {
        // TODO: not idempotent, also probably don't need to create links between *every* task
        
        // Iterate over each individual task
        for i in 0..self.tasks.len() {
            let (left, (subject, _), right) = partition_slice_mut(&mut self.tasks, i);

            // Create channel to send to all others
            // TODO: tasks can't send to themselves - is this desirable?
            for (other, _) in left.iter_mut().chain(right.iter_mut()) {
                let (sender, receiver) = crossbeam_channel::bounded(0);
                other.receivers.insert(subject.id, receiver);
                subject.senders.insert(other.id, sender);
            }
        }
    }

    fn take_task_id(&mut self) -> TaskID {
        let result = self.next_task_id;
        self.next_task_id.0 += 1;
        result
    }
}

fn partition_slice_mut<'s, T>(slice: &'s mut [T], index: usize) -> (&'s mut [T], &'s mut T, &'s mut [T]) {
    let (left, rest) = slice.split_at_mut(index);
    let (middle, right) = rest.split_at_mut(1);
    (left, middle.first_mut().unwrap(), right)
} 
