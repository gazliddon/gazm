////////////////////////////////////////////////////////////////////////////////
// Node
pub trait CtxTrait: Clone + std::fmt::Debug {}

#[derive(PartialEq, Clone)]
pub struct BaseNode<I, C: CtxTrait = Dummy> {
    pub item: I,
    pub children: Vec<Box<Self>>,
    pub ctx: C,
}

fn box_it<I>(v: Vec<I>) -> Vec<Box<I>> {
    v.into_iter().map(Box::new).collect()
}

impl<I, C: CtxTrait> BaseNode<I, C> {
    pub fn ctx(&self) -> &C {
        &self.ctx
    }

    pub fn item(&self) -> &I {
        &self.item
    }
    pub fn get_child(&self, n: usize) -> Option<&BaseNode<I, C>> {
        if let Some(box_node) = self.children.get(n) {
            Some(box_node)
        } else {
            None
        }
    }

    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    pub fn new_with_children<X>(item: I, children: Vec<Self>, ctx: X) -> Self
    where
        X: Into<C>,
    {
        let children = box_it(children);
        Self {
            item,
            children,
            ctx: ctx.into(),
        }
    }

    pub fn new<X>(item: I, ctx: X) -> Self
    where
        X: Into<C>,
    {
        Self {
            item,
            children : Default::default(),
            ctx: ctx.into(),
        }
    }

    pub fn from_item<X>(item: I, ctx: X) -> Self
    where
        X: Into<C>,
    {
        Self::new(item, ctx)
    }

    pub fn take_children(self, other: Self) -> Self {
        let mut ret = self;
        ret.children = other.children;
        ret
    }

    pub fn with_child(self, child: Self) -> Self {
        let mut ret = self;
        ret.children = vec![child.into()];
        ret
    }
    pub fn add_child(&mut self, n: Self) {
        self.children.push(Box::new(n))
    }

    pub fn with_ctx<X>(self, ctx: X) -> Self
    where
        X: Into<C>,
    {
        let mut ret = self;
        ret.ctx = ctx.into();
        ret
    }
}

impl<I: std::fmt::Debug, C: CtxTrait> std::fmt::Debug for BaseNode<I, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut db = f.debug_struct("Node");
        db.field("item", &self.item);

        if self.has_children() {
            db.field("children", &self.children);
        }

        db.field("ctx", &self.ctx());

        db.finish()
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Dummy {}
impl CtxTrait for Dummy {}

