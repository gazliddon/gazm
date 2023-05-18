use super::{SymbolError, SymbolScopeId, symboltree::SymbolTree,ScopeIdTraits, SymIdTraits};

////////////////////////////////////////////////////////////////////////////////
pub struct SymbolTreeWriter<'a, SCOPEID, SYMID,SYMVALUE>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
{
    current_scope_id: SCOPEID,
    sym_tree: &'a mut SymbolTree<SCOPEID, SYMID, SYMVALUE>,
}

impl<'a, SCOPEID, SYMID,V> SymbolTreeWriter<'a,SCOPEID,SYMID,V>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
{
    pub fn new(sym_tree: &'a mut SymbolTree<SCOPEID,SYMID,V>, current_scope_id: SCOPEID) -> Self {
        Self {
            current_scope_id,
            sym_tree,
        }
    }

    pub fn new_root(sym_tree: &'a mut SymbolTree<SCOPEID, SYMID,V>) -> Self {
        Self::new(sym_tree, sym_tree.get_root_scope_id())
    }

    pub fn pop(&mut self) {
        let n = self
            .sym_tree
            .get_node_from_id(self.current_scope_id)
            .expect("Invalid id");
        if let Some(n) = n.parent() {
            self.current_scope_id = n.value().get_scope_id()
        }
    }

    pub fn goto_root(&mut self) {
        self.current_scope_id = self.sym_tree.get_root_scope_id();
    }

    pub fn get_scope(&self) -> SCOPEID {
        self.current_scope_id
    }

    pub fn get_scope_fqn(&self) -> String {
        self.sym_tree.get_fqn_from_id(self.current_scope_id)
    }

    pub fn set_scope_from_id(&mut self, id: SCOPEID) -> Result<(), SymbolError<SCOPEID, SYMID>> {
        self.current_scope_id = id;
        Ok(())
    }

    // enters the child scope below the current_scope
    // If it doesn't exist then create it
    pub fn create_or_set_scope(&mut self, name: &str) -> SCOPEID {
        let new_scope_node_id = self
            .sym_tree
            .create_or_get_scope_for_parent(name, self.current_scope_id);
        self.current_scope_id = new_scope_node_id;
        new_scope_node_id
    }


    pub fn add_reference_symbol(
        &mut self,
        name: &str,
        id: SymbolScopeId<SCOPEID,SYMID>,
    ) -> Result<(), SymbolError<SCOPEID, SYMID>> {
        self.sym_tree
            .add_reference_symbol(name, self.current_scope_id, id)
    }

    pub fn create_and_set_symbol(
        &mut self,
        name: &str,
        val: V,
    ) -> Result<SymbolScopeId<SCOPEID,SYMID>, SymbolError<SCOPEID,SYMID>> {
        let symbol_id = self.create_symbol(name)?;
        self.sym_tree.set_symbol_from_id(symbol_id, val)?;
        Ok(symbol_id)
    }

    pub fn remove_symbol(&mut self, name: &str) -> Result<(), SymbolError<SCOPEID,SYMID>> {
        self.sym_tree
            .remove_symbol_for_id(name, self.current_scope_id)
    }

    pub fn create_symbol(&mut self, name: &str) -> Result<SymbolScopeId<SCOPEID,SYMID>, SymbolError<SCOPEID,SYMID>> {
        self.sym_tree
            .create_symbol_in_scope(self.current_scope_id, name)
    }
}
impl<'a, SCOPEID, SYMID,SYMVALUE> SymbolTreeWriter<'a,SCOPEID,SYMID,SYMVALUE>
where
    SCOPEID: ScopeIdTraits + std::fmt::Debug,
    SYMID: SymIdTraits + std::fmt::Debug, {
    pub fn dump_scope(&self) {
        let x = self
            .sym_tree
            .get_node_from_id(self.current_scope_id)
            .unwrap();
        println!("{:#?}", x.value())
    }

}

