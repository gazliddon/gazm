use std::collections::HashMap;

use serde::ser::SerializeMap;

use super::{SymbolError, SymbolInfo, SymbolQuery, SymbolScopeId, SymbolTable, SymbolWriter};

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
        let root = SymbolTable::new_with_scope("", "", 0);
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
    for c in n.children() {
        if c.value().get_scope_name() == name {
            return Some(c);
        }
    }
    None
}

impl SymbolTree {
    fn get_scope_node_mut_from_id(&mut self, scope_id: u64) -> Result<SymbolNodeMut,SymbolError> {
        let id = self.scope_id_to_node_id.get(&scope_id).ok_or(SymbolError::InvalidScope)?;
        Ok( self.tree.get_mut(*id).unwrap() )
    }
    pub fn safe_add_symbols_to_scope(
        &mut self,
        scope_id: u64,
        names: &[String],
    ) -> Vec<SymbolScopeId> {
        let old_scope_id = self.get_current_scope_id();
        self.set_scope_from_id(scope_id).unwrap();

        let ret = names
            .iter()
            .map(|name| {
                let sym = self.get_symbol_info(name);
                match sym {
                    Ok(si) => si.symbol_id,
                    Err(..) => self.add_symbol(name).unwrap(),
                }
            })
            .collect();

        self.set_scope_from_id(old_scope_id).unwrap();
        ret
    }
    pub fn add_symbols_to_scope(
        &mut self,
        scope_id: u64,
        names: &[String],
    ) -> Result<Vec<SymbolScopeId>, SymbolError> {
        let old_scope_id = self.get_current_scope_id();
        self.set_scope_from_id(scope_id)?;

        let ret: Result<Vec<SymbolScopeId>, SymbolError> =
            names.iter().map(|name| self.add_symbol(name)).collect();

        self.set_scope_from_id(old_scope_id)?;

        ret
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_next_scope_id(&mut self) -> u64 {
        let ret = self.next_scope_id;
        self.next_scope_id += 1;
        ret
    }

    pub fn pop_scope(&mut self) {
        let id = self.get_node_id_from_scope_id(self.current_scope_id).unwrap();

        let n = self.tree.get(id).unwrap();

        if let Some(n) = n.parent() {
            self.current_scope_id = n.value().get_scope_id()
        }
    }
    pub fn get_root(&self) -> SymbolNodeId {
        self.tree.root().id()
    }
    pub fn get_root_id(&self) -> u64 {
        self.tree.root().value().get_scope_id()
    }

    pub fn set_root(&mut self) {
        self.current_scope_id = self.tree.root().value().get_scope_id()
    }

    pub fn get_current_scope_chain(&self) -> Vec<u64> {
        self.get_scope_chain(self.current_scope_id).unwrap()
    }

    fn get_scope_node_id_from_id(&self, scope_id: u64) -> Result<SymbolNodeId,SymbolError> {
        self.scope_id_to_node_id.get(&scope_id).cloned().ok_or(SymbolError::InvalidScope)
    }

    pub fn get_scope_chain(&self, scope_id: u64) -> Result<Vec<u64>,SymbolError> {
        let scope_node_id = self.get_scope_node_id_from_id(scope_id)?;
        let mut cscope = self.tree.get(scope_node_id).unwrap();
        let mut ret = vec![scope_id];

        while let Some(s) = cscope.parent() {
            ret.push(s.value().get_scope_id());
            cscope = s
        }

        ret.reverse();
        Ok(ret)

    }

    pub fn get_current_scope(&self) -> u64 {
        self.current_scope_id
    }

    pub fn get_current_scope_symbols(&self) -> &SymbolTable {
        self.get_scope_symbols_from_id(self.current_scope_id).unwrap()
    }

    fn get_node_id_from_scope_id(&self, scope_id: u64) -> Result<SymbolNodeId, SymbolError>  {
        self.scope_id_to_node_id.get(&scope_id).cloned().ok_or(SymbolError::InvalidScope)
    }

    pub fn get_scope_symbols_from_id(&self, scope_id: u64) -> Result<&SymbolTable,SymbolError> {
        self.get_scope_node_from_id(scope_id).map(|n| n.value())
    }

    fn get_scope_node_from_id(&self, scope_id: u64) -> Result<SymbolNodeRef,SymbolError> {
        let node_id = self.get_node_id_from_scope_id(scope_id)?;
        self.tree.get(node_id).ok_or(SymbolError::InvalidScope)
    }

    pub fn get_current_scope_fqn(&self) -> String {
        let scopes = self.get_current_scope_chain();
        let v: Vec<_> = scopes
            .into_iter()
            .map(|id| self.get_scope_symbols_from_id(id).unwrap().get_scope_name())
            .collect();
        v.join("::")
    }

    pub fn get_fqn_scope_id(&self, name: &str) -> Option<SymbolNodeId> {
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

    pub fn set_scope_from_id(&mut self, id: u64) -> Result<(),SymbolError>{
        self.get_scope_node_mut_from_id(id)?;
        self.current_scope_id = id;
        Ok(())
    }

    pub fn get_current_scope_id(&self) -> u64 {
        self.current_scope_id
    }

    // enters the child scope below the current_scope
    // If it doesn't exist then create it
    pub fn set_scope(&mut self, name: &str) -> u64 {
        let node = self.get_scope_node_from_id(self.current_scope_id).expect("WHUT");

        for c in node.children() {
            if c.value().get_scope_name() == name {
                let id = c.value().get_scope_id();
                self.current_scope_id = id;
                return id
            }
        }
        let new_scope_node_id = self.insert_new_table(name, self.current_scope_id);
        self.current_scope_id = new_scope_node_id;
        new_scope_node_id
    }

    pub fn to_hash_map(&self) -> HashMap<String, Option<i64>> {
        let mut hm: HashMap<String, Option<i64>> = HashMap::new();

        walk_syms(
            self.tree.root(),
            self.get_current_scope_fqn(),
            &mut |name: &str, value: Option<i64>| {
                hm.insert(name.to_string(), value);
            },
        );

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

        assert!(path[0] == "");

        // pop the first one off
        let mut scope_id = self.tree.root().value().get_scope_id();

        for part in &path[1..] {
            let n = self.get_scope_node_from_id(scope_id).unwrap();
            let n_id = n.value().get_scope_id();

            if let Some(new_id) = get_subscope(n, part) {
                scope_id = new_id.value().get_scope_id();
            } else {
                let new_scope_id = self.insert_new_table(part, n_id);
                scope_id = new_scope_id
            }
        }

        let mut n = self.get_scope_node_mut_from_id(scope_id).unwrap();
        n.value().add_symbol_with_value(sym, val.unwrap()).unwrap();
    }

    fn insert_new_table(&mut self, name: &str, parent_id: u64) -> u64 {
        let tab = self.create_new_table(name, "");
        let tab_id = tab.get_scope_id();
        let parent_id = self.scope_id_to_node_id.get(&parent_id).unwrap();
        let mut parent_mut = self.tree.get_mut(*parent_id).unwrap();
        let mut n = parent_mut.append(tab);
        self.scope_id_to_node_id.insert(tab_id, n.id());
        n.value().get_scope_id()
    }

    fn create_new_table(&mut self, name: &str, fqn: &str) -> SymbolTable {
        let scope_id = self.get_next_scope_id();
        SymbolTable::new_with_scope(name, fqn,scope_id)
    }

    pub fn resolve_label(&self, name: &str, scope_id : u64) -> Result<&SymbolInfo, SymbolError> {
        let scope_id = self.scope_id_to_node_id.get(&scope_id).ok_or(SymbolError::InvalidScope)?;
        let mut node = self.tree.get(*scope_id);

        while node.is_some() {
            if let Some(n) = node {
                if let Ok(v) = n.value().get_symbol_info(name) {
                    return Ok(v);
                }
            }
            node = node.and_then(|n| n.parent());
        }

        Err(SymbolError::NotFound)
    }
}

impl SymbolQuery for SymbolTree {
    fn get_symbol_info_from_id(&self, id: SymbolScopeId) -> Result<&SymbolInfo, SymbolError> {
        let node_id = self.scope_id_to_node_id.get(&id.scope_id).unwrap();
        self.tree
            .get(*node_id)
            .unwrap()
            .value()
            .get_symbol_info_from_id(id)
    }

    fn get_symbol_info(&self, name: &str) -> Result<&SymbolInfo, SymbolError> {
        let scope = self.get_current_scope_id();
        self.resolve_label(name, scope)
    }
}

pub fn walk_syms<F>(node: SymbolNodeRef, scope: String, f: &mut F)
where
    F: FnMut(&str, Option<i64>),
{
    for info in node.value().get_symbols() {
        let fqn = format!("{scope}::{}", info.name());
        f(&fqn, info.value)
    }

    for n in node.children() {
        let scope = format!("{scope}::{}", n.value().get_scope_name());
        walk_syms(n, scope, f);
    }
}

pub fn print_syms(node: SymbolNodeRef, scope: String) {
    walk_syms(node, scope, &mut |name, val| println!("{name} = {:?}", val))
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

    let mut vars: Vec<_> = node.value().info.values().collect();
    vars.sort_by(|a, b| a.name().partial_cmp(&b.name()).unwrap());

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
    fn set_symbol(&mut self, symbol_id: SymbolScopeId, val: i64) -> Result<(), SymbolError> {
        let node_id = self
            .scope_id_to_node_id
            .get(&symbol_id.scope_id)
            .expect("Internal error");
        let mut x = self.tree.get_mut(*node_id).expect("Interal error");
        x.value().set_symbol(symbol_id, val)
    }

    fn add_symbol_with_value(
        &mut self,
        name: &str,
        value: i64,
    ) -> Result<SymbolScopeId, SymbolError> {
        let id = self.scope_id_to_node_id.get(&self.current_scope_id).unwrap();
        let mut node = self.tree.get_mut(*id).unwrap();
        node.value().add_symbol_with_value(name, value)
    }


    fn remove_symbol_name(&mut self, name: &str) {
        let mut node = self.get_scope_node_mut_from_id(self.current_scope_id).unwrap();
        node.value().remove_symbol_name(name)
    }

    fn add_symbol(&mut self, name: &str) -> Result<SymbolScopeId, SymbolError> {
        let mut node = self.get_scope_node_mut_from_id(self.current_scope_id).unwrap();
        node.value().add_symbol(name)
    }

    fn add_reference_symbol(&mut self, name: &str, val: i64) {
        let mut node = self.get_scope_node_mut_from_id(self.current_scope_id).unwrap();
        node.value().add_reference_symbol(name, val)
    }
}

#[allow(unused_imports)]
mod test {
    use super::*;

    #[test]
    fn test_sym_tree() {
        let mut st = SymbolTree::default();

        let _ = st.add_symbol_with_value("root_gaz", 100);

        st.set_scope("scope_a");
        let _ = st.add_symbol_with_value("gaz", 100);
        let _ = st.add_symbol_with_value("root_gaz", 100);

        let scope_fqn = st.get_current_scope_fqn();
        println!("SCOPE is {scope_fqn}");
        st.pop_scope();

        let scope_fqn = st.get_current_scope_fqn();
        println!("SCOPE is {scope_fqn}");
    }
}
