use crate::ast::{ AstNodeId };
use crate::item::Item;
use std::collections::HashMap;

#[derive(Hash, PartialEq, Eq)]
pub struct FixKey {
    pc: usize,
    id: AstNodeId,
}
pub struct FixerUpper {
    fixups: HashMap<FixKey, Item>,
}

impl Default for FixerUpper {
    fn default() -> Self {
        Self {
            fixups: HashMap::new()
        }
    }

}

impl FixerUpper {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_fixup(&mut self, pc: usize, id: AstNodeId, v: Item) {
        let k = FixKey { id, pc };
        self.fixups.insert(k, v);
    }
    fn get_fixup(&self, pc: usize, id: AstNodeId) -> Option<&Item> {
        self.fixups.get(&FixKey { pc, id })
    }

    pub fn get_fixup_or_default(&self, pc: usize, id: AstNodeId, i : &Item) -> Item {
        self.get_fixup(pc, id).unwrap_or(i).clone()
    }
}
