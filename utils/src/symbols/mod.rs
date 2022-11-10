mod table;
mod tree;
mod paths;
mod cursor;

pub use table::*;
pub use tree::*;
pub use paths::*;
pub use cursor::*;


#[derive(thiserror::Error,Debug, PartialEq, Eq, Clone)]
pub enum ScopeErrorKind {
    #[error("Scope not found: {0}")]
    ScopeNotFound(String),
    #[error("Invalid scope id")]
    InvalidScopeId,
    #[error("Needed an absolute scope path")]
    AbsPathNeeded,
    #[error("Needed a relative scope path")]
    RelPathNeeded,
    #[error("Scope path {0} already exists")]
    PathAlreadyExists(String),
    #[error("No parent for this scope")]
    NoParent,
    #[error("Symbol already defined: {0}")]
    SymbolAlreadyDefined(String),
    #[error("Mismatch")]
    Mismatch { expected: i64 },
    #[error("Symbol id not found")]
    SymbolIdNotFound,
}

pub type ScopeResult<T> = Result<T, ScopeErrorKind>;


use std::hash::Hash;
pub trait IdTraits : From<usize> + Clone + PartialEq + Eq + Hash + Copy {
}

pub trait ValueTraits : Default + Clone { }

pub trait SymbolReader<V : ValueTraits, ID : IdTraits> {
    fn get_symbol_from_name(&self, name: &str) -> Option<&V> {
        self.get_symbol_id(name).and_then(|id| self.get_symbol(id))
    }

    fn symbol_exists_from_name(&self, name: &str) -> bool {
        self.get_symbol_from_name(name).is_some()
    }

    fn get_symbol_name(&self, _id : &ID) -> Option<&str> {
        panic!()
    }

    fn get_symbol_id(&self, name: &str) -> Option<ID>;
    fn get_symbol(&self, id: ID) -> Option<&V>;
}


pub trait SymbolWriter<V : ValueTraits, ID : IdTraits> : SymbolReader<V, ID> {
    fn add_symbol(&mut self, name: &str) -> ScopeResult<ID> {
        self.add_symbol_with_value(name, V::default())
    }

    fn remove_symbol_name(&mut self, name: &str) -> ScopeResult< ()> {
        let id = self.get_symbol_id(name).ok_or(ScopeErrorKind::SymbolIdNotFound)?;
        self.remove_symbol(id)
    }

    fn add_symbol_with_value(&mut self, name: &str, value: V) -> ScopeResult<  ID>;
    fn remove_symbol(&mut self, id: ID) -> ScopeResult< ()>;
    fn get_symbol_mut(&mut self, id: ID) -> ScopeResult< &mut V>;
}

