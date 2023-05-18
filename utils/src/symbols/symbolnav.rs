/// Trait for navigating around a symbol tree
use super::{ScopedName, SymbolError, SymbolTree, ScopeIdTraits, SymIdTraits};

pub enum NavError {}

type NResult<T> = Result<T, NavError>;

trait SymbolNav<SCOPEID>
where
    SCOPEID: std::hash::Hash + std::ops::AddAssign<u64> + std::clone::Clone,
{
    fn up(&mut self) -> NResult<()>;
    fn root(&mut self);
    fn cd(&mut self, dir: &str) -> NResult<SCOPEID>;
    fn get_id(&self) -> SCOPEID;
}

pub struct Naver<'a, SCOPEID, SYMID, SYMVALUE>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
{
    tree: &'a SymbolTree<SCOPEID, SYMID, SYMVALUE>,
    current_scope: SCOPEID,
}

impl<'a, SCOPEID, SYMID, SYMVALUE> Naver<'a, SCOPEID, SYMID,SYMVALUE>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
{
    pub fn new(tree: &'a SymbolTree<SCOPEID, SYMID, SYMVALUE>) -> Self {
        Self {
            tree,
            current_scope: tree.get_root_scope_id(),
        }
    }
}

impl<SCOPEID, SYMID, V> SymbolTree<SCOPEID, SYMID, V>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
{
    pub(crate) fn up(&self, scope_id: SCOPEID) -> Result<SCOPEID, SymbolError<SCOPEID,SYMID>> {
        let n = self.get_node_from_id(scope_id)?;
        let node_id_to_find = n.parent().map(|n| n.id()).ok_or(SymbolError::NotFound)?;

        for (id, node_id) in self.scope_id_to_node_id.iter() {
            if node_id_to_find == *node_id {
                return Ok(*id);
            }
        }
        panic!()
    }

    pub(crate) fn cd(
        &mut self,
        _current_scope: SCOPEID,
        dir: &str,
    ) -> Result<u64, SymbolError<SCOPEID,SYMID, >> {
        let _x = ScopedName::new(dir);
        todo!()
    }
}

impl<'a, SCOPEID, SYMID, SYMVALUE> SymbolNav<SCOPEID> for Naver<'a, SCOPEID, SYMID, SYMVALUE>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
{
    fn up(&mut self) -> NResult<()> {
        todo!()
    }

    fn root(&mut self) {
        self.current_scope = self.tree.get_root_scope_id();
    }

    fn cd(&mut self, dir: &str) -> NResult<SCOPEID> {
        let _x = ScopedName::new(dir);
        todo!()
    }

    fn get_id(&self) -> SCOPEID {
        self.current_scope
    }
}
