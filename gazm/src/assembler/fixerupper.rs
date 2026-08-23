#![forbid(unused_imports)]
use crate::{frontend::AstNodeKind, semantic::AstNodeId};

use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct FixKey {
    scope: u64,
    id: AstNodeId,
}

#[derive(Debug, Default)]
pub struct FixerUpper {
    pub fixups: HashMap<FixKey, Arc<AstNodeKind>>,
}

impl FixerUpper {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_fixup(&mut self, scope: u64, id: AstNodeId, v: AstNodeKind) {
        let k = FixKey { id, scope };
        self.fixups.insert(k, Arc::new(v));
    }
    pub fn get_fixup(&self, scope: u64, id: AstNodeId) -> Option<&AstNodeKind> {
        self.fixups.get(&FixKey { scope, id }).map(Arc::as_ref)
    }

    pub fn get_fixup_or_default(
        &self,
        scope: u64,
        id: AstNodeId,
        i: &AstNodeKind,
    ) -> Arc<AstNodeKind> {
        self.fixups
            .get(&FixKey { scope, id })
            .cloned()
            .unwrap_or_else(|| Arc::new(i.clone()))
    }
}
