use super::{
    SymbolError, SymbolInfo, SymbolNav, SymbolQuery, SymbolResolutionBarrier, SymbolScopeId,
    SymbolTable, SymbolWriter,
};
use serde::ser::SerializeMap;
use std::collections::HashMap;

////////////////////////////////////////////////////////////////////////////////
// SymbolTree

pub type SymbolTreeTree = ego_tree::Tree<SymbolTable>;
pub type SymbolNodeRef<'a> = ego_tree::NodeRef<'a, SymbolTable>;
pub type SymbolNodeId = ego_tree::NodeId;
pub type SymbolNodeMut<'a> = ego_tree::NodeMut<'a, SymbolTable>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SymbolTree {
    pub tree: ego_tree::Tree<SymbolTable>,
    current_scope_id: u64,
    next_scope_id: u64,
    scope_id_to_node_id: HashMap<u64, SymbolNodeId>,
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
            current_scope_id: 0,
            next_scope_id: 1,
            scope_id_to_node_id,
        }
    }
}

impl serde::Serialize for SymbolTree {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let hm = self.to_hash_map();
        let mut map = serializer.serialize_map(Some(hm.len()))?;
        for (k, v) in hm {
            map.serialize_entry(&k, &v)?;
        }
        map.end()
    }
}

impl<'de> serde::Deserialize<'de> for SymbolTree {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut ret = Self::new();
        let hm: HashMap<String, Option<i64>> = serde::Deserialize::deserialize(_deserializer)?;

        for (k, v) in hm {
            ret.add_fqn(&k, v)
        }

        Ok(ret)
    }
}

fn split_fqn(text: &str) -> Vec<&str> {
    text.split("::").collect()
}

fn get_subscope<'a>(n: SymbolNodeRef<'a>, name: &str) -> Option<SymbolNodeRef<'a>> {
    n.children().find(|c| c.value().get_scope_name() == name)
}

////////////////////////////////////////////////////////////////////////////////
/// All rely on current scope
impl SymbolTree {
    pub fn pop_scope(&mut self) {
        let n = self
            .get_node_from_id(self.current_scope_id)
            .expect("Invalid id");
        if let Some(n) = n.parent() {
            self.current_scope_id = n.value().get_scope_id()
        }
    }

    pub fn set_root(&mut self) {
        self.current_scope_id = self.tree.root().value().get_scope_id()
    }

    pub fn get_current_scope(&self) -> u64 {
        self.current_scope_id
    }

    pub fn get_current_scope_symbols(&self) -> &SymbolTable {
        self.get_symbols_from_id(self.current_scope_id).unwrap()
    }

    pub fn get_current_scope_fqn(&self) -> String {
        self.get_fqn_from_id(self.current_scope_id)
    }

    pub fn set_current_scope_from_id(&mut self, id: u64) -> Result<(), SymbolError> {
        self.get_node_mut_from_id(id)?;
        self.current_scope_id = id;
        Ok(())
    }

    pub fn get_current_scope_id(&self) -> u64 {
        self.current_scope_id
    }

    // enters the child scope below the current_scope
    // If it doesn't exist then create it
    pub fn set_current_scope(&mut self, name: &str) -> u64 {
        let new_scope_node_id = self.create_or_get_scope_id(name);
        self.current_scope_id = new_scope_node_id;
        new_scope_node_id
    }

