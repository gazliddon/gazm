#![forbid(unused_imports)]
use crate::{frontend::AstNodeKind, semantic::AstNodeId};

use std::collections::HashMap;


#[derive(Debug, Hash, PartialEq, Eq)]
pub struct FixKey {
    scope: u64,
    id: AstNodeId,
}

#[derive(Debug, Default)]
pub struct FixerUpper
{
    pub fixups: HashMap<FixKey, AstNodeKind>,
}

impl FixerUpper
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_fixup(&mut self, scope: u64, id: AstNodeId, v: AstNodeKind) {
        let k = FixKey { id, scope };
        self.fixups.insert(k, v);
    }
    pub fn get_fixup(&self, scope: u64, id: AstNodeId) -> Option<&AstNodeKind> {
        self.fixups.get(&FixKey { scope, id })
    }

    pub fn get_fixup_or_default(
        &self,
        scope: u64,
        id: AstNodeId,
        i: &AstNodeKind,
    ) -> AstNodeKind {
        self.get_fixup(scope, id).unwrap_or(i).clone()
    }
}
