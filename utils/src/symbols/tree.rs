

use super::table::SymbolTable;
use super::{IdTraits, SymbolError, ValueTraits};
use super::SymbolReader;

pub type ScopeRef<'a, V, ID> = ego_tree::NodeRef<'a, SymbolTable<V, ID>>;
pub type ScopeMut<'a, V, ID> = ego_tree::NodeMut<'a, SymbolTable<V, ID>>;
pub type ScopeId = ego_tree::NodeId;

struct Scopes<V: ValueTraits, ID: IdTraits> {
    scopes: ego_tree::Tree<SymbolTable<V, ID>>,
    root_id: ego_tree::NodeId,
}

impl<V: ValueTraits, ID: IdTraits> Scopes<V, ID> {
    pub fn new() -> Self {
        let syms = SymbolTable::new("root");
        let scopes = ego_tree::Tree::new(syms);
        let root_id = scopes.root().id();
        Self { scopes, root_id }
    }

    pub fn get(&self, id: ScopeId) -> Option<ScopeRef<'_, V, ID>> {
        self.scopes.get(id)
    }
    pub fn get_mut(&mut self, id: ScopeId) -> Option<ScopeMut<'_, V, ID>> {
        self.scopes.get_mut(id)
    }

    fn get_scope_walker(&self, id: ScopeId) -> impl Iterator<Item = ScopeId> + '_ {
        let mut current_node = self.get(id);

        std::iter::from_fn(move || {
            let id = current_node?.id();
            current_node = current_node.unwrap().parent();
            Some(id)
        })
    }

    pub fn get_scope_abs_fqn(&self, id: ScopeId) -> Option<String> {
        let abs: Vec<_> = self
            .get_scope_abs(id)?
            .into_iter()
            .map(|id| self.get(id).unwrap().value().get_scope_name().to_owned())
            .collect();
        Some(abs.join("::"))
    }

    pub fn get_scope_abs(&self, id: ScopeId) -> Option<Vec<ScopeId>> {
        let walker = self.get_scope_walker(id);
        let mut ret: Vec<_> = walker.collect();
        ret.reverse();
        Some(ret)
    }

    pub fn new_scope(&mut self, parent: ScopeId, name : &str) -> Result<ScopeId,SymbolError<ScopeId>> {
        let mut scope = self.scopes.get_mut(parent).expect("!");
        let id = scope.append(SymbolTable::new(name)).id();
        Ok(id)
    }
}

struct ScopeCursor<'a, V: ValueTraits, ID: IdTraits> {
    scopes: &'a mut Scopes<V, ID>,
    current_scope: ScopeId,
}

#[derive(Debug, PartialEq, Clone)]
struct ScopedSymbolId<ID: IdTraits> {
    pub scope: ScopeId,
    pub symbol: ID,
}

impl<'a, V: ValueTraits, ID: IdTraits> ScopeCursor<'a, V, ID> {
    pub fn get_current_scope_id(&self) -> ScopeId {
        self.current_scope
    }

    pub fn get_current_scope(&self) -> ScopeRef<V, ID> {
        let id = self.get_current_scope_id();
        self.scopes.get(id).unwrap()
    }

    pub fn get_current_scope_fqn(&self) -> String {
        self.scopes.get_scope_abs_fqn(self.current_scope).unwrap()
    }

    pub fn change_scope_by_name(&mut self, _name: &str) -> Result<ScopeId, String> {
        Err("whoops".to_owned())
    }

    fn navigate_relative(&mut self, _path: &[String]) -> Result<ScopeId, String> {
        panic!();
    }

    fn navigate_abs(&mut self, path: &[String]) -> Result<ScopeId, String> {
        // Save current_scope
        let old_scope = self.current_scope;
        self.current_scope = self.scopes.root_id;
        let res = self.navigate_relative(path);

        match res {
            Ok(_) => res,
            Err(_) => {
                // restore current scope if there was an error
                self.current_scope = old_scope;
                res
            }
        }
    }
}

impl<'a, V: ValueTraits, ID: IdTraits> SymbolReader<V, ID> for ScopeCursor<'a, V, ID> {
    fn get_symbol_id(&self, name: &str) -> Option<ID> {
        let scope = self.scopes.get(self.current_scope)?;
        scope.value().get_symbol_id(name)
    }
    fn get_symbol(&self, id: ID) -> Option<&V> {
        let scope = self.scopes.get(self.current_scope)?;
        scope.value().get_symbol(id)
    }
}