    pub fn create_or_get_scope_id(&mut self, name: &str) -> u64 {
        self.create_or_get_scope_for_parent(name, self.current_scope_id)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Public functions
impl SymbolTree {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_symbols_to_scope(
        &mut self,
        scope_id: u64,
        names: &[String],
    ) -> Result<Vec<SymbolScopeId>, SymbolError> {
        let mut node = self.get_node_mut_from_id(scope_id)?;
        let syms = node.value();

        let ret: Result<Vec<SymbolScopeId>, SymbolError> =
            names.iter().map(|name| syms.add_symbol(name)).collect();
        ret
    }

    pub fn get_root_id(&self) -> u64 {
        self.tree.root().value().get_scope_id()
    }

    pub fn get_symbols_from_id(&self, scope_id: u64) -> Result<&SymbolTable, SymbolError> {
        self.get_node_from_id(scope_id).map(|n| n.value())
    }
    pub fn get_fqn_from_id(&self, scope_id: u64) -> String {
        let scope = self.get_symbols_from_id(scope_id).expect("Invalid scope");
        scope.get_scope_fqn_name().to_owned()
    }

    pub fn get_id_from_fqn(&self, name: &str) -> Option<SymbolNodeId> {
        let fqn = split_fqn(name);

        let mut scope_id = self.tree.root().id();

        for s in fqn.iter() {
            let mut found_scope = false;
            let n = self.tree.get(scope_id).unwrap();

            for c in n.children() {
                if &c.value().get_scope_name() == s {
                    scope_id = c.id();
                    found_scope = true;
                    break;
                }
            }
            if !found_scope {
                return None;
            }
        }

        Some(scope_id)
    }

    pub fn resolve_label(
        &self,
        name: &str,
        scope_id: u64,
        barrier: SymbolResolutionBarrier,
    ) -> Result<&SymbolInfo, SymbolError> {
        let scope_id = self.get_scope_node_id_from_id(scope_id)?;
        let mut node = self.tree.get(scope_id);

        while node.is_some() {
            if let Some(n) = node {
                if let Ok(v) = n.value().get_symbol_info(name) {
                    return Ok(v);
                }

                if !n
                    .value()
                    .get_symbol_resoultion_barrier()
                    .can_pass_barrier(barrier)
                {
                    break;
                }
            }
            node = node.and_then(|n| n.parent());
        }

        Err(SymbolError::NotFound)
    }

    pub fn get_symbol_nav(&mut self, scope_id: u64) -> SymbolNav {
        SymbolNav::new(self, scope_id)
    }

    pub fn get_symbol_reader(&self, scope_id: u64) -> SymbolTreeReader {
        SymbolTreeReader::new( self , scope_id)
    }
}

pub struct SymbolTreeReader<'a> {
    current_scope: u64,
    syms: &'a SymbolTree
}

impl<'a> SymbolTreeReader<'a> {
    pub fn new(syms: &'a SymbolTree, current_scope: u64) -> Self {
        Self {
            syms, current_scope
        }
    }
}

impl<'a> SymbolQuery for SymbolTreeReader<'a> {
    fn get_symbol_info(&self, name: &str) -> Result<&SymbolInfo, SymbolError> {
        let scope = self.syms.get_current_scope_id();
        self.syms.resolve_label(name, scope, SymbolResolutionBarrier::default())
    }

