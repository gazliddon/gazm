use std::collections::HashMap;
use thin_vec::ThinVec;

use super::{
    ScopedName, SymbolError, SymbolInfo, SymbolResolutionBarrier, SymbolScopeId, SymbolTable,
    SymbolTreeReader, SymbolTreeWriter,
};

////////////////////////////////////////////////////////////////////////////////
// SymbolTree
pub type SymbolTreeTree = ego_tree::Tree<SymbolTable>;
pub type SymbolNodeRef<'a> = ego_tree::NodeRef<'a, SymbolTable>;
pub type SymbolNodeId = ego_tree::NodeId;
pub type SymbolNodeMut<'a> = ego_tree::NodeMut<'a, SymbolTable>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SymbolTree {
    pub tree: ego_tree::Tree<SymbolTable>,
    pub next_scope_id: u64,
    pub scope_id_to_node_id: HashMap<u64, SymbolNodeId>,
    pub scope_id_to_symbol_info: HashMap<SymbolScopeId, SymbolInfo>,
}

impl Default for SymbolTree {
    fn default() -> Self {
        let root = SymbolTable::new("", "", 0, SymbolResolutionBarrier::default());
        let tree = SymbolTreeTree::new(root);
        let current_scope = tree.root().id();
        let mut scope_id_to_node_id: HashMap<u64, SymbolNodeId> = Default::default();
        scope_id_to_node_id.insert(0, current_scope);

        Self {
            tree,
            next_scope_id: 1,
            scope_id_to_node_id,
            scope_id_to_symbol_info: Default::default(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Public functions
impl SymbolTree {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn remove_symbol_for_id(&mut self, name: &str, scope_id: u64) -> Result<(), SymbolError> {
        let node_id = self.get_node_id_from_scope_id(scope_id)?;
        let mut node_mut = self.tree.get_mut(node_id).unwrap();
        node_mut.value().remove_symbol(name)
    }

    pub fn create_symbols_in_scope(
        &mut self,
        scope_id: u64,
        names: &[String],
    ) -> Result<ThinVec<SymbolScopeId>, SymbolError> {
        let ret: Result<ThinVec<SymbolScopeId>, SymbolError> = names
            .iter()
            .map(|name| self.create_symbol_in_scope(scope_id, name))
            .collect();
        ret
    }

    pub fn create_symbol_in_scope(
        &mut self,
        scope_id: u64,
        name: &str,
    ) -> Result<SymbolScopeId, SymbolError> {
        let mut node = self.get_node_mut_from_id(scope_id)?;
        let syms = node.value();
        let symbol_id = syms.create_symbol(name)?;
        let si = SymbolInfo::new(name, None, symbol_id, syms.get_scope_fqn_name());

        self.scope_id_to_symbol_info.insert(symbol_id, si);
        Ok(symbol_id)
    }

    pub fn set_value_for_id(&mut self, id: SymbolScopeId, val: i64) -> Result<(), SymbolError> {
        let x = self
            .scope_id_to_symbol_info
            .get_mut(&id)
            .ok_or(SymbolError::NotFound)?;
        x.value = Some(val);
        Ok(())
    }

    pub fn get_root_scope_id(&self) -> u64 {
        self.tree.root().value().get_scope_id()
    }

    pub fn get_symbols_from_id(&self, scope_id: u64) -> Result<&SymbolTable, SymbolError> {
        self.get_node_from_id(scope_id).map(|n| n.value())
    }

    pub fn get_fqn_from_id(&self, scope_id: u64) -> String {
        let scope = self.get_symbols_from_id(scope_id).expect("Invalid scope");
        scope.get_scope_fqn_name().to_owned()
    }

    pub fn resolve_label(
        &self,
        name: &str,
        scope_id: u64,
        barrier: SymbolResolutionBarrier,
    ) -> Result<SymbolScopeId, SymbolError> {
        let node_scope_id = self.get_scope_node_id_from_id(scope_id)?;
        let mut node = self.tree.get(node_scope_id);


        while let Some(n) = node {
            if let Ok(v) = n.value().get_symbol_id(name) {
                return Ok(v);
            }

            if !n
                .value()
                .get_symbol_resoultion_barrier()
                .can_pass_barrier(barrier)
            {
                break;
            }
            node = node.and_then(|n| n.parent());
        }


        Err(SymbolError::NotFound)
    }

    pub fn get_writer(&mut self, scope_id: u64) -> SymbolTreeWriter {
        SymbolTreeWriter::new(self, scope_id)
    }

    pub fn get_root_writer(&mut self) -> SymbolTreeWriter {
        SymbolTreeWriter::new(self, self.get_root_scope_id())
    }

    pub fn get_reader(&self, scope_id: u64) -> SymbolTreeReader {
        SymbolTreeReader::new(self, scope_id)
    }

    pub fn get_root_reader(&self) -> SymbolTreeReader {
        self.get_reader(self.get_root_scope_id())
    }

    pub fn get_symbol_info_from_id(
        &self,
        symbol_id: SymbolScopeId,
    ) -> Result<&SymbolInfo, SymbolError> {
        self.scope_id_to_symbol_info
            .get(&symbol_id)
            .ok_or(SymbolError::InvalidId)
    }

    pub fn get_symbol_info_from_scoped_name(
        &self,
        name: &ScopedName,
    ) -> Result<&SymbolInfo, SymbolError> {
        assert!(name.is_abs());

        let scopes = name.path();
        let name = name.symbol();

        let mut current_node = self.tree.root();

        for x in scopes.iter() {
            for c in current_node.children() {
                if c.value().get_scope_name() == *x {
                    current_node = c;
                    break;
                }
                return Err(SymbolError::InvalidScope);
            }
        }

        self.get_symbol_info(name, current_node.value().get_scope_id())
    }

    pub fn get_symbol_info(&self, name: &str, scope_id: u64) -> Result<&SymbolInfo, SymbolError> {
        let n = self.get_symbols_from_id(scope_id)?;
        let id = n.get_symbol_id(name)?;
        self.scope_id_to_symbol_info
            .get(&id)
            .ok_or(SymbolError::NotFound)
    }

    pub fn add_reference_symbol(
        &mut self,
        name: &str,
        scope_id: u64,
        symbol_id: SymbolScopeId,
    ) -> Result<(), SymbolError> {
        let mut node = self.get_node_from_id_mut(scope_id).expect("Invalid scope");
        node.value().add_reference_symbol(name, symbol_id)
    }

    pub fn create_or_get_scope_for_parent(&mut self, name: &str, id: u64) -> u64 {
        let node = self.get_node_from_id(id).expect("Invalid scope");

        for c in node.children() {
            if c.value().get_scope_name() == name {
                let id = c.value().get_scope_id();
                return id;
            }
        }
        self.insert_new_table(name, id, SymbolResolutionBarrier::default())
    }

    pub fn set_symbol_from_id(
        &mut self,
        symbol_id: SymbolScopeId,
        val: i64,
    ) -> Result<(), SymbolError> {
        let mut x = self
            .scope_id_to_symbol_info
            .get_mut(&symbol_id)
            .ok_or(SymbolError::InvalidScope)?;
        x.value = Some(val);
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////
// Private implementation funcs
impl SymbolTree {
    pub(crate) fn get_tree(&self) -> &SymbolTreeTree {
        &self.tree
    }

    pub(crate) fn get_tree_mut(&mut self) -> &mut SymbolTreeTree {
        &mut self.tree
    }

    pub(crate) fn get_node_mut_from_id(
        &mut self,
        scope_id: u64,
    ) -> Result<SymbolNodeMut, SymbolError> {
        let node_id = self.get_node_id_from_scope_id(scope_id)?;
        self.tree.get_mut(node_id).ok_or(SymbolError::InvalidScope)
    }
    pub(crate) fn get_next_scope_id(&mut self) -> u64 {
        let ret = self.next_scope_id;
        self.next_scope_id += 1;
        ret
    }

    fn get_scope_node_id_from_id(&self, scope_id: u64) -> Result<SymbolNodeId, SymbolError> {
        self.scope_id_to_node_id
            .get(&scope_id)
            .cloned()
            .ok_or(SymbolError::InvalidScope)
    }

    pub(crate) fn get_node_id_from_scope_id(
        &self,
        scope_id: u64,
    ) -> Result<SymbolNodeId, SymbolError> {
        self.scope_id_to_node_id
            .get(&scope_id)
            .cloned()
            .ok_or(SymbolError::InvalidScope)
    }

    pub(crate) fn get_node_from_id(&self, scope_id: u64) -> Result<SymbolNodeRef, SymbolError> {
        let node_id = self.get_node_id_from_scope_id(scope_id)?;
        self.tree.get(node_id).ok_or(SymbolError::InvalidScope)
    }

    pub(crate) fn get_node_from_id_mut(
        &mut self,
        scope_id: u64,
    ) -> Result<SymbolNodeMut, SymbolError> {
        let node_id = self.get_node_id_from_scope_id(scope_id)?;
        self.tree.get_mut(node_id).ok_or(SymbolError::InvalidScope)
    }

    pub(crate) fn insert_new_table(
        &mut self,
        name: &str,
        parent_id: u64,
        barrier: SymbolResolutionBarrier,
    ) -> u64 {
        let tab = self.create_new_table(name, parent_id, barrier);
        let tab_id = tab.get_scope_id();
        let parent_id = self.scope_id_to_node_id.get(&parent_id).unwrap();
        let mut parent_mut = self.tree.get_mut(*parent_id).unwrap();
        let mut n = parent_mut.append(tab);
        self.scope_id_to_node_id.insert(tab_id, n.id());
        n.value().get_scope_id()
    }

    fn create_new_table(
        &mut self,
        name: &str,
        parent_id: u64,
        barrier: SymbolResolutionBarrier,
    ) -> SymbolTable {
        let parent_fqn = self.get_fqn_from_id(parent_id);
        let fqn = format!("{parent_fqn}::{name}");
        let scope_id = self.get_next_scope_id();
        SymbolTable::new(name, &fqn, scope_id, barrier)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Functions to do with serialization
fn walk_syms<F>(node: SymbolNodeRef, scope: String, f: &mut F)
where
    F: FnMut(&SymbolInfo),
{
    for info in node.value().get_symbols() {
        f(info)
    }

    for n in node.children() {
        let scope = format!("{scope}::{}", n.value().get_scope_name());
        walk_syms(n, scope, f);
    }
}

pub fn print_syms(node: SymbolNodeRef, scope: String) {
    walk_syms(node, scope, &mut |sym| {
        println!("{} = {:?}", sym.scoped_name(), sym.value)
    })
}

pub fn display_tree(
    _out: &mut std::fmt::Formatter<'_>,
    _node: SymbolNodeRef,
    _depth: usize,
) -> Result<(), std::fmt::Error> {
    panic!()
}

impl std::fmt::Display for SymbolTree {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        display_tree(out, self.tree.root(), 0)
    }
}

#[allow(unused_imports)]
mod test {
    use super::*;

    #[test]
    fn test_sym_tree() {
        // let mut st = SymbolTree::default();

        // let _ = st.add_symbol_with_value("root_gaz", 100);

        // st.set_current_scope("scope_a");
        // let _ = st.add_symbol_with_value("gaz", 100);
        // let _ = st.add_symbol_with_value("root_gaz", 100);

        // let scope_fqn = st.get_current_scope_fqn();
        // println!("SCOPE is {scope_fqn}");
        // st.pop_scope();

        // let scope_fqn = st.get_current_scope_fqn();
        // println!("SCOPE is {scope_fqn}");
    }
}
