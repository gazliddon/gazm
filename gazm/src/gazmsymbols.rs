use grl_sources::grl_symbols;
pub type ScopeId = u64;
pub type SymbolId = u64;
pub type SymValue = i64;

pub type SymbolInfo = grl_symbols::prelude::SymbolInfo<ScopeId,SymbolId, SymValue>;
pub type SymbolScopeId = grl_symbols::prelude::SymbolScopeId<ScopeId,SymbolId>;
pub type SymbolError = grl_symbols::prelude::SymbolError;

pub type SymbolTreeWriter<'a> = grl_symbols::symboltreewriter::SymbolTreeWriter<'a,ScopeId,SymbolId,SymValue>;
pub type SymbolTreeReader<'a> = grl_symbols::symboltreereader::SymbolTreeReader<'a,ScopeId,SymbolId,SymValue>;
pub type SymbolTree = grl_symbols::SymbolTree<ScopeId,SymbolId,SymValue>;

pub type Serializable = grl_symbols::serialize::Seriablizable<ScopeId,SymbolId,SymValue>;

pub use grl_symbols::prelude::{ SymbolResolutionBarrier, ScopedName };
