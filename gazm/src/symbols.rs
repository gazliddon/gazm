use emu::utils::symbols;


pub type ScopeId = u64;
pub type SymbolId = u64;
pub type SymValue = i64;
pub type SymbolInfo = symbols::SymbolInfo<ScopeId,SymbolId, SymValue>;
pub type SymbolTree = symbols::SymbolTree<ScopeId,SymbolId,SymValue>;
pub type SymbolScopeId = symbols::SymbolScopeId<ScopeId,SymbolId>;
pub type SymbolError = symbols::SymbolError<ScopeId,SymbolId>;
pub type SymbolTable = symbols::SymbolTable<ScopeId,SymbolId>;
pub type SymbolTreeWriter<'a> = symbols::SymbolTreeWriter<'a,ScopeId,SymbolId,SymValue>;
pub type SymbolTreeReader<'a> = symbols::SymbolTreeReader<'a,ScopeId,SymbolId,SymValue>;

