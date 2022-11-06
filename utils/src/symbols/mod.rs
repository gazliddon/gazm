mod table;
mod tree;
mod paths;

pub use table::*;
pub use tree::*;
pub use paths::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SymbolErrorKind<ID> {
    AlreadyDefined(ID),
    Mismatch { expected: i64 },
    NotFound,
    AbsPathNeeded,
}

pub type SymbolResult<ID,T> = Result<T, SymbolErrorKind<ID>>;

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
    fn add_symbol(&mut self, name: &str) -> SymbolResult<ID,ID> {
        self.add_symbol_with_value(name, V::default())
    }

    fn remove_symbol_name(&mut self, name: &str) -> SymbolResult<ID, ()> {
        let id = self.get_symbol_id(name).ok_or(SymbolErrorKind::NotFound)?;
        self.remove_symbol(id)
    }

    fn add_symbol_with_value(&mut self, name: &str, value: V) -> SymbolResult<ID,  ID>;
    fn remove_symbol(&mut self, id: ID) -> SymbolResult<ID, ()>;
    fn get_symbol_mut(&mut self, id: ID) -> SymbolResult<ID, &mut V>;
}

