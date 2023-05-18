use super::{
    IdTraits, ScopeErrorKind, ScopeId, ScopeMut, ScopePath, ScopeRef, ScopeResult, Scopes,
    ValueTraits, SymbolReader,SymbolId, SymbolWriter
};

pub struct ScopeCursor<'a, V: ValueTraits, ID: IdTraits> {
    scopes: &'a mut Scopes<V, ID>,
    current_scope: ScopeId,
}

impl<'a, V: ValueTraits, ID: IdTraits> ScopeCursor<'a, V, ID> {
    // Creation
    pub fn new(scopes: &'a mut Scopes<V, ID>, current_scope: ScopeId) -> Self {
        Self {
            scopes,
            current_scope,
        }
    }

    // Navigation
    pub fn go_root(self) -> Self {
        let id = self.scopes.root_id;
        self.go_id(id)
    }

    pub fn go_parent(self) -> Self {
        let id = self.get_parent_id().unwrap_or(self.current_scope);
        self.go_id(id)
    }

    pub fn go_id(self, id: ScopeId) -> Self {
        Self {
            current_scope: id,
            ..self
        }
    }

    pub fn go_path<P: Into<ScopePath>>(self, path: P) -> ScopeResult<Self> {
        let path: ScopePath = path.into();
        self.go_scope_path(&path)
    }

    pub fn go_scope_path(self, path: &ScopePath) -> ScopeResult<Self> {
        let new_id = if path.is_abs() {
            self.scopes.find_scope_abs(path.clone())
        } else {
            self.scopes.find_scope_rel_id(self.current_scope, path.clone())
        }?;

        Ok(Self {
            current_scope: new_id,
            scopes: self.scopes,
        })
    }

    // Queries
    pub fn get_current_scope_id(&self) -> ScopeId {
        self.current_scope
    }

    fn get_current_scope_node(&self) -> ScopeRef<V, ID> {
        let id = self.get_current_scope_id();
        self.scopes.get(id).unwrap()
    }

    fn get_current_scope_node_mut(&mut self) -> ScopeMut<V, ID> {
        let id = self.get_current_scope_id();
        self.scopes.get_mut(id).unwrap()
    }

    pub fn get_current_scope_fqn(&self) -> String {
        self.scopes.get_scope_abs_fqn(self.current_scope).unwrap()
    }

    pub fn get_parent_id(&self) -> ScopeResult<ScopeId> {
        self.get_current_scope_node()
            .parent()
            .map(|x| x.id())
            .ok_or(ScopeErrorKind::NoParent)
    }

    // Symbol shit
    pub fn remove_symbol(&mut self, name: &str) -> ScopeResult<()> {
        let mut x = self.get_current_scope_node_mut();
        x.value().remove_symbol_name(name)
    }

    pub fn add_symbol(&mut self, name: &str, value: V) -> ScopeResult<SymbolId<ID>> {
        let mut x = self.get_current_scope_node_mut();

        let symbol_id = x
            .value()
            .add_symbol_with_value(name, value)?;

        Ok(SymbolId {
            symbol_id,
            scope_id: self.get_current_scope_id(),
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

    type Symbols = super::Scopes<String, usize>;
    type SymbolId = super::SymbolId<usize>;

    #[test]
    fn test_cursor() {
        let mut syms = Symbols::new();

        syms.create_scope_recursive("::test::foo").unwrap();
        let test_bar_id = syms.create_scope_recursive("::test::bar").unwrap();
        syms.create_scope_recursive("::test::bar::gaz").unwrap();
        syms.create_scope_recursive("::main").unwrap();

        println!("{syms}");

        let c = syms.cursor_from_path("::test::bar").unwrap();

        assert_eq!(c.get_current_scope_id(), test_bar_id);

        let c = c.go_path("super").unwrap();

        let new_test_id = c.get_current_scope_id();
        let test_id = syms.find_scope_abs("::test").unwrap();

        assert_eq!(new_test_id, test_id);
    }
}