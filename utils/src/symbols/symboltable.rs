use super::{ ScopeIdTraits, SymbolError,  SymbolScopeId, SymIdTraits};

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

impl<SCOPEID, SYMID> Default for SymbolTable<SCOPEID, SYMID> 
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
{
    fn default() -> Self {
        panic!()
    }
}

/// Holds information about symbols
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct SymbolTable<SCOPEID, SYMID>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
{
    scope: String,
    fqn_scope: String,
    name_to_id: HashMap<String, SYMID>,
    ref_name_to_symbol_id: HashMap<String, SymbolScopeId<SCOPEID, SYMID>>,
    highest_id: SYMID,
    scope_id: SCOPEID,
    symbol_resolution_barrier: SymbolResolutionBarrier,
}

impl<SCOPEID, SYMID> Display for SymbolTable<SCOPEID, SYMID>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
{
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

impl<SCOPEID, SYMID> SymbolTable<SCOPEID, SYMID>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
{
    pub(crate) fn get_symbol_id(
        &self,
        name: &str,
    ) -> Result<SymbolScopeId<SCOPEID, SYMID>, SymbolError<SCOPEID,SYMID>> {
        // Is this a ref id?
        if let Some(id) = self.ref_name_to_symbol_id.get(name) {
            Ok(*id)
        } else {
            let symbol_id = self
                .name_to_id
                .get(name)
                .ok_or(SymbolError::NotFound)
                .cloned()?;
            let scope_id = self.scope_id;
            Ok(SymbolScopeId {
                scope_id,
                symbol_id,
            })
        }
    }

    pub(crate) fn create_symbol(
        &mut self,
        name: &str,
    ) -> Result<SymbolScopeId<SCOPEID, SYMID>, SymbolError<SCOPEID, SYMID>> {
        if let Ok(id) = self.get_symbol_id(name) {
            Err(SymbolError::AlreadyDefined(id))
        } else {
            let symbol_id = self.get_next_id();

            self.name_to_id.insert(name.to_string(), symbol_id);

            Ok(SymbolScopeId {
                scope_id: self.scope_id,
                symbol_id,
            })
        }
    }

    pub(crate) fn remove_symbol(&mut self, name: &str) -> Result<(), SymbolError<SCOPEID,SYMID>> {
        self.name_to_id
            .remove(name)
            .ok_or(SymbolError::NotFound)
            .map(|_| ())
    }

    pub(crate) fn get_symbol_resoultion_barrier(&self) -> SymbolResolutionBarrier {
        self.symbol_resolution_barrier
    }

    pub(crate) fn get_scope_name(&self) -> &str {
        &self.scope
    }
    pub(crate) fn get_scope_fqn_name(&self) -> &str {
        &self.fqn_scope
    }

    pub(crate) fn new(
        name: &str,
        fqn_scope: &str,
        scope_id: SCOPEID,
        symbol_resolution_barrier: SymbolResolutionBarrier,
    ) -> Self {
        Self {
            scope: name.to_string(),
            highest_id: 1.into(),
            fqn_scope: fqn_scope.to_string(),
            scope_id,
            symbol_resolution_barrier,
            ..Default::default()
        }
    }

    pub(crate) fn get_scope_id(&self) -> SCOPEID {
        self.scope_id
    }

    pub(crate) fn add_reference_symbol(
        &mut self,
        name: &str,
        symbol_id: SymbolScopeId<SCOPEID,SYMID>,
    ) -> Result<(), SymbolError<SCOPEID, SYMID>> {
        if let Some(id) = self.ref_name_to_symbol_id.get(name) {
            Err(SymbolError::AlreadyDefined(*id))
        } else {
            self.ref_name_to_symbol_id
                .insert(name.to_string(), symbol_id);
            Ok(())
        }
    }

    fn get_next_id(&mut self) -> SYMID {
        let ret = self.highest_id;
        self.highest_id += 1;
        ret.into()
    }
}
