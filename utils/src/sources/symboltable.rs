use super::{SymbolError, SymbolInfo, SymbolQuery, SymbolWriter};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod new {
    use super::*;
    pub struct SymbolTable<T> {
        scope: String,
        info: HashMap<u64, T>,
        name_to_id: HashMap<String, u64>,
        ref_name_to_value: HashMap<String, i64>,
        id: u64,
    }

    pub struct SymbolInfo<T> {
        /// Symbol Name
        pub name: String,
        /// Unique Symbol Id
        pub id: u64,
        /// Value, if any
        pub value: T,
    }

    impl<T> Default for SymbolTable<T> {
        fn default() -> Self {
            Self::new()
        }
    }

    impl<T> SymbolTable<T> {
        pub fn new() -> Self {
            Self::new_with_scope("No Scope")
        }

        pub fn get_scope_name(&self) -> &str {
            &self.scope
        }

        pub fn new_with_scope(name: &str) -> Self {
            Self {
                scope: name.to_string(),
                info: Default::default(),
                name_to_id: Default::default(),
                ref_name_to_value: Default::default(),
                id: 1,
            }
        }

        pub fn get(&self, id: u64) -> Result<&T, SymbolError> {
            self.info.get(&id).ok_or(SymbolError::NotFound)
        }

        pub fn symbol_exists(&self, name: &str) -> bool {
            self.name_to_id.get(name).is_some()
        }

        fn get_next_id(&mut self) -> u64 {
            let ret = self.id;
            self.id += 1;
            ret
        }

        fn get_id_from_name(&mut self, name: &str) -> Option<u64> {
            self.name_to_id.get(name).map(|x| *x)
        }

        fn get_mut(&mut self, id : u64) -> Option<&mut T> {
            self.info.get_mut(&id)
        }

        fn get_from_name_mut(&mut self, name : &str) -> Option<&mut T> {
            if let Some(id) = self.name_to_id.get(name).cloned() {
                self.get_mut(id)
            } else {
                None
            }
        }

        pub fn update_or_add_symbol(&mut self, name: &str, v : T) -> Result<u64,SymbolError> {
            if self.symbol_exists(name) {
                self.update_symbol(name, v)
            } else {
                self.add_symbol(name, v)
            }
        }

        pub fn update_symbol(&mut self, name : &str, v : T) -> Result<u64, SymbolError> {
            let id = self.get_id_from_name(name).ok_or(SymbolError::NotFound)?;
            let s = self.get_mut(id).unwrap();
            *s = v;
            Ok(id)
        }

        pub fn add_symbol(&mut self, name: &str, v : T) -> Result<u64, SymbolError> {
            if let Some(id) = self.get_id_from_name(name) {
                Err(SymbolError::AlreadyDefined(id))
            } else {
                let name = name.to_string();
                let id = self.get_next_id();
                self.name_to_id.insert(name.clone(), id);
                self.info.insert(id, v);
                Ok(id)
            }
        }
    }
}

/// Holds information about symbols
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SymbolTable {
    scope: String,
    pub info: HashMap<u64, SymbolInfo>,
    name_to_id: HashMap<String, u64>,
    ref_name_to_value: HashMap<String, i64>,
    id: u64,
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolQuery for SymbolTable {
    fn get_symbol_info(&self, name: &str) -> Result<&SymbolInfo, SymbolError> {
        let id = self.name_to_id.get(name).ok_or(SymbolError::NotFound)?;
        self.get(*id)
    }
}

impl SymbolWriter for SymbolTable {
    fn add_symbol_with_value(&mut self, name: &str, value: i64) -> Result<u64, SymbolError> {
        let nstr: String = name.into();
        let id = self.add_symbol(&nstr)?;
        self.set_value(id, value)?;

        if let Some(expected) = self.ref_name_to_value.get(&nstr) {
            if *expected != value {
                return Err(SymbolError::Mismatch {
                    expected: *expected,
                });
            }
        }

        Ok(id)
    }

    fn remove_symbol_name(&mut self, name: &str) {
        if let Ok(x) = self.get_symbol_info(name) {
            let id = x.id;
            self.name_to_id.remove(name);
            self.info.remove(&id);
        }
    }

    fn add_symbol(&mut self, name: &str) -> Result<u64, SymbolError> {
        let name: String = name.into();

        if let Ok(sym_info) = self.get_symbol_info(&name) {
            Err(SymbolError::AlreadyDefined(sym_info.id))
        } else {
            let id = self.get_next_id();

            self.name_to_id.insert(name.clone(), id);

            let info = SymbolInfo {
                name,
                id,
                value: None,
            };

            self.info.insert(id, info);
            Ok(id)
        }
    }

    fn add_reference_symbol(&mut self, name: &str, val: i64) {
        let res = self.ref_name_to_value.insert(name.to_string(), val);
        assert!(res.is_none());
    }
}

impl SymbolTable {
    pub fn new() -> Self {
        Self::new_with_scope("No Scope")
    }

    pub fn get_scope_name(&self) -> &str {
        &self.scope
    }

    pub fn new_with_scope(name: &str) -> Self {
        Self {
            scope: name.to_string(),
            info: Default::default(),
            name_to_id: Default::default(),
            ref_name_to_value: Default::default(),
            id: 1,
        }
    }

    fn get(&self, id: u64) -> Result<&SymbolInfo, SymbolError> {
        self.info.get(&id).ok_or(SymbolError::NotFound)
    }

    fn get_mut(&mut self, id: u64) -> Result<&mut SymbolInfo, SymbolError> {
        self.info.get_mut(&id).ok_or(SymbolError::NotFound)
    }

    fn symbol_exists(&self, id: u64) -> bool {
        self.get(id).is_ok()
    }

    fn set_value(&mut self, id: u64, value: i64) -> Result<(), SymbolError> {
        let i = self.get_mut(id)?;
        i.value = Some(value);
        Ok(())
    }

    fn get_next_id(&mut self) -> u64 {
        let ret = self.id;
        self.id += 1;
        ret
    }
}
