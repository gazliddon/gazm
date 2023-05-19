use std::collections::HashMap;
use thin_vec::ThinVec;

use super::{
    symboltable::SymbolTable, symboltreereader::SymbolTreeReader,
    symboltreewriter::SymbolTreeWriter, ScopeIdTraits, ScopedName, SymIdTraits, SymbolError,
    SymbolInfo, SymbolResolutionBarrier, SymbolScopeId,
};

////////////////////////////////////////////////////////////////////////////////
// SymbolTree
type SymbolTreeTree<SCOPEID, SYMID> = ego_tree::Tree<SymbolTable<SCOPEID, SYMID>>;

pub (crate) type SymbolNodeRef<'a, SCOPEID, SYMID> = ego_tree::NodeRef<'a, SymbolTable<SCOPEID, SYMID>>;
pub (crate) type SymbolNodeId = ego_tree::NodeId;
pub (crate) type SymbolNodeMut<'a, SCOPEID, SYMID> = ego_tree::NodeMut<'a, SymbolTable<SCOPEID, SYMID>>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SymbolTree<SCOPEID, SYMID, SYMVALUE>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
{
    pub tree: ego_tree::Tree<SymbolTable<SCOPEID, SYMID>>,
    pub next_scope_id: SCOPEID,
    pub scope_id_to_node_id: HashMap<SCOPEID, SymbolNodeId>,
    pub scope_id_to_symbol_info:
        HashMap<SymbolScopeId<SCOPEID, SYMID>, SymbolInfo<SCOPEID, SYMID, SYMVALUE>>,
}

