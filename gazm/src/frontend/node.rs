#![forbid(unused_imports)]
use super::NodeIter;
use std::fmt::Debug;
use thin_vec::{thin_vec, ThinVec};

////////////////////////////////////////////////////////////////////////////////
// Node
pub trait CtxTrait: Clone + std::fmt::Debug {}

/// I: Item
#[derive(PartialEq, Clone)]
pub struct BaseNode<I: Clone, C: CtxTrait = Dummy> {
    pub item: I,
    pub ctx: C,
    pub children: ThinVec<Self>,
}

impl<ITEM, CTX> BaseNode<ITEM, CTX>
where
    ITEM: Clone,
    CTX: CtxTrait,
{
    pub fn get_child(&self, n: usize) -> Option<&BaseNode<ITEM, CTX>> {
        self.children.get(n)
    }

    pub fn new_with_children<X>(item: ITEM, children: &[Self], ctx: X) -> Self
    where
        X: Into<CTX>,
    {
        Self {
            item,
            children: children.iter().cloned().collect(),
            ctx: ctx.into(),
        }
    }

    pub fn new<X>(item: ITEM, ctx: X) -> Self
    where
        X: Into<CTX>,
    {
        Self {
            item,
            children: Default::default(),
            ctx: ctx.into(),
        }
    }

    pub fn take_others_children(self, other: Self) -> Self {
        self.with_children(&other.children)
    }

    pub fn with_children_vec<V>(self, children: V) -> Self
    where
        V: Into<ThinVec<Self>>,
    {
        Self {
            children: children.into(),
            ..self
        }
    }

    pub fn with_children(self, children: &[Self]) -> Self {
        Self {
            children: children.into(),
            ..self
        }
    }

    pub fn with_child(self, child: Self) -> Self {
        Self {
            children: thin_vec![child],
            ..self
        }
    }
    pub fn with_item(self, item: ITEM) -> Self {
        Self { item, ..self }
    }

    pub fn add_child(&mut self, n: Self) {
        self.children.push(n)
    }

    pub fn iter(&self) -> NodeIter<Self> {
        NodeIter::new(self)
    }
}

impl<ITEM, CTX> Debug for BaseNode<ITEM, CTX>
where
    ITEM: Clone + Debug,
    CTX: CtxTrait,
{
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
