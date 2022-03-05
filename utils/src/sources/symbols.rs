
////////////////////////////////////////////////////////////////////////////////
// Traits

pub trait SymbolQuery {
    fn get_symbol_info(&self, name: &str) -> Result<&SymbolInfo, SymbolError>;

    fn get_value(&self, name: &str) -> Result<i64, SymbolError> {
        let si = self.get_symbol_info(name)?;
        si.value.ok_or(SymbolError::NoValue)
    }

    fn symbol_exists_from_name(&self, name: &str) -> bool {
        self.get_symbol_info(name).is_ok()
    }
}

pub trait SymbolWriter {
    fn add_symbol_with_value(
        &mut self,
        name: &str,
        value: i64,
    ) -> Result<SymbolId, SymbolError>;

    fn remove_symbol_name(&mut self, name: &str);
    fn add_symbol(&mut self, name: &str) -> Result<SymbolId, SymbolError>;

    fn add_reference_symbol(&mut self, name: &str, val: i64);
}


////////////////////////////////////////////////////////////////////////////////
// SymbolTree

pub type SymbolTreeTree = ego_tree::Tree<SymbolTable>;
pub type SymbolNodeRef<'a> = ego_tree::NodeRef<'a, SymbolTable>;
pub type SymbolNodeId = ego_tree::NodeId;
pub type SymbolNodeMut<'a> = ego_tree::NodeMut<'a, SymbolTable>;

#[derive(Debug, PartialEq, Clone)]
pub struct SymbolTree {
    tree: ego_tree::Tree<SymbolTable>,
    current_scope: SymbolNodeId,
}

impl SymbolTree {
    pub fn new() -> Self {
        let root = SymbolTable::new_with_scope("root");
        let tree = SymbolTreeTree::new(root);
        let current_scope = tree.root().id();

        Self {
            tree,
            current_scope,
        }
    }

    pub fn pop_scope(&mut self) {
        let n = self.tree.get(self.current_scope).unwrap();

        if let Some(n) = n.parent() {
            self.current_scope = n.id()
        }
    }

    // enters the child scope below the current_scope
    // If it doesn't exist then create it
    pub fn set_scope(&mut self, name : &str) {
        let node = self.tree.get(self.current_scope).unwrap();

        for c in node.children() {
            if c.value().get_scope_name() == name {
                self.current_scope = c.id();
                return
            }
        }

        let mut node_mut = self.tree.get_mut(self.current_scope).unwrap();
        let new_tab = SymbolTable::new_with_scope(name);
        let new_node = node_mut.append(new_tab);
        self.current_scope = new_node.id();
    }
}

impl SymbolQuery for SymbolTree {

    fn get_symbol_info(&self, name: &str) -> Result<&SymbolInfo, SymbolError> {
        let scope = self.current_scope;

        let mut node = self.tree.get(scope);

        while node.is_some() {
            if let Some(n) = node {
                if let Ok(v) = n.value().get_symbol_info(name) {
                    return Ok(v)
                }
            }
            node = node.and_then(|n| n.parent());
        }

        Err(SymbolError::NotFound)
    }
}

impl SymbolWriter for SymbolTree {
    fn add_symbol_with_value(
        &mut self,
        name: &str,
        value: i64,
    ) -> Result<SymbolId, SymbolError> {
        let mut node = self.tree.get_mut(self.current_scope).unwrap();
        node.value().add_symbol_with_value(name, value)
    }

    fn remove_symbol_name(&mut self, name: &str) {
        let mut node = self.tree.get_mut(self.current_scope).unwrap();
        node.value().remove_symbol_name(name)
    }

    fn add_symbol(&mut self, name: &str) -> Result<SymbolId, SymbolError> {
        let mut node = self.tree.get_mut(self.current_scope).unwrap();
        node.value().add_symbol(name)
    }

    fn add_reference_symbol(&mut self, name: &str, val: i64) {
        let mut node = self.tree.get_mut(self.current_scope).unwrap();
        node.value().add_reference_symbol(name, val)
    }

}


////////////////////////////////////////////////////////////////////////////////
use std::collections::HashMap;
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
    Mismatch { expected: i64 },
    NotFound,
    NoValue,
}

/// Holds information about symbols
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SymbolTable {
    scope: String,
    info: HashMap<SymbolId, SymbolInfo>,
    name_to_id: HashMap<String, SymbolId>,
    ref_name_to_value: HashMap<String, i64>,
    id: usize,
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
    fn add_symbol_with_value(
        &mut self,
        name: &str,
        value: i64,
    ) -> Result<SymbolId, SymbolError> {
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

    fn add_symbol(&mut self, name: &str) -> Result<SymbolId, SymbolError> {
        let name: String = name.into();

        if let Ok(sym_info) = self.get_symbol_info(&name) {
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

    fn get(&self, id: SymbolId) -> Result<&SymbolInfo, SymbolError> {
        self.info.get(&id).ok_or(SymbolError::NotFound)
    }

    fn get_mut(&mut self, id: SymbolId) -> Result<&mut SymbolInfo, SymbolError> {
        self.info.get_mut(&id).ok_or(SymbolError::NotFound)
    }

    fn symbol_exists(&self, id: SymbolId) -> bool {
        self.get(id).is_ok()
    }

    fn set_value(&mut self, id: SymbolId, value: i64) -> Result<(), SymbolError> {
        let i = self.get_mut(id)?;
        i.value = Some(value);
        Ok(())
    }

    fn get_next_id(&mut self) -> SymbolId {
        let ret = self.id;
        self.id += 1;
        ret
    }


}
