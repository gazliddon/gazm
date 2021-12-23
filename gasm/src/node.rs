
////////////////////////////////////////////////////////////////////////////////
// Node

pub struct NodeIt<'a, I, CTX : Default > {
    index : usize,
    node : &'a BaseNode<I,CTX>
}

impl<'a, I, CTX : Default > NodeIt<'a, I, CTX > {
    pub fn new(node : &'a BaseNode<I,CTX>) -> Self {
        Self { index: 0, node }
    }
}

impl<'a, I, CTX: Default > Iterator for NodeIt<'a, I, CTX> {
    type Item = &'a BaseNode<I,CTX>;

    fn next(&mut self) -> Option<&'a BaseNode<I,CTX>> {
        if let Some(ret) = self.node.children.get(self.index) {
            self.index = self.index + 1;
            Some(ret)
        } else {
            None
        }
    }
}

#[derive(Debug,PartialEq, Clone)]
pub struct BaseNode<I,CTX : Default = Dummy> {
    item: I,
    pub children: Vec<Box<Self>>,
    ctx: CTX,
}

fn box_it<I>(v : Vec<I>) -> Vec<Box<I>> {
    v.into_iter().map(Box::new).collect()
}

impl<I, CTX : Default > BaseNode<I, CTX> {

    pub fn ctx(&self) -> &CTX {
        &self.ctx
    }
    pub fn item(&self) -> &I {
        &self.item
    }

    pub fn iter(&self) -> NodeIt<I,CTX> {
        NodeIt::new(self)
    }

    pub fn new(item : I, children: Vec<Self>, ctx : CTX) -> Self {
        let children = box_it(children);
        Self {item, children, ctx }
    }

    pub fn from_item(item: I) -> Self {
        Self::new(item, vec![], CTX::default())
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

    pub fn with_ctx(self, ctx : CTX) -> Self {
        let mut ret = self;
        ret.ctx = ctx;
        ret
    }

}

// impl<I : std::fmt::Debug , CTX: Default > std::fmt::Debug for BaseNode<I,CTX> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{:?}", self.item)
//     }
// }

#[derive(Debug, PartialEq, Clone)]
pub struct Dummy { }

impl Default for Dummy {
    fn default() -> Self {
        Self{}
    }
}



