#![forbid(unused_imports)]
use crate::{ast::AstNodeId, item::Item};
use std::collections::HashMap;

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct FixKey {
    scope: u64,
    id: AstNodeId,
}

#[derive(Debug, Default)]
pub struct FixerUpper {
    pub fixups: HashMap<FixKey, Item>,
}

impl FixerUpper {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_fixup(&mut self, scope: u64, id: AstNodeId, v: Item) {
        let k = FixKey { id, scope };
        self.fixups.insert(k, v);
    }
    pub fn get_fixup(&self, scope: u64, id: AstNodeId) -> Option<&Item> {
        self.fixups.get(&FixKey { scope, id })
    }

    pub fn get_fixup_or_default(&self, scope: u64, id: AstNodeId, i: &Item) -> Item {
        self.get_fixup(scope, id).unwrap_or(i).clone()
    }
}
