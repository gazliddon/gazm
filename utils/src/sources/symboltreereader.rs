use super::{
    SymbolTree,SymbolTable,SymbolQuery, SymbolError,SymbolInfo,SymbolResolutionBarrier, SymbolScopeId
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
    pub fn get_current_symbols(&self) -> &SymbolTable {
        self.syms.get_symbols_from_id(self.current_scope).unwrap()
    }
}

impl<'a> SymbolQuery for SymbolTreeReader<'a> {
    fn get_symbol_info(&self, name: &str) -> Result<&SymbolInfo, SymbolError> {
        let scope = self.current_scope;
        self.syms
            .resolve_label(name, scope, SymbolResolutionBarrier::default())
    }

    fn get_symbol_info_from_id(&self, id: SymbolScopeId) -> Result<&SymbolInfo, SymbolError> {
        self.syms.get_symbol_info_from_id(id)
    }
}

