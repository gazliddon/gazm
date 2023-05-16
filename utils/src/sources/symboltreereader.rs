use super::{
    SymbolError, SymbolInfo, SymbolResolutionBarrier, SymbolScopeId, SymbolTable, SymbolTree,
};

pub struct SymbolTreeReader<'a> {
    current_scope: u64,
    syms: &'a SymbolTree,
}

impl<'a> SymbolTreeReader<'a> {
    pub fn new(syms: &'a SymbolTree, current_scope: u64) -> Self {
        Self {
            syms,
            current_scope,
        }
    }

    pub fn syms(&self) -> &SymbolTree {
        self.syms
    }

    pub fn get_current_symbols_2(&self) -> &SymbolTable {
        self.syms.get_symbols_from_id(self.current_scope).unwrap()
    }

    pub fn get_symbol_info(&self, name: &str) -> Result<&SymbolInfo, SymbolError> {
        let scope = self.current_scope;
        let id = self
            .syms
            .resolve_label(name, scope, SymbolResolutionBarrier::default())?;

        self.get_symbol_info_from_id(id)
    }

    pub fn get_symbol_info_from_id(&self, id: SymbolScopeId) -> Result<&SymbolInfo, SymbolError> {
        self.syms.get_symbol_info_from_id(id)
    }
}