impl<SCOPEID, SYMID, SYMVALUE> Default for SymbolTree<SCOPEID, SYMID, SYMVALUE>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
{
    fn default() -> Self {
        let root: SymbolTable<SCOPEID, SYMID> =
            SymbolTable::new("", "", 0.into(), SymbolResolutionBarrier::default());
        let tree: SymbolTreeTree<SCOPEID, SYMID> = SymbolTreeTree::new(root);
        let current_scope = tree.root().id();
        let mut scope_id_to_node_id: HashMap<SCOPEID, SymbolNodeId> = Default::default();
        scope_id_to_node_id.insert(0.into(), current_scope);

        Self {
            tree,
            next_scope_id: 1.into(),
            scope_id_to_node_id,
            scope_id_to_symbol_info: Default::default(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Public functions
impl<SCOPEID, SYMID, V> SymbolTree<SCOPEID, SYMID, V>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn remove_symbol_for_id(
        &mut self,
        name: &str,
        scope_id: SCOPEID,
    ) -> Result<(), SymbolError<SCOPEID, SYMID>> {
        let node_id = self.get_node_id_from_scope_id(scope_id)?;

        if let Some(ref mut node_mut) = self.tree.get_mut(node_id) {
            node_mut.value().remove_symbol(name)
        } else {
            panic!("Change to an error")
        }
    }

    pub fn create_symbols_in_scope(
        &mut self,
        scope_id: SCOPEID,
        names: &[String],
    ) -> Result<ThinVec<SymbolScopeId<SCOPEID, SYMID>>, SymbolError<SCOPEID, SYMID>> {
        let ret: Result<ThinVec<SymbolScopeId<SCOPEID, SYMID>>, SymbolError<SCOPEID, SYMID>> =
            names
                .iter()
                .map(|name| self.create_symbol_in_scope(scope_id, name))
                .collect();
        ret
    }

    pub fn create_symbol_in_scope(
        &mut self,
        scope_id: SCOPEID,
        name: &str,
    ) -> Result<SymbolScopeId<SCOPEID, SYMID>, SymbolError<SCOPEID, SYMID>> {
        let mut node = self.get_node_mut_from_id(scope_id)?;
        let syms = node.value();
        let symbol_id = syms.create_symbol(name)?;
        let si = SymbolInfo::new(name, None, symbol_id, syms.get_scope_fqn_name());

        self.scope_id_to_symbol_info.insert(symbol_id, si);
        Ok(symbol_id)
    }

    pub fn set_value_for_id(
        &mut self,
        id: SymbolScopeId<SCOPEID, SYMID>,
        val: V,
    ) -> Result<(), SymbolError<SCOPEID, SYMID>> {
        let x = self
            .scope_id_to_symbol_info
            .get_mut(&id)
            .ok_or(SymbolError::NotFound)?;
        x.value = Some(val);
        Ok(())
    }

    pub fn get_root_scope_id(&self) -> SCOPEID {
        self.tree.root().value().get_scope_id()
    }

    pub fn scope_exists(&self, scope: SCOPEID) -> bool {
        self.get_node_from_id(scope).map(|n| n.value()).is_ok()
    }

    pub fn get_symbols_from_id(
        &self,
        scope_id: SCOPEID,
    ) -> Result<&SymbolTable<SCOPEID, SYMID>, SymbolError<SCOPEID, SYMID>> {
        self.get_node_from_id(scope_id).map(|n| n.value())
    }

    pub fn get_fqn_from_id(&self, scope_id: SCOPEID) -> String {
        let scope = self.get_symbols_from_id(scope_id).expect("Invalid scope");
        scope.get_scope_fqn_name().to_owned()
    }

    pub fn resolve_label(
        &self,
        name: &str,
        scope_id: SCOPEID,
        barrier: SymbolResolutionBarrier,
    ) -> Result<SymbolScopeId<SCOPEID, SYMID>, SymbolError<SCOPEID, SYMID>> {
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

    pub fn get_writer(&mut self, scope_id: SCOPEID) -> SymbolTreeWriter<SCOPEID, SYMID, V> {
        SymbolTreeWriter::new(self, scope_id)
    }

    pub fn get_root_writer(&mut self) -> SymbolTreeWriter<SCOPEID, SYMID, V> {
        SymbolTreeWriter::new(self, self.get_root_scope_id())
    }

    pub fn get_reader(&self, scope_id: SCOPEID) -> SymbolTreeReader<SCOPEID, SYMID, V> {
        SymbolTreeReader::new(self, scope_id)
    }

    pub fn get_root_reader(&self) -> SymbolTreeReader<SCOPEID, SYMID, V> {
        self.get_reader(self.get_root_scope_id())
    }

    pub fn get_symbol_info_from_id(
        &self,
        symbol_id: SymbolScopeId<SCOPEID, SYMID>,
    ) -> Result<&SymbolInfo<SCOPEID, SYMID, V>, SymbolError<SCOPEID, SYMID>> {
        self.scope_id_to_symbol_info
            .get(&symbol_id)
            .ok_or(SymbolError::InvalidId)
    }

    pub fn get_symbol_info_from_scoped_name(
        &self,
        name: &ScopedName,
    ) -> Result<&SymbolInfo<SCOPEID, SYMID, V>, SymbolError<SCOPEID, SYMID>> {
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

    pub fn get_symbol_info(
        &self,
        name: &str,
        scope_id: SCOPEID,
    ) -> Result<&SymbolInfo<SCOPEID, SYMID, V>, SymbolError<SCOPEID, SYMID>> {
        let n = self.get_symbols_from_id(scope_id)?;
        let id = n.get_symbol_id(name)?;
        self.scope_id_to_symbol_info
            .get(&id)
            .ok_or(SymbolError::NotFound)
    }

    pub fn add_reference_symbol(
        &mut self,
        name: &str,
        scope_id: SCOPEID,
        symbol_id: SymbolScopeId<SCOPEID, SYMID>,
    ) -> Result<(), SymbolError<SCOPEID, SYMID>> {
        let mut node = self.get_node_from_id_mut(scope_id).expect("Invalid scope");
        node.value().add_reference_symbol(name, symbol_id)
    }

    pub fn create_or_get_scope_for_parent(&mut self, name: &str, id: SCOPEID) -> SCOPEID {
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
        symbol_id: SymbolScopeId<SCOPEID, SYMID>,
        val: V,
    ) -> Result<(), SymbolError<SCOPEID, SYMID>> {
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
impl<SCOPEID, SYMID, V> SymbolTree<SCOPEID, SYMID, V>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
{
    pub(crate) fn get_node_mut_from_id(
        &mut self,
        scope_id: SCOPEID,
    ) -> Result<SymbolNodeMut<SCOPEID, SYMID>, SymbolError<SCOPEID, SYMID>> {
        let node_id = self.get_node_id_from_scope_id(scope_id)?;
        self.tree.get_mut(node_id).ok_or(SymbolError::InvalidScope)
    }

    pub(crate) fn get_next_scope_id(&mut self) -> SCOPEID {
        let ret = self.next_scope_id;
        self.next_scope_id += 1;
        ret.into()
    }

    fn get_scope_node_id_from_id(
        &self,
        scope_id: SCOPEID,
    ) -> Result<SymbolNodeId, SymbolError<SCOPEID, SYMID>> {
        self.scope_id_to_node_id
            .get(&scope_id)
            .cloned()
            .ok_or(SymbolError::InvalidScope)
    }

    pub(crate) fn get_node_id_from_scope_id(
        &self,
        scope_id: SCOPEID,
    ) -> Result<SymbolNodeId, SymbolError<SCOPEID, SYMID>> {
        self.scope_id_to_node_id
            .get(&scope_id)
            .cloned()
            .ok_or(SymbolError::InvalidScope)
    }

    pub(crate) fn get_node_from_id(
        &self,
        scope_id: SCOPEID,
    ) -> Result<SymbolNodeRef<SCOPEID, SYMID>, SymbolError<SCOPEID, SYMID>> {
        let node_id = self.get_node_id_from_scope_id(scope_id)?;
        self.tree.get(node_id).ok_or(SymbolError::InvalidScope)
    }

    pub(crate) fn get_node_from_id_mut(
        &mut self,
        scope_id: SCOPEID,
    ) -> Result<SymbolNodeMut<SCOPEID, SYMID>, SymbolError<SCOPEID, SYMID>> {
        let node_id = self.get_node_id_from_scope_id(scope_id)?;
        self.tree.get_mut(node_id).ok_or(SymbolError::InvalidScope)
    }

    pub(crate) fn insert_new_table(
        &mut self,
        name: &str,
        parent_id: SCOPEID,
        barrier: SymbolResolutionBarrier,
    ) -> SCOPEID {
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
        parent_id: SCOPEID,
        barrier: SymbolResolutionBarrier,
    ) -> SymbolTable<SCOPEID, SYMID> {
        let parent_fqn = self.get_fqn_from_id(parent_id);
        let fqn = format!("{parent_fqn}::{name}");
        let scope_id = self.get_next_scope_id();
        SymbolTable::new(name, &fqn, scope_id, barrier)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Functions to do with serialization
fn walk_syms<F, SCOPEID, SYMID, V>(_node: SymbolNodeRef<SCOPEID, SYMID>, _scope: String, _f: &mut F)
where
    F: FnMut(&SymbolInfo<SCOPEID, SYMID, V>),
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
{
    // for _info in node.value().get_symbols() {
    //     panic!()
    //     // f(info)
    // }

    // for n in node.children() {
    //     let scope = format!("{scope}::{}", n.value().get_scope_name());
    //     walk_syms(n, scope, f);
    // }
}

pub fn print_syms<SCOPEID, SYMID, V>(node: SymbolNodeRef<SCOPEID, SYMID>, scope: String)
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
    V: std::fmt::Debug,
{
    walk_syms(node, scope, &mut |sym: &SymbolInfo<SCOPEID, SYMID, V>| {
        println!("{} = {:?}", sym.scoped_name(), sym.value)
    })
}

pub fn display_tree<SCOPEID, SYMID, V>(
    _out: &mut std::fmt::Formatter<'_>,
    _node: SymbolNodeRef<SCOPEID, SYMID>,
    _depth: usize,
) -> Result<(), std::fmt::Error>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
{
    panic!()
}

impl<SCOPEID, SYMID, V> std::fmt::Display for SymbolTree<SCOPEID, SYMID, V>
where
    SCOPEID: ScopeIdTraits + std::fmt::Debug,
    SYMID: SymIdTraits + std::fmt::Debug,
    V: std::fmt::Debug,
{
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        display_tree::<SCOPEID, SYMID, V>(out, self.tree.root(), 0)
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