    fn get_symbol_info_from_id(&self, id: SymbolScopeId) -> Result<&SymbolInfo, SymbolError> {
        let node = self.syms.get_node_from_id(id.scope_id)?;
        node.value().get_symbol_info_from_id(id)
    }
}

// impl SymbolQuery for SymbolTree {
//     fn get_symbol_info_from_id(&self, id: SymbolScopeId) -> Result<&SymbolInfo, SymbolError> {
//         let node = self.get_node_from_id(id.scope_id)?;
//         node.value().get_symbol_info_from_id(id)
//     }

//     fn get_symbol_info(&self, name: &str) -> Result<&SymbolInfo, SymbolError> {
//         let scope = self.get_current_scope_id();
//         self.resolve_label(name, scope, SymbolResolutionBarrier::default())
//     }
// }

////////////////////////////////////////////////////////////////////////////////
// Private implementation funcs
impl SymbolTree {
    pub fn set_symbol_from_id(
        &mut self,
        symbol_id: SymbolScopeId,
        val: i64,
    ) -> Result<(), SymbolError> {
        let mut x = self.get_node_mut_from_id(symbol_id.scope_id)?;
        x.value().set_symbol(symbol_id, val)
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

    pub fn get_node_mut_from_id(&mut self, scope_id: u64) -> Result<SymbolNodeMut, SymbolError> {
        let node_id = self.get_node_id_from_scope_id(scope_id)?;
        self.tree.get_mut(node_id).ok_or(SymbolError::InvalidScope)
    }

    pub fn get_next_scope_id(&mut self) -> u64 {
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

    pub fn get_node_id_from_scope_id(&self, scope_id: u64) -> Result<SymbolNodeId, SymbolError> {
        self.scope_id_to_node_id
            .get(&scope_id)
            .cloned()
            .ok_or(SymbolError::InvalidScope)
    }

    pub fn get_node_from_id(&self, scope_id: u64) -> Result<SymbolNodeRef, SymbolError> {
        let node_id = self.get_node_id_from_scope_id(scope_id)?;
        self.tree.get(node_id).ok_or(SymbolError::InvalidScope)
    }

    fn insert_new_table(
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
impl SymbolTree {
    pub fn to_hash_map(&self) -> HashMap<String, Option<i64>> {
        let mut hm: HashMap<String, Option<i64>> = HashMap::new();

        walk_syms(self.tree.root(), self.get_current_scope_fqn(), &mut |si| {
            hm.insert(si.name().to_string(), si.value);
        });

        hm
    }

    pub fn to_json(&self) -> String {
        let hm = self.to_hash_map();
        serde_json::to_string_pretty(&hm).unwrap()
    }

    // This is shit, much shame
    pub fn add_fqn(&mut self, text: &str, val: Option<i64>) {
        let items: Vec<_> = split_fqn(text);

        let (path, sym) = match items.len() {
            0 => panic!("WTF"),
            1 => panic!("Neeed 2!"),
            _ => (&items[0..items.len() - 1], &items[items.len() - 1]),
        };

        assert!(path[0].is_empty());

        // pop the first one off
        let mut scope_id = self.tree.root().value().get_scope_id();

        for part in &path[1..] {
            let n = self.get_node_from_id(scope_id).unwrap();
            let n_id = n.value().get_scope_id();

            if let Some(new_id) = get_subscope(n, part) {
                scope_id = new_id.value().get_scope_id();
            } else {
                let new_scope_id =
                    self.insert_new_table(part, n_id, SymbolResolutionBarrier::default());
                scope_id = new_scope_id
            }
        }

        let mut n = self.get_node_mut_from_id(scope_id).unwrap();
        n.value().add_symbol_with_value(sym, val.unwrap()).unwrap();
    }
}

pub fn walk_syms<F>(node: SymbolNodeRef, scope: String, f: &mut F)
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
    out: &mut std::fmt::Formatter<'_>,
    node: SymbolNodeRef,
    depth: usize,
) -> Result<(), std::fmt::Error> {
    let spaces = " ".repeat(depth * 4);
    writeln!(out, "{spaces}scope: {}", node.value().get_scope_name(),)?;

    let depth = depth + 1;
    let spaces = " ".repeat(depth * 4);

    let mut vars: Vec<_> = node.value().get_symbol_info_hash().values().collect();
    vars.sort_by(|a, b| a.name().partial_cmp(b.name()).unwrap());

    for v in vars {
        if let Some(val) = v.value {
            writeln!(out, "{spaces} {:10} : {:04X}", v.name(), val)?;
        }
    }

    let depth = depth + 1;
    for x in node.children() {
        writeln!(out)?;
        display_tree(out, x, depth)?;
    }

    Ok(())
}

impl std::fmt::Display for SymbolTree {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        display_tree(out, self.tree.root(), 0)
    }
}

impl SymbolWriter for SymbolTree {
    fn add_symbol_with_value(
        &mut self,
        name: &str,
        value: i64,
    ) -> Result<SymbolScopeId, SymbolError> {
        self.get_symbol_nav(self.current_scope_id)
            .add_symbol_with_value(name, value)
    }

    fn remove_symbol_name(&mut self, name: &str) {
        self.get_symbol_nav(self.current_scope_id)
        .remove_symbol_name(name)
    }

    fn add_symbol(&mut self, name: &str) -> Result<SymbolScopeId, SymbolError> {
        self.get_symbol_nav(self.current_scope_id)
        .add_symbol(name)
    }

    fn add_reference_symbol(&mut self, name: &str, val: i64) {
        self.get_symbol_nav(self.current_scope_id)
        .add_reference_symbol(name, val)
    }
}

#[allow(unused_imports)]
mod test {
    use super::*;

    #[test]
    fn test_sym_tree() {
        let mut st = SymbolTree::default();

        let _ = st.add_symbol_with_value("root_gaz", 100);

        st.set_current_scope("scope_a");
        let _ = st.add_symbol_with_value("gaz", 100);
        let _ = st.add_symbol_with_value("root_gaz", 100);

        let scope_fqn = st.get_current_scope_fqn();
        println!("SCOPE is {scope_fqn}");
        st.pop_scope();

        let scope_fqn = st.get_current_scope_fqn();
        println!("SCOPE is {scope_fqn}");
    }
}
