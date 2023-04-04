use std::collections::HashMap;

use serde::ser::SerializeMap;

use super::{SymbolError, SymbolInfo, SymbolQuery, SymbolTable, SymbolWriter};

////////////////////////////////////////////////////////////////////////////////
// SymbolTree

pub type SymbolTreeTree = ego_tree::Tree<SymbolTable>;
pub type SymbolNodeRef<'a> = ego_tree::NodeRef<'a, SymbolTable>;
pub type SymbolNodeId = ego_tree::NodeId;
pub type SymbolNodeMut<'a> = ego_tree::NodeMut<'a, SymbolTable>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SymbolTree {
    tree: ego_tree::Tree<SymbolTable>,
    current_scope: SymbolNodeId,
}

impl Default for SymbolTree {
    fn default() -> Self {
        let root = SymbolTable::new_with_scope("");
        let tree = SymbolTreeTree::new(root);
        let current_scope = tree.root().id();

        Self {
            tree,
            current_scope,
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

fn get_subscope(n: SymbolNodeRef, name: &str) -> Option<SymbolNodeId> {
    for c in n.children() {
        if c.value().get_scope_name() == name {
            return Some(c.id());
        }
    }
    None
}

impl SymbolTree {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pop_scope(&mut self) {
        let n = self.tree.get(self.current_scope).unwrap();

        if let Some(n) = n.parent() {
            self.current_scope = n.id()
        }
    }
    pub fn get_root(&self) -> SymbolNodeId {
        self.tree.root().id()
    }

    pub fn set_root(&mut self) {
        self.current_scope = self.tree.root().id()
    }

    pub fn get_current_scope_chain(&self) -> Vec<SymbolNodeId> {
        let mut cscope = self.tree.get(self.current_scope).unwrap();
        let mut ret = vec![cscope.id()];

        while let Some(s) = cscope.parent() {
            ret.push(s.id());
            cscope = s
        }

        ret.reverse();
        ret
    }
    pub fn get_current_scope(&self) -> SymbolNodeId {
        self.current_scope
    }

    pub fn get_current_scope_symbols(&self) -> &SymbolTable {
        self.tree.get(self.current_scope).unwrap().value()
    }

    pub fn get_scope_symbols(&self, scope_id : SymbolNodeId) -> Option<&SymbolTable> {
        self.tree.get(scope_id).map(|nr| nr.value())
    }

    pub fn get_current_scope_fqn(&self) -> String {
        let scopes = self.get_current_scope_chain();
        let v: Vec<_> = scopes
            .into_iter()
            .map(|id| self.tree.get(id).unwrap().value().get_scope_name())
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

    // enters the child scope below the current_scope
    // If it doesn't exist then create it
    pub fn set_scope(&mut self, name: &str) {
        let node = self.tree.get(self.current_scope).unwrap();

        for c in node.children() {
            if c.value().get_scope_name() == name {
                self.current_scope = c.id();
                return;
            }
        }

        let mut node_mut = self.tree.get_mut(self.current_scope).unwrap();
        let new_tab = SymbolTable::new_with_scope(name);
        let new_node = node_mut.append(new_tab);
        self.current_scope = new_node.id();
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
        let mut scope_id = self.tree.root().id();

        for part in &path[1..] {
            let n = self.tree.get(scope_id).unwrap();

            if let Some(new_id) = get_subscope(n, part) {
                scope_id = new_id
            } else {
                let mut n_mut = self.tree.get_mut(scope_id).unwrap();
                let n = n_mut.insert_after(SymbolTable::new_with_scope(part));
                scope_id = n.id()
            }
        }

        let mut n = self.tree.get_mut(scope_id).unwrap();
        n.value().add_symbol_with_value(sym, val.unwrap()).unwrap();
    }
}

impl SymbolQuery for SymbolTree {
    fn get_symbol_info(&self, name: &str) -> Result<&SymbolInfo, SymbolError> {
        let scope = self.current_scope;

        let mut node = self.tree.get(scope);

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

pub fn walk_syms<F>(node: SymbolNodeRef, scope: String, f: &mut F)
where
    F: FnMut(&str, Option<i64>),
{
    for info in node.value().get_symbols() {
        let fqn = format!("{scope}::{}", info.name);
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
    vars.sort_by(|a, b| a.name.partial_cmp(&b.name).unwrap());

    for v in vars {
        if let Some(val) = v.value {
            writeln!(out, "{spaces} {:10} : {:04X}", v.name, val)?;
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
    fn add_symbol_with_value(&mut self, name: &str, value: i64) -> Result<u64, SymbolError> {
        let mut node = self.tree.get_mut(self.current_scope).unwrap();
        node.value().add_symbol_with_value(name, value)
    }

    fn remove_symbol_name(&mut self, name: &str) {
        let mut node = self.tree.get_mut(self.current_scope).unwrap();
        node.value().remove_symbol_name(name)
    }

    fn add_symbol(&mut self, name: &str) -> Result<u64, SymbolError> {
        let mut node = self.tree.get_mut(self.current_scope).unwrap();
        node.value().add_symbol(name)
    }

    fn add_reference_symbol(&mut self, name: &str, val: i64) {
        let mut node = self.tree.get_mut(self.current_scope).unwrap();
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
