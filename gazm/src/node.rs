////////////////////////////////////////////////////////////////////////////////
// Node
pub trait CtxTrait: Clone + std::fmt::Debug {}
use thin_vec::{thin_vec, ThinVec};

#[derive(PartialEq, Clone)]
pub struct BaseNode<I: Clone, C: CtxTrait = Dummy> {
    pub item: I,
    pub ctx: C,
    pub children: ThinVec<Self>,
}

impl<I: Clone, C: CtxTrait> BaseNode<I, C> {
    pub fn get_child(&self, n: usize) -> Option<&BaseNode<I, C>> {
        self.children.get(n)
    }

    pub fn new_with_children<X>(item: I, children: &[Self], ctx: X) -> Self
    where
        X: Into<C>,
    {
        Self {
            item,
            children: children.iter().cloned().collect(),
            ctx: ctx.into(),
        }
    }

    pub fn new<X>(item: I, ctx: X) -> Self
    where
        X: Into<C>,
    {
        Self {
            item,
            children: Default::default(),
            ctx: ctx.into(),
        }
    }

    pub fn take_others_children(self, other: Self) -> Self {
        let mut ret = self;
        ret.children = other.children;
        ret
    }

    pub fn with_child(self, child: Self) -> Self {
        let mut ret = self;
        ret.children = thin_vec![child];
        ret
    }

    pub fn add_child(&mut self, n: Self) {
        self.children.push(n)
    }
}

impl<I: std::fmt::Debug + Clone, C: CtxTrait> std::fmt::Debug for BaseNode<I, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut db = f.debug_struct("Node");
        db.field("item", &self.item);
        if !self.children.is_empty() {
            db.field("children", &self.children);
        }
        db.field("ctx", &self.ctx);
        db.finish()
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Dummy {}
impl CtxTrait for Dummy {}
