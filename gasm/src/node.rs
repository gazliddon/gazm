use std::slice::Iter;

use crate::ctx::Ctx;


////////////////////////////////////////////////////////////////////////////////
// Node

pub trait CtxTrait : Default + Clone + std::fmt::Debug {
}

////////////////////////////////////////////////////////////////////////////////
// Traverse the AST

#[derive(PartialEq, Clone)]
pub struct NodeTreeIt<'a, I, C : CtxTrait> {
    parent : Option<&'a Self>,
    node : &'a BaseNode<I,C>,

    child_it : Option<Box<NodeTreeIt<'a, I, C>>>,

    first : bool,
    index : usize,
    depth : usize,
}

impl <'a, I, C : CtxTrait> NodeTreeIt<'a, I, C > { 
    pub fn new(node : &'a BaseNode<I,C>) -> Self {
        Self::new_with_depth(None,0, node)
    }

    pub fn new_with_depth(parent: Option<&'a Self>, depth: usize, node : &'a BaseNode<I,C>) -> Self {
        Self {
            node,
            index: 0,
            first : true,
            child_it : None,
            depth,
            parent
        }
    }

    pub fn parent(&self) -> Option<&'a Self> {
        self.parent
    }


    fn next_child_it(&mut self) -> Option<Box<Self>> {
        if self.index < self.node.children.len() {
            let ret = Self::new_with_depth(
                None, self.depth+1,
                &self.node.children[self.index]).into();

            self.index += 1;

            Some(ret)
        } else {
            None
        }
    }

    fn next_child(&mut self){
        self.child_it = self.next_child_it();
    }
}


impl<'a, I, C: CtxTrait > Iterator for NodeTreeIt<'a, I, C> {
    type Item = &'a BaseNode<I,C>;

    fn next(&mut self) -> Option<Self::Item> {

        if self.first {
            self.first = false;
            self.next_child();
            Some(self.node)
        } else {
            if let Some(it_box) = &mut self.child_it {

                if let Some(n) = it_box.as_mut().next() {
                    Some(n)
                } else {
                    self.next_child();
                    self.next()
                }
            } else {
                None
            }
        }
    }
}

pub struct NodeIt<'a, I, C : CtxTrait > {
    index : usize,
    node : &'a BaseNode<I,C>
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
    pub fn get_child(&self, n : usize) -> Option<&BaseNode<I,C>> {
        if let Some(box_node) = self.children.get(n) {
            Some(&*box_node)
        } else {
            None
        }
    }

    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    pub fn iter(&self) -> NodeIt<I,C> {
        NodeIt::new(self)
    }

    pub fn tree_iter(&self) -> NodeTreeIt<I,C> {
        NodeTreeIt::new(self)
    }

    pub fn new(item : I, children: Vec<Self>, ctx : C) -> Self {
        let children = box_it(children);
        Self {item, children, ctx }
    }

    pub fn from_item(item: I) -> Self {
        Self::new(item, vec![], C::default())
    }
    pub fn from_item_item(item: I, child: I) -> Self {
        Self::from_item(item).with_child(Self::from_item(child))
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
        let mut db = f.debug_struct("Node");
         db.field("item", &self.item);

         if self.has_children() {
             db.field("children", &self.children);
         }

         db.field("ctx",&self.ctx());

         db.finish()
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



