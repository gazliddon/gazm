// symtab
use std::collections::HashMap;
// use serde_json::json;
use serde::{Deserialize, Serialize};

pub type SymbolId = usize;
/// Holds information about a symbol
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct SymbolInfo {
    /// Symbol Name
    pub name: String,
    /// Unique Symbol Id
    pub id: SymbolId,
    /// Value, if any
    pub value: Option<i64>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum SymbolError {
    AlreadyDefined(String),
    Mismatch{ expected: i64},
    NotFound,
    NoValue,
}

/// Holds information about symbols
#[derive(Serialize, Deserialize,Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SymbolTable {
    info: HashMap<SymbolId,SymbolInfo>,
    name_to_id: HashMap<String, SymbolId>,
    ref_name_to_value: HashMap<String,i64>,
    id: usize,
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            info: Default::default(),
            name_to_id: Default::default(),
            ref_name_to_value : Default::default(),
            id: 1
        }
    }

    pub fn add_reference_symbol(&mut self, name: &str, val : i64) {
        let res = self.ref_name_to_value.insert(name.to_string(), val);
        assert!(res.is_none());
    }

    pub fn get_value<S>(&self, name: S) -> Result<i64, SymbolError>
    where
        S: Into<String>,
    {
        let info = self.get_from_name(&name.into())?;
        info.value.ok_or(SymbolError::NoValue)
    }

    fn get(&self, id: SymbolId) -> Result<&SymbolInfo, SymbolError> {
        self.info.get(&id).ok_or(SymbolError::NotFound)
    }

    fn get_mut(&mut self, id: SymbolId) -> Result<&mut SymbolInfo, SymbolError> {
        self.info.get_mut(&id).ok_or(SymbolError::NotFound)
    }

    pub fn get_from_name<S>(&self, name: S) -> Result<&SymbolInfo, SymbolError>
    where
        S: Into<String>,
    {
        let id = self
            .name_to_id
            .get(&name.into())
            .ok_or(SymbolError::NotFound)?;
        self.get(*id)
    }

    pub fn symbol_exists(&self, id: SymbolId) -> bool {
        self.get(id).is_ok()
    }

    pub fn symbol_exists_from_name(&self, name: &str) -> bool {
        self.get_from_name(name).is_ok()
    }

    fn set_value(&mut self, id: SymbolId, value: i64) -> Result<(), SymbolError> {
        let i = self.get_mut(id)?;
        i.value = Some(value);
        Ok(())
    }

    pub fn add_symbol_with_value<S>(
        &mut self,
        name: S,
        value: i64,
    ) -> Result<SymbolId, SymbolError>
    where
        S: Into<String>,
    {
        let nstr : String = name.into();
        let id = self.add_symbol(&nstr)?;
        self.set_value(id, value)?;

        if let Some(expected) = self.ref_name_to_value.get(&nstr) {
            if *expected != value {
                return Err(SymbolError::Mismatch{expected: *expected})
            }
        }

        Ok(id)
    }
    fn get_next_id(&mut self) -> SymbolId {
        let ret = self.id;
        self.id += 1;
        ret
    }

    pub fn remove_symbol_name(&mut self, name: &str) 
    {
        if let Ok(x) = self.get_from_name(name) {
            let id = x.id;
            self.name_to_id.remove(name);
            self.info.remove(&id);
        }
    }

    pub fn add_symbol<S>(&mut self, name: S ) -> Result<SymbolId, SymbolError>
    where
        S: Into<String>,
    {
        let name: String = name.into();

        if let Ok(sym_info) = self.get_from_name(&name) {
            Err(SymbolError::AlreadyDefined(sym_info.name.clone()))
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
}
