use super::{
    IdTraits, ScopeErrorKind, ScopeId, ScopeMut, ScopeRef, ScopeResult, Scopes, ValueTraits, ScopePath,
};

use super::{ SymbolWriter, SymbolId, SymbolResult };


use crate::symbols::SymbolReader;

pub struct ScopeCursor<'a, V: ValueTraits, ID: IdTraits> {
    scopes: &'a mut Scopes<V, ID>,
    current_scope: ScopeId,
}

impl<'a, V: ValueTraits, ID: IdTraits> ScopeCursor<'a, V, ID> {
    pub fn new(scopes: &'a mut Scopes<V,ID>, current_scope: ScopeId) -> Self {
        Self {
            scopes, current_scope
        }
    }

    pub fn go_root(&mut self) {
        self.current_scope = self.scopes.root_id;
    }

    pub fn go_parent(&mut self) {
        if let Ok(id) = self.get_parent() {
            self.current_scope = id;
        }
    }

    pub fn get_current_scope(&self) -> ScopeId {
        self.current_scope
    }

    fn get_current_scope_node(&self) -> ScopeRef<V, ID> {
        let id = self.get_current_scope();
        self.scopes.get(id).unwrap()
    }

    fn get_current_scope_node_mut(&mut self) -> ScopeMut<V, ID> {
        let id = self.get_current_scope();
        self.scopes.get_mut(id).unwrap()
    }

    pub fn get_current_scope_fqn(&self) -> String {
        self.scopes.get_scope_abs_fqn(self.current_scope).unwrap()
    }

    pub fn get_parent(&mut self) -> ScopeResult<ScopeId> {
        self.get_current_scope_node()
            .parent()
            .map(|x| x.id())
            .ok_or(ScopeErrorKind::NoParent)
    }

    pub fn go<P: Into<ScopePath>>(&mut self, path: P) -> ScopeResult<ScopeId> {
        let path: ScopePath = path.into();

        let new_id = if path.is_abs() {
            self.scopes.find_scope_abs(path)
        } else {
            self.scopes.find_scope_rel_id(self.current_scope, path)
        }?;
        self.current_scope = new_id;
        Ok(new_id)
    }

    pub fn remove_symbol(&mut self, name: &str) -> SymbolResult<ID, ()> {
        let mut x = self.get_current_scope_node_mut();
        x.value().remove_symbol_name(name)
    }

    pub fn add_symbol(&mut self, name: &str, value: V) -> SymbolResult<ID, SymbolId<ID>> {
        let mut x = self.get_current_scope_node_mut();

        let symbol_id = x
            .value()
            .add_symbol_with_value(name, value)
            .map_err(|e| e)?;

        Ok(SymbolId {
            symbol_id,
            scope_id: self.get_current_scope(),
        })
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


#[allow(unused_imports)]
mod test {

    use super::*;
    use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};

    type Symbols = Scopes<String, usize>;
    type SymbolId = crate::symbols::SymbolId<usize>;


    #[test]
    fn test_cursor() {
        let mut syms = Symbols::new();

        syms.create_scope_recursive("::test::foo").unwrap();
        let test_bar_id = syms.create_scope_recursive("::test::bar").unwrap();
        syms.create_scope_recursive("::test::bar::gaz").unwrap();
        syms.create_scope_recursive("::main").unwrap();

        println!("{syms}");

        let mut c = syms.cursor_from_path("::test::bar").unwrap();

        assert_eq!(c.get_current_scope(), test_bar_id);

        let new_test_id = c.go("super").unwrap();
        let test_id = syms.find_scope_abs("::test").unwrap();

        assert_eq!(new_test_id, test_id);
    }
}
