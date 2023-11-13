// #![forbid(unused_imports)]
use super::TokenKind;

use crate::{
    ast::{Ast, AstTree, ItemWithPos},
    ast::{AstNodeId, AstNodeMut},
    item::{Item, Node},
};

struct AstNew {
    ast: AstTree,
}

impl AstNew {
    pub fn from_node(node: &Node) -> Self {
        let mut ast = AstTree::new(ItemWithPos::new(node));

        for c in &node.children {
            Self::add_node(&mut ast.root_mut(), c);
        }

        AstNew { ast }
    }

    fn add_node(parent: &mut AstNodeMut, node: &Node) {
        let ipos = ItemWithPos::new(node);

        let mut this_node = parent.append(ipos);

        for n in &node.children {
            Self::add_node(&mut this_node, n);
        }
    }
}

