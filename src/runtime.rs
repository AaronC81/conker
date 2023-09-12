use std::{collections::HashMap, thread::{JoinHandle, self}, sync::Arc};

use crate::{interpreter::{TaskID, TaskState, Globals, Value, InterpreterError}, node::Node};

pub struct Runtime {
    globals: Globals,
    tasks: Vec<(TaskState, Node)>,

    next_task_id: TaskID,

    handles: Vec<JoinHandle<Result<Value, InterpreterError>>>,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            globals: Globals {
                tasks: HashMap::new(),
            },
            tasks: vec![],
            next_task_id: TaskID(1),
            handles: vec![],
        }
    }
    
    pub fn add_task(&mut self, name: &str, body: Node) {
        let id = self.take_task_id();
        let state = TaskState {
            name: name.to_string(),
            id,
            locals: HashMap::new(),

            receivers: HashMap::new(),
            senders: HashMap::new(),
        };

        self.tasks.push((state, body));
        self.globals.tasks.insert(name.to_string(), id);
    }

    pub fn start(&mut self) {
        for (task, body) in &mut self.tasks {
            let cloned_globals = self.globals.clone();
            let cloned_body = body.clone();

            // TODO: cloning task is Bad, probably!
            let mut cloned_task = task.clone();
            
            self.handles.push(thread::spawn(move || {
                cloned_task.evaluate(&cloned_body, &cloned_globals)
            }));
        }
    }

    pub fn join(&mut self) -> Result<(), InterpreterError> {
        for handle in self.handles.drain(..) {
            handle.join().unwrap()?;
        }

        Ok(())
    }

    pub fn create_task_channels(&mut self) {
        // TODO: not idempotent, also probably don't need to create links between *every* task
        
        // Iterate over each individual task
        for i in 0..self.tasks.len() {
            let (left, (subject, _), right) = partition_slice_mut(&mut self.tasks, i);

            // Create channel to send to all others
            // TODO: tasks can't send to themselves - is this desirable?
            for (other, _) in left.iter_mut().chain(right.iter_mut()) {
                // TODO: consider 1 for TIS-100-likeness
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
