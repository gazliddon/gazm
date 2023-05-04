use super::{
    SymbolError, SymbolInfo, SymbolQuery, SymbolResolutionBarrier, SymbolScopeId, SymbolTable,
    SymbolTree, SymbolWriter,
};

////////////////////////////////////////////////////////////////////////////////
pub struct SymbolNav<'a> {
    current_scope_id: u64,
    sym_tree: &'a mut SymbolTree,
}

impl<'a> SymbolNav<'a> {
    pub fn new(sym_tree: &'a mut SymbolTree, current_scope_id: u64) -> Self {
        let _ = sym_tree
            .get_node_id_from_scope_id(current_scope_id)
            .expect("Invalid ID");
        Self {
            current_scope_id,
            sym_tree,
        }
    }

    pub fn get_tree(&self) -> &SymbolTree {
        &self.sym_tree
    }

    pub fn pop_scope(&mut self) {
        let n = self
            .sym_tree
            .get_node_from_id(self.current_scope_id)
            .expect("Invalid id");
        if let Some(n) = n.parent() {
            self.current_scope_id = n.value().get_scope_id()
        }
    }

    pub fn set_root(&mut self) {
        self.current_scope_id = self.sym_tree.tree.root().value().get_scope_id()
    }

    pub fn get_current_scope(&self) -> u64 {
        self.current_scope_id
    }

    pub fn get_current_scope_symbols(&self) -> &SymbolTable {
        self.sym_tree
            .get_symbols_from_id(self.current_scope_id)
            .unwrap()
    }

    pub fn get_current_scope_fqn(&self) -> String {
        self.sym_tree.get_fqn_from_id(self.current_scope_id)
    }

    pub fn set_current_scope_from_id(&mut self, id: u64) -> Result<(), SymbolError> {
        self.sym_tree.get_node_mut_from_id(id)?;
        self.current_scope_id = id;
        Ok(())
    }

    pub fn get_current_scope_id(&self) -> u64 {
        self.current_scope_id
    }

    // enters the child scope below the current_scope
    // If it doesn't exist then create it
    pub fn set_current_scope(&mut self, name: &str) -> u64 {
        let new_scope_node_id = self.create_or_get_scope_id(name);
        self.current_scope_id = new_scope_node_id;
        new_scope_node_id
    }

    pub fn create_or_get_scope_id(&mut self, name: &str) -> u64 {
        self.sym_tree
            .create_or_get_scope_for_parent(name, self.current_scope_id)
    }
}

impl<'a> SymbolQuery for SymbolNav<'a> {
    fn get_symbol_info_from_id(&self, id: SymbolScopeId) -> Result<&SymbolInfo, SymbolError> {
        let node = self.sym_tree.get_node_from_id(id.scope_id)?;
        node.value().get_symbol_info_from_id(id)
    }

    fn get_symbol_info(&self, name: &str) -> Result<&SymbolInfo, SymbolError> {
        let scope = self.sym_tree.get_current_scope_id();
        self.sym_tree
            .resolve_label(name, scope, SymbolResolutionBarrier::default())
    }
}

impl<'a> SymbolWriter for SymbolNav<'a> {
    fn set_symbol(&mut self, symbol_id: SymbolScopeId, val: i64) -> Result<(), SymbolError> {
        let mut x = self.sym_tree.get_node_mut_from_id(symbol_id.scope_id)?;
        x.value().set_symbol(symbol_id, val)
    }

    fn add_symbol_with_value(
        &mut self,
        name: &str,
        value: i64,
    ) -> Result<SymbolScopeId, SymbolError> {
        let mut node = self.sym_tree.get_node_mut_from_id(self.current_scope_id)?;
        node.value().add_symbol_with_value(name, value)
    }

    fn remove_symbol_name(&mut self, name: &str) {
        let mut node = self
            .sym_tree
            .get_node_mut_from_id(self.current_scope_id)
            .expect("invalide node");
        node.value().remove_symbol_name(name)
    }

    fn add_symbol(&mut self, name: &str) -> Result<SymbolScopeId, SymbolError> {
        let mut node = self.sym_tree.get_node_mut_from_id(self.current_scope_id)?;
        node.value().add_symbol(name)
    }

    fn add_reference_symbol(&mut self, name: &str, val: i64) {
        let mut node = self
            .sym_tree
            .get_node_mut_from_id(self.current_scope_id)
            .unwrap();
        node.value().add_reference_symbol(name, val)
    }
}
