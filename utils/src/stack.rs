use std::{collections::VecDeque, ops::Deref};
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Stack<OP: Clone> {
    stack: VecDeque<OP>,
}

enum StackError {
    CantPop,
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
// Todo some error for illegal pops
impl<OP: Clone> Stack<OP> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push<X : Into<OP>>(&mut self, op: X) {
        self.stack.push_front(op.into())
    }

    pub fn pop(&mut self) -> Option<OP> {
        self.stack.pop_front()
    }
    pub fn top(&self) -> Option<&OP> {
        self.stack.front()
    }

    /// Pop top pair from stack
    /// rhs = top, lhs = next
    /// returns (rhs, lhs)
    pub fn pop_pair(&mut self) -> Option<(OP, OP)> {
        self.pop().and_then(|rhs| self.pop().map(|lhs| (rhs,lhs)))
    }

    pub fn get_deque(&self) -> &VecDeque<OP> {
        &self.stack
    }

    pub fn flush(&mut self) -> Vec<OP> {
        let ret = self.stack.iter().cloned().collect();
        self.stack = VecDeque::new();
        ret
    }
}
