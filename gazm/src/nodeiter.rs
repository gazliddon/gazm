use crate::node::{CtxTrait, BaseNode};

use super::item::Node;

#[derive(Clone, Debug)]
pub enum IterState<'a, I : Clone, C : CtxTrait> {
    IterateKids {
        node: &'a BaseNode<I,C>,
        kid_num: usize,
        depth: usize,
    },
    Root(&'a BaseNode<I,C>),
}

impl<'a, I: Clone, C: CtxTrait> IterState<'a, I,C> {
    pub fn root(node: &'a BaseNode<I,C>) -> Self {
        IterState::Root(node)
    }
    pub fn iter_kids(node: &'a BaseNode<I,C>, depth: usize, kid_num: usize) -> Self {
        IterState::IterateKids {
            node,
            kid_num,
            depth,
        }
    }
}

use grl_sources::grl_utils::Stack;

pub struct NodeIter<'a, I: Clone, C: CtxTrait> {
    node_stack: Stack<IterState<'a,I,C>>,
}

impl<'a,I: Clone, C: CtxTrait> NodeIter<'a, I,C> {
    pub fn pop_state(&mut self) {
        self.node_stack.pop();
    }

    pub fn push_state(&mut self, st: IterState<'a,I,C>) {
        self.node_stack.push(st)
    }

    pub fn set_state(&mut self, st: IterState<'a,I,C>) {
        self.node_stack.front_mut().map(|x| *x = st);
    }

    pub fn get_state(&self) -> Option<IterState<'a,I,C>> {
        self.node_stack.front().cloned()
    }

    pub fn new(n: &'a BaseNode<I,C>) -> Self {
        let mut node_stack = Stack::new();
        let state = IterState::Root(n);
        node_stack.push(state);
        Self { node_stack }
    }
}

#[derive(Debug)]
pub struct NodeInfo<'a, I : Clone, C: CtxTrait> {
    pub node: &'a BaseNode<I,C>,
    pub depth: usize,
    pub kid_num: usize,
}

impl<'a, I: Clone, C: CtxTrait> NodeInfo<'a, I,C> {
    pub fn new(node: &'a BaseNode<I,C>, depth: usize, kid_num: usize) -> Self {
        Self {
            node,
            depth,
            kid_num,
        }
    }

    pub fn has_kids(&self) -> bool {
        self.node.children.len() != 0
    }
}

impl<'a,I : Clone,C: CtxTrait> Iterator for NodeIter<'a, I,C> {
    type Item = NodeInfo<'a,I,C>;

    fn next(&mut self) -> Option<Self::Item> {
        use IterState::*;

        let state = self.get_state();

        match state {
            None => None,

            Some(Root(n)) => {
                let ret = NodeInfo::new(n, 0, 0);

                if ret.has_kids() {
                    self.set_state(IterState::iter_kids(n, 1, 0));
                } else {
                    self.pop_state();
                };
                Some(ret)
            }

            Some(IterateKids {
                node,
                depth,
                kid_num,
            }) => {
                let is_last = kid_num == node.children.len() - 1;
                let kid_node = &node.children[kid_num];
                let ret = NodeInfo::new(kid_node, depth, kid_num);

                if is_last {
                    self.pop_state()
                } else {
                    self.set_state(IterState::iter_kids(node, depth, kid_num + 1))
                }

                if ret.has_kids() {
                    self.push_state(IterState::iter_kids(kid_node, depth + 1, 0));
                }

                Some(ret)
            }
        }
    }
}
