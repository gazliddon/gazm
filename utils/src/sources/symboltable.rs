use super::{SymbolError, SymbolInfo, SymbolScopeId, SymbolWriter};

use std::{collections::HashMap, fmt::Display};

////////////////////////////////////////////////////////////////////////////////
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default, Copy, PartialOrd)]
pub enum SymbolResolutionBarrier {
    Local = 0,
    Module = 1,
    #[default]
    Global = 2,
}

impl SymbolResolutionBarrier {
    pub fn can_pass_barrier(&self, i: SymbolResolutionBarrier) -> bool {
        i >= *self
    }
}

/// Holds information about symbols
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
pub struct SymbolTable {
    scope: String,
    fqn_scope: String,
    name_to_id: HashMap<String, u64>,
    ref_name_to_value: HashMap<String, SymbolScopeId>,
    highest_id: u64,
    scope_id: u64,
    symbol_resolution_barrier: SymbolResolutionBarrier,
}

pub enum SymbolKind {
    Undefined,
    Number,
    MacroDefinition { node: u64 },
}

impl Display for SymbolTable {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        panic!()
        // writeln!(f, "Scope: {}", self.scope)?;

        // for (name, id) in &self.name_to_id {
        //     let val = self.get(*id).unwrap();
        //     match &val.value {
        //         Some(val) => writeln!(f, "{name} = {val:04X} ({val})")?,
        //         _ => writeln!(f, "{name} = undefined",)?,
        //     }
        // }
        // Ok(())
    }
}

impl SymbolWriter for SymbolTable {
    fn create_and_set_symbol(
        &mut self,
        _name: &str,
        _value: i64,
    ) -> Result<SymbolScopeId, SymbolError> {
        // let nstr: String = name.into();
        // let id = self.create_symbol(&nstr)?;
        // self.set_value(id.symbol_id, value)?;
        // Ok(id)
        panic!()
    }

    fn remove_symbol(&mut self, name: &str) -> Result<(),SymbolError>{
        if let Some(_id) = self.name_to_id.get(name).cloned() {
            self.name_to_id.remove(name);
            Ok(())
        } else {
            Err(SymbolError::NotFound)
        }
    }

    fn create_symbol(&mut self, name: &str) -> Result<SymbolScopeId, SymbolError> {
        let name: String = name.into();

        if let Some(id) = self.name_to_id.get(&name) {
            Err(SymbolError::AlreadyDefined((*id, None)))
        } else {
            let new_symbol_id = self.get_next_id();

            self.name_to_id.insert(name.clone(), new_symbol_id);
            let id = SymbolScopeId {
                symbol_id: new_symbol_id,
                scope_id: self.scope_id,
            };

            Ok(id)
        }
    }

    fn add_reference_symbol(&mut self, name: &str, symbol_id: SymbolScopeId) {
        if self.get_symbol_id(name).is_ok() {
            panic!("Can't add reference {name}: Symbol already exists in this scope!")
        }
        self.ref_name_to_value.insert(name.to_string(), symbol_id);
    }
}

impl SymbolTable {
    pub fn get_symbol_id(&self, name: &str) -> Result<SymbolScopeId,SymbolError>  {
        // Is this a ref id?
        if let Some(id) = self.ref_name_to_value.get(name) {
            Ok(*id)
        } else {
            let symbol_id = self.name_to_id.get(name).ok_or(SymbolError::NotFound).cloned()?;
            let scope_id = self.scope_id;
            Ok(SymbolScopeId { scope_id, symbol_id })
        }
    }

    // pub fn get_symbol_info(&self, name: &str) -> Result<&SymbolInfo, SymbolError> {
    //     let id = self.get_symbol_id(name)?;
    //     self.get_symbol_info_from_id(id.symbol_id)
    // }

    // pub (crate) fn get_symbol_info_from_id(&self, _id: u64) -> Result<&SymbolInfo,SymbolError>{
    //     self.get(_id)
    // }

    // pub fn set_symbol(&mut self, id: u64, val: i64) -> Result<(), SymbolError> {
    //         self.set_value(id, val)
    // }

    pub fn get_symbol_resoultion_barrier(&self) -> SymbolResolutionBarrier {
        self.symbol_resolution_barrier
    }

    pub fn get_scope_name(&self) -> &str {
        &self.scope
    }
    pub fn get_scope_fqn_name(&self) -> &str {
        &self.fqn_scope
    }


    pub fn new(
        name: &str,
        fqn_scope: &str,
        scope_id: u64,
        symbol_resolution_barrier: SymbolResolutionBarrier,
    ) -> Self {
        Self {
            scope: name.to_string(),
            highest_id: 1,
            fqn_scope: fqn_scope.to_string(),
            scope_id,
            symbol_resolution_barrier,
            ..Default::default()
        }
    }

    pub fn get_symbols(&self) -> Vec<&SymbolInfo> {
        panic!()
    }

    pub fn get_scope_id(&self) -> u64 {
        self.scope_id
    }

}

impl SymbolTable {
    // fn get(&self, id: u64) -> Result<SymbolScopeId, SymbolError> {
    //     self.info.get(&id).ok_or(SymbolError::NotFound)?;
    //     Ok(SymbolScopeId { scope_id: self.scope_id, symbol_id: u64 })

    // }

    fn  get_ref_from_name(&self, name: &str) -> Option<SymbolScopeId> {
        self.ref_name_to_value.get(name).cloned()
    }

    // pub (crate)fn get_symbol_info_hash(&self) -> &HashMap<u64, SymbolInfo> {
    //     &self.info
    // }

    // fn get_mut(&mut self, id: u64) -> Result<&mut SymbolInfo, SymbolError> {
    //     self.info.get_mut(&id).ok_or(SymbolError::NotFound)
    // }

    // fn set_value(&mut self, id: u64, value: i64) -> Result<(), SymbolError> {
    //     let i = self.get_mut(id)?;
    //     i.value = Some(value);
    //     Ok(())
    // }

    fn get_next_id(&mut self) -> u64 {
        let ret = self.highest_id;
        self.highest_id += 1;
        ret
    }

}
