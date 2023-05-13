/// Trait for navigating around a symbol tree

use super::{ SymbolTree, ScopedName, SymbolError };

pub enum NavError {
}

type NResult<T> = Result<T,NavError>;

trait SymbolNav {
    fn up(&mut self) -> NResult<()>;
    fn root(&mut self);
    fn cd(&mut self, dir: &str) -> NResult<u64>;
    fn get_id(&self) -> u64;
}

pub struct Naver<'a> {
    tree : &'a SymbolTree,
    current_scope : u64,
}

impl<'a> Naver<'a> {
    pub fn new(tree : &'a SymbolTree) -> Self {
        Self {
            tree,
            current_scope : tree.get_root_scope_id(),
        }
    }
}

impl SymbolTree {
    pub (crate) fn up(&self, scope_id : u64) -> Result<u64, SymbolError> {
        let n = self.get_node_from_id(scope_id)?;
        let node_id_to_find = n.parent().map(|n| n.id()).ok_or(SymbolError::NotFound)?;

        for (id, node_id) in self.scope_id_to_node_id.iter() {
            if node_id_to_find == *node_id  {
                return Ok(*id)
            }
        }
        panic!()
    }

    pub(crate) fn cd(&mut self, _current_scope : u64, dir: &str) -> Result<u64, SymbolError>{
        let _x = ScopedName::new(dir);
        todo!()
    }

}

impl<'a> SymbolNav for Naver<'a> {
    fn up(&mut self) -> NResult<()> {
        todo!()
    }

    fn root(&mut self) {
        self.current_scope = self.tree.get_root_scope_id();
    }

    fn cd(&mut self, dir: &str) -> NResult<u64> {
        let _x = ScopedName::new(dir);
        todo!()
    }

    fn get_id(&self) -> u64 {
        self.current_scope
    }
}


