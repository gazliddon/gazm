use crate::ctx::Ctx;


////////////////////////////////////////////////////////////////////////////////
// Node

pub trait CtxTrait : Default {
}

pub struct NodeIt<'a, I, CTX : CtxTrait > {
    index : usize,
    node : &'a BaseNode<I,CTX>
}

impl<'a, I, C : CtxTrait > NodeIt<'a, I, C > {
    pub fn new(node : &'a BaseNode<I,C>) -> Self {
        Self { index: 0, node }
    }
}

impl<'a, I, C: CtxTrait > Iterator for NodeIt<'a, I, C> {
    type Item = &'a BaseNode<I,C>;

    fn next(&mut self) -> Option<&'a BaseNode<I,C>> {
        if let Some(ret) = self.node.children.get(self.index) {
            self.index += 1;
            Some(ret)
        } else {
            None
        }
    }
}

#[derive(PartialEq, Clone)]
pub struct BaseNode<I,C : CtxTrait = Dummy> {
    item: I,
    pub children: Vec<Box<Self>>,
    ctx: C,
}

fn box_it<I>(v : Vec<I>) -> Vec<Box<I>> {
    v.into_iter().map(Box::new).collect()
}

impl<I, C : CtxTrait > BaseNode<I, C> {

    pub fn ctx(&self) -> &C {
        &self.ctx
    }
    pub fn item(&self) -> &I {
        &self.item
    }

    pub fn iter(&self) -> NodeIt<I,C> {
        NodeIt::new(self)
    }

    pub fn new(item : I, children: Vec<Self>, ctx : C) -> Self {
        let children = box_it(children);
        Self {item, children, ctx }
    }

    pub fn from_item(item: I) -> Self {
        Self::new(item, vec![], C::default())
    }

    pub fn with_children(self, children : Vec<Self>) -> Self {
        let mut ret = self;
        ret.children = box_it(children);
        ret
    }

    pub fn with_child(self, child : Self) -> Self {
        let mut ret = self;
        ret.children = vec![child.into()];
        ret
    }

    pub fn with_ctx(self, ctx : C) -> Self {
        let mut ret = self;
        ret.ctx = ctx;
        ret
    }

}

impl<I : std::fmt::Debug , C: CtxTrait > std::fmt::Debug for BaseNode<I,C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.item)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Dummy { }

impl CtxTrait for Dummy {
}

impl Default for Dummy {
    fn default() -> Self {
        Self{}
    }
}



