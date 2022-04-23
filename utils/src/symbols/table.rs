use super::*;
use std::collections::HashMap;

// #[derive(Debug, PartialEq, Hash, Eq, Clone, Copy)]
// pub struct SymbolId(u64);

// impl From<u64> for SymbolId {
//     fn from(v: u64) -> Self {
//         SymbolId(v)
//     }
// }

#[derive(Debug)]
struct ScopedSymbol<'a> {
    path: Vec<&'a str>,
    symbol: &'a str,
}

impl<'a> ScopedSymbol<'a> {
    pub fn new(fqn : &'a str) -> Self {
         let mut path : Vec<_>  = fqn.split("::").collect();
         assert!(path.len() > 0 );
         let symbol = path.last().unwrap().clone();
         path.resize(path.len() - 1, "");
         Self {
             symbol, path
         }
    }
    pub fn get_symbol(&self) -> &str {
        self.symbol
    }

    pub fn get_fqn(&self) -> String {
        if self.path.is_empty() {
            self.symbol.to_string()
        } else {
            format!("{}::{}", self.path.join("::"), self.symbol)
        }
    }
}

#[derive(Debug)]
pub struct SymbolTable<V : super::ValueTraits, ID : IdTraits> {
    scope: String,
    name_to_id: HashMap<String, ID>,
    id_to_name: HashMap<ID, String>,
    id_to_value: HashMap<ID, V>,
    id: usize,
}
impl<V : ValueTraits, ID : IdTraits> SymbolReader<V, ID> for SymbolTable<V, ID> { 
    fn get_symbol_id(&self, name: &str) -> Option<ID> {
        self.name_to_id.get(name).cloned()
    }

    fn get_symbol(&self, id: ID) -> Option<&V> {
        self.id_to_value.get(&id)
    }
}

impl<V : ValueTraits, ID : IdTraits> SymbolWriter<V, ID> for SymbolTable<V, ID> {

    fn add_symbol_with_value(&mut self, name: &str, value: V) -> Result<ID, SymbolError<ID>> {
        if let Some(id) = self.get_symbol_id(name) {
            Err(SymbolError::AlreadyDefined(id))
        } else {
            let id = self.get_next_id();
            self.name_to_id.insert(name.to_string(), id);
            self.id_to_value.insert(id, value);
            self.id_to_name.insert(id, name.to_string());
            Ok(id)
        }
    }

    fn remove_symbol(&mut self, id: ID) -> Result<(), SymbolError<ID>> {
        self.get_symbol(id).ok_or(SymbolError::NotFound)?;
        let name = self.id_to_name.get(&id).unwrap();
        self.name_to_id.remove(name);
        self.id_to_value.remove(&id);
        self.id_to_name.remove(&id);
        Ok(())
    }

    fn get_symbol_mut(&mut self, id: ID) -> Result<&mut V, SymbolError<ID>> {
        self.id_to_value.get_mut(&id).ok_or(SymbolError::NotFound)
    }
}

impl<V : ValueTraits,ID: IdTraits> Default for SymbolTable<V,ID> {
    fn default() -> Self {
        Self::new("no name")
    }
}

impl<V : ValueTraits,ID: IdTraits> SymbolTable<V,ID> {
    pub fn get_scope_name(&self) -> &str {
        &self.scope
    }

    pub fn new(name: &str) -> Self {
        Self {
            scope: name.to_string(),
            id: 1,
            name_to_id: Default::default(),
            id_to_name: Default::default(),
            id_to_value: Default::default(),
        }
    }

    fn get_next_id(&mut self) -> ID {
        let ret = self.id;
        self.id += 1;
        ret.into()
    }
}
