use std::collections::HashMap;

use super::{SymbolError, SymbolInfo, SymbolQuery, SymbolTable, SymbolWriter};
////////////////////////////////////////////////////////////////////////////////
// SymbolTree

pub type SymbolTreeTree = ego_tree::Tree<SymbolTable>;
pub type SymbolNodeRef<'a> = ego_tree::NodeRef<'a, SymbolTable>;
pub type SymbolNodeId = ego_tree::NodeId;
pub type SymbolNodeMut<'a> = ego_tree::NodeMut<'a, SymbolTable>;

#[derive(Debug, PartialEq,Eq, Clone)]
pub struct SymbolTree {
    tree: ego_tree::Tree<SymbolTable>,
    current_scope: SymbolNodeId,
}

impl Default for SymbolTree {
    fn default() -> Self {
        let root = SymbolTable::new_with_scope("root");
        let tree = SymbolTreeTree::new(root);
        let current_scope = tree.root().id();

        Self {
            tree,
            current_scope,
        }
    }
}

fn split_fqn(text: &str) -> Vec<&str> {
    text.split("::").collect()
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

    pub fn to_json(&self) {
        let mut hm = HashMap::<String,&SymbolTable >::new();

        for n in self.tree.nodes() {
            let name = n.value().get_scope_name();
            hm.insert(name.to_string(), n.value());
        }
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

pub fn display_tree(out: &mut std::fmt::Formatter<'_>, node : SymbolNodeRef, depth : usize) -> Result<(), std::fmt::Error>{

    let spaces = " ".repeat(depth * 4);
    writeln!(out, "{spaces}scope: {}", node.value().get_scope_name(), )?;

    let depth = depth + 1;
    let spaces = " ".repeat(depth * 4);

    let mut vars : Vec<_> = node.value().info.values().collect();
    vars.sort_by(|a,b| a.name.partial_cmp(&b.name).unwrap());

    for v in vars {
        if let Some(val) = v.value {
            writeln!(out, "{spaces} {:10} : {:04X}", v.name, val) ?;

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
