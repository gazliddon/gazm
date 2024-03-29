#![forbid(unused_imports)]

use grl_sources::grl_utils::Stack;

#[derive(Default, Debug, Clone)]
pub struct ScopeTracker {
    stack: Stack<u64>,
}

impl ScopeTracker {
    pub fn new(scope_id: u64) -> Self {
        let mut ret = Self {
            ..Default::default()
        };
        ret.stack.push(scope_id);
        ret
    }

    pub fn scope(&self) -> u64 {
        *self.stack.front().unwrap()
    }
    pub fn set_scope(&mut self, scope_id: u64) {
        let r = self.stack.front_mut().unwrap();
        *r = scope_id;
    }

    pub fn push(&mut self, scope: u64) {
        self.stack.push(scope)
    }

    pub fn pop(&mut self) -> u64 {
        self.stack.pop().unwrap()
    }
}
