// symtab
use std::collections::{HashMap, VecDeque};

use crate::ast::AstNodeId;

pub type SymbolId = usize;
/// Holds information about a symbol
#[derive(Debug, PartialEq, Clone)]
pub struct SymbolInfo {
    /// Symbol Name
    pub name : String,
    /// Unique Symbol Id
    pub id: SymbolId,
    /// Id of Ast Node that symbol was defined in
    pub node_id: AstNodeId,
    /// Value, if any
    pub value : Option<i64>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum SymbolError {
    AlreadyDefined(SymbolId),
    NotFound,
    NoValue,
}

/// Holds information about symbols
#[derive(Debug, PartialEq, Clone)]
pub struct SymbolTable {
    info : Vec<SymbolInfo>,
    name_to_id: HashMap<String, SymbolId>,
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            info : Default::default(),
            name_to_id : Default::default(),
        }
    }
    pub fn get_value(&self, name : &String) -> Result<i64, SymbolError> {
        let info = self.get_from_name(name)?;

        info.value.ok_or(
            SymbolError::NoValue
        )
    }

    pub fn get(&self, id : SymbolId) -> Result<&SymbolInfo, SymbolError> {
        self.info.get(id).ok_or(SymbolError::NotFound)
    }

    pub fn get_mut(&mut self, id : SymbolId) -> Result<&mut SymbolInfo, SymbolError> {
        self.info.get_mut(id).ok_or(SymbolError::NotFound)
    }

    pub fn get_from_name<'a>(&'a self, name : &String) -> Result<&'a SymbolInfo, SymbolError> {
        let id = self.name_to_id.get(name).ok_or(SymbolError::NotFound)?;
        self.get(*id)
    }

    pub fn symbol_exists(&self, id: SymbolId) -> bool {
        self.get(id).is_ok()
    }

    pub fn symbol_exists_from_name(&self, name: &String) -> bool {
        self.get_from_name(name).is_ok()
    }

    fn set_value(&mut self, id : SymbolId, value : i64) -> Result<(),SymbolError> {
        let i = self.get_mut(id)?;
        i.value = Some(value);
        Ok(())
    }

    pub fn add_symbol_with_value(&mut self, name : &String, value : i64,node_id : AstNodeId ) -> Result<SymbolId, SymbolError> {
        let id = self.add_symbol(name, node_id)?;
        self.set_value(id, value)?;
        Ok(id)
    }

    pub fn add_symbol<'a>(&mut self, name : &String, node_id : AstNodeId ) -> Result<SymbolId, SymbolError> {
        if self.symbol_exists_from_name(name) {
            let x = self.get_from_name(&name.to_string()).unwrap();
            Err(SymbolError::AlreadyDefined(x.id))

        } else {
            let id = self.info.len();

            let info = SymbolInfo {
                name : name.clone(),
                id,
                node_id,
                value: None,
            };
            self.info.push(info);

            self.name_to_id.insert(name.clone(), id);

            Ok(id)
        }
    }
}


