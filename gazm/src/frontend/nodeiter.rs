use super::{BaseNode, CtxTrait};
use grl_utils::Stack;
////////////////////////////////////////////////////////////////////////////////

pub trait HasChildren {
    fn has_children(&self) -> bool {
        self.num_of_children() > 0
    }

    fn num_of_children(&self) -> usize;

    fn is_last(&self, n: usize) -> bool {
        let nk = self.num_of_children();
        match nk {
            0 => false,
            _ => n == nk - 1,
        }
    }

    fn get_child(&self, n: usize) -> Option<&Self>;
}

////////////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
pub struct NodeInfo<'a, N> {
    pub node: &'a N,
    pub depth: usize,
    pub child_num: usize,
}

impl<'a, N> NodeInfo<'a, N> {
    pub fn new(node: &'a N, depth: usize, child_num: usize) -> Self {
        Self {
            node,
            depth,
            child_num,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
#[derive(Clone, Debug)]
pub enum IterState<'a, N: Clone> {
    IterateChildren {
        node: &'a N,
        child_num: usize,
        depth: usize,
    },
    Root(&'a N),
}

impl<'a, N: Clone> IterState<'a, N> {
    pub fn root(node: &'a N) -> Self {
        IterState::Root(node)
    }
    pub fn iter_children(node: &'a N, depth: usize, child_num: usize) -> Self {
        IterState::IterateChildren {
            node,
            child_num,
            depth,
        }
    }
}

#[derive(Debug)]
pub struct NodeIter<'a, N: Clone> {
    node_stack: Stack<IterState<'a, N>>,
}

impl<'a, N: Clone> NodeIter<'a, N> {
    pub fn new(n: &'a N) -> Self {
        let mut node_stack = Stack::new();
        let state = IterState::Root(n);
        node_stack.push(state);
        Self { node_stack }
    }
}

pub trait IterTree<'a> {
    type Node: Clone + HasChildren;

    fn pop_state(&mut self) -> Option<IterState<'a, Self::Node>> {
        self.stack_mut().pop()
    }

    fn push_state(&mut self, st: IterState<'a, Self::Node>) {
        self.stack_mut().push(st)
    }

    fn set_state(&mut self, st: IterState<'a, Self::Node>) {
        if let Some(x) = self.stack_mut().front_mut() {
            *x = st
        }
    }
    fn get_state(&mut self) -> Option<IterState<'a, Self::Node>> {
        self.stack_mut().front().cloned()
    }

    fn stack_mut(&mut self) -> &mut Stack<IterState<'a, Self::Node>>;
    fn handle_root(&mut self, root: &'a Self::Node) -> NodeInfo<'a, Self::Node>;

    fn next_state(&mut self, state: IterState<'a, Self::Node>) -> Option<NodeInfo<'a, Self::Node>> {
        use IterState::*;
        match state {
            Root(root) => {
                let ret = self.handle_root(root);
                if ret.node.has_children() {
                    self.set_state(IterState::iter_children(root, 1, 0));
                } else {
                    self.pop_state();
                };
                Some(ret)
            }

            IterateChildren {
                node: parent_node,
                depth,
                child_num,
            } => {
                let child_node = parent_node.get_child(child_num).expect("getting child");
                let ret = NodeInfo::new(child_node, depth, child_num);

                if parent_node.is_last(child_num) {
                    // If we're the last item pop our state
                    self.pop_state();
                } else {
                    // Otherwise replace this state with the next child at same depth
                    self.set_state(IterState::iter_children(parent_node, depth, child_num + 1))
                }

                // If this node has a  child then iter through that node
                if ret.node.has_children() {
                    self.push_state(IterState::iter_children(ret.node, depth + 1, 0));
                }

                Some(ret)
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
impl<'a, I: Clone, CTX: CtxTrait> IterTree<'a> for NodeIter<'a, BaseNode<I, CTX>> {
    type Node = BaseNode<I, CTX>;

    fn handle_root(&mut self, root: &'a Self::Node) -> NodeInfo<'a, Self::Node> {
        NodeInfo::new(root, 0, 0)
    }

    fn stack_mut(&mut self) -> &mut Stack<IterState<'a, Self::Node>> {
        &mut self.node_stack
    }
}

impl<'a, I: Clone, C: CtxTrait> Iterator for NodeIter<'a, BaseNode<I, C>> {
    type Item = NodeInfo<'a, BaseNode<I, C>>;
    fn next(&mut self) -> Option<Self::Item> {
        self.get_state().and_then(|state| self.next_state(state))
    }
}

impl<I: Clone, C: CtxTrait> HasChildren for BaseNode<I, C> {
    fn num_of_children(&self) -> usize {
        self.children.len()
    }

    fn get_child(&self, n: usize) -> Option<&Self> {
        self.children.get(n)
    }
}
