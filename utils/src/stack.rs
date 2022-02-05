use std::{collections::VecDeque, ops::Deref};
#[derive(Debug, PartialEq, Clone)]
pub struct Stack<OP: Clone > {
    stack : VecDeque<OP>,
}

impl<OP: Clone> Default for Stack<OP> {
    fn default() -> Self {
        Self {
            stack: Default::default(),
        }
    }
}

impl<A: Clone> Deref for Stack<A> {
    type Target = VecDeque<A>;

    fn deref(&self) -> &Self::Target {
        &self.stack
    }
}

impl<OP: Clone > Stack<OP> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, op : OP) {
        self.stack.push_front(op)
    }

    pub fn pop(&mut self) -> OP {
        self.stack.pop_front().unwrap()
    }

    pub fn pop_pair(&mut self) -> (OP,OP) {
        let a = self.pop();
        let b = self.pop();
        (a,b)
    }

    pub fn flush(&mut self) -> Vec<OP> {
        let ret = self.stack.iter().cloned().collect();
        self.stack = VecDeque::new();
        ret
    }
}
