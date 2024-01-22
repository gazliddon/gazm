#![forbid(unused_imports)]
use crate::{frontend::Item, semantic::AstNodeId};

use std::collections::HashMap;

use super::AssemblerCpuTrait;

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct FixKey {
    scope: u64,
    id: AstNodeId,
}

#[derive(Debug)]
pub struct FixerUpper<C>
where
    C: AssemblerCpuTrait,
{
    pub fixups: HashMap<FixKey, Item<C::NodeKind>>,
}

impl<C> Default for FixerUpper<C>
where
    C: AssemblerCpuTrait,
{
    fn default() -> Self {
        Self { fixups: Default::default() }
    }
}

impl<C> FixerUpper<C>
where
    C: AssemblerCpuTrait,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_fixup(&mut self, scope: u64, id: AstNodeId, v: Item<C::NodeKind>) {
        let k = FixKey { id, scope };
        self.fixups.insert(k, v);
    }
    pub fn get_fixup(&self, scope: u64, id: AstNodeId) -> Option<&Item<C::NodeKind>> {
        self.fixups.get(&FixKey { scope, id })
    }

    pub fn get_fixup_or_default(
        &self,
        scope: u64,
        id: AstNodeId,
        i: &Item<C::NodeKind>,
    ) -> Item<C::NodeKind> {
        self.get_fixup(scope, id).unwrap_or(i).clone()
    }
}
