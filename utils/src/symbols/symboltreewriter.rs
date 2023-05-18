use super::{
    SymbolError, SymbolScopeId, 
    SymbolTree, 
};

////////////////////////////////////////////////////////////////////////////////
pub struct SymbolTreeWriter<'a> {
    current_scope_id: u64,
    sym_tree: &'a mut SymbolTree,
}

impl<'a> SymbolTreeWriter<'a> {
    pub fn new(sym_tree: &'a mut SymbolTree, current_scope_id: u64) -> Self {
        Self {
            current_scope_id,
            sym_tree,
        }
    }

    pub fn new_root(sym_tree: &'a mut SymbolTree) -> Self {
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

    pub fn get_scope(&self) -> u64 {
        self.current_scope_id
    }

    pub fn get_scope_fqn(&self) -> String {
        self.sym_tree.get_fqn_from_id(self.current_scope_id)
    }

    pub fn set_scope_from_id(&mut self, id: u64) -> Result<(), SymbolError> {
        self.current_scope_id = id;
        Ok(())
    }

    // enters the child scope below the current_scope
    // If it doesn't exist then create it
    pub fn create_or_set_scope(&mut self, name: &str) -> u64 {
        let new_scope_node_id = self.sym_tree
            .create_or_get_scope_for_parent(name, self.current_scope_id);
        self.current_scope_id = new_scope_node_id;
        new_scope_node_id
    }

    pub fn dump_scope(&self) {
        let x = self.sym_tree.get_node_from_id(self.current_scope_id).unwrap();
        println!("{:#?}", x.value())
    }

    pub fn add_reference_symbol(&mut self, name: &str, id: SymbolScopeId) -> Result<(),SymbolError> {
        self.sym_tree.add_reference_symbol(name, self.current_scope_id,id)
    }

    pub fn create_and_set_symbol(
        &mut self,
        name: &str,
        val: i64,
    ) -> Result<SymbolScopeId, SymbolError> {
        let symbol_id = self.create_symbol(name)?;
        self.sym_tree.set_symbol_from_id(symbol_id, val)?;
        Ok(symbol_id)
    }

    pub fn remove_symbol(&mut self, name: &str) -> Result<(), SymbolError>{
        self.sym_tree.remove_symbol_for_id(name, self.current_scope_id)
    }

    pub fn create_symbol(&mut self, name: &str) -> Result<SymbolScopeId, SymbolError> {
        self.sym_tree.create_symbol_in_scope(self.current_scope_id, name)
    }
}
