use emu::utils::symbols;

pub type ScopeId = u64;
pub type SymbolId = u64;
pub type SymValue = i64;

pub type SymbolInfo = symbols::prelude::SymbolInfo<ScopeId,SymbolId, SymValue>;
pub type SymbolScopeId = symbols::prelude::SymbolScopeId<ScopeId,SymbolId>;
pub type SymbolError = symbols::prelude::SymbolError;

pub type SymbolTreeWriter<'a> = symbols::symboltreewriter::SymbolTreeWriter<'a,ScopeId,SymbolId,SymValue>;
pub type SymbolTreeReader<'a> = symbols::symboltreereader::SymbolTreeReader<'a,ScopeId,SymbolId,SymValue>;
pub type SymbolTree = symbols::SymbolTree<ScopeId,SymbolId,SymValue>;

pub use symbols::prelude::{ SymbolResolutionBarrier, ScopedName };
