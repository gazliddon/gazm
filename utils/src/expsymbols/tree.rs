use super::ScopeCursor;
use super::ScopeErrorKind;
use super::ScopePath;
use super::ScopeResult;
use super::SymbolReader;
use super::SymbolTable;
use super::{IdTraits, ValueTraits};
use std::collections::HashMap;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub enum ScopeKind {
    #[default]
    File,
    Function,
    Block,
}

pub type ScopeRef<'a, V, ID> = ego_tree::NodeRef<'a, SymbolTable<V, ID>>;
pub type ScopeMut<'a, V, ID> = ego_tree::NodeMut<'a, SymbolTable<V, ID>>;
pub type ScopeId = ego_tree::NodeId;

// Indices of a symbol in the tree
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolId<ID: IdTraits> {
    pub(crate) scope_id: ScopeId,
    pub(crate) symbol_id: ID,
}

#[derive(Debug, Clone)]
pub struct SymbolInfo<'a, V: ValueTraits, ID: IdTraits> {
    name: &'a str,
    value: &'a V,
    id: SymbolId<ID>,
}

#[derive(Debug)]
pub struct ScopeNode<V: ValueTraits, ID: IdTraits> {
    pub symbols: SymbolTable<V, ID>,
    pub aliases: HashMap<String, ScopeId>,
    pub name: String,
}

impl<V: ValueTraits, ID: IdTraits> ScopeNode<V, ID> {
    pub fn new(name: &str) -> Self {
        Self {
            symbols: Default::default(),
            aliases: Default::default(),
            name: name.to_owned(),
        }
    }
}

#[derive(Debug)]
pub struct Scopes<V: ValueTraits, ID: IdTraits> {
    scopes: ego_tree::Tree<SymbolTable<V, ID>>,
    pub(crate) root_id: ego_tree::NodeId,
}

impl<V: ValueTraits, ID: IdTraits> Default for Scopes<V, ID> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: ValueTraits, ID: IdTraits> std::fmt::Display for Scopes<V, ID> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.write(f)
    }
}

impl<V: ValueTraits, ID: IdTraits> Scopes<V, ID> {
    fn write_lo(
        &self,
        id: ScopeId,
        indent: usize,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        let sc = self.scopes.get(id).unwrap();

        let pad = " ".repeat(indent * 2);

        let name = sc.value().get_scope_name();
        let name = match name {
            "" => "root",
            _ => name,
        };

        writeln!(f, "{pad}{name}")?;

        for c in sc.children() {
            self.write_lo(c.id(), indent + 1, f)?;
        }
        Ok(())
    }

    pub fn root(&self) -> ScopeId {
        self.root_id
    }

    fn write(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.write_lo(self.root_id, 0, f)
    }

    pub fn create_scope<F: Into<ScopePath>>(
        &mut self,
        parent: F,
        scope_name: &str,
    ) -> ScopeResult<ScopeId> {
        let parent: ScopePath = parent.into();
        let id = self.find_scope_abs(parent)?;
        self.new_scope(id, scope_name)
    }

    pub fn create_scope_recursive<F: Into<ScopePath>>(&mut self, path: F) -> ScopeResult<ScopeId> {
        let path: ScopePath = path.into();

        let scope_id = self.find_scope_abs(path.clone());

        if path.is_relative() {
            Err(ScopeErrorKind::AbsPathNeeded)
        } else if scope_id.is_ok() {
            Ok(scope_id.unwrap())
        } else {
            let mut id = self.root_id;

            for part in path.get_rel_parts() {
                let scope = self.get(id).unwrap();

                if let Some(node) = scope
                    .children()
                    .find(|x| x.value().get_scope_name() == part)
                {
                    id = node.id()
                } else {
                    let new_scope = self.new_scope(scope.id(), part)?;
                    id = new_scope
                }
            }

            Ok(id)
        }
    }

    pub fn new() -> Self {
        let syms = SymbolTable::new("");
        let scopes = ego_tree::Tree::new(syms);
        let root_id = scopes.root().id();
        Self { scopes, root_id }
    }

    pub fn get(&self, id: ScopeId) -> Option<ScopeRef<'_, V, ID>> {
        self.scopes.get(id)
    }

    pub fn get_symbol_info<'a>(&'a self, id: &SymbolId<ID>) -> Option<SymbolInfo<'a, V, ID>> {
        let scope_ref = self.scopes.get(id.scope_id).unwrap();
        let name = scope_ref.value().get_symbol_name(&id.symbol_id).unwrap();

        scope_ref
            .value()
            .get_symbol(id.symbol_id)
            .map(|s| SymbolInfo {
                name,
                value: s,
                id: id.clone(),
            })
    }

    pub fn get_mut(&mut self, id: ScopeId) -> Option<ScopeMut<'_, V, ID>> {
        self.scopes.get_mut(id)
    }

    pub fn get_scope_parent_walker(&self, id: ScopeId) -> impl Iterator<Item = ScopeId> + '_ {
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
        let walker = self.get_scope_parent_walker(id);
        let mut ret: Vec<_> = walker.collect();
        ret.reverse();
        Some(ret)
    }

    pub fn new_scope(&mut self, parent: ScopeId, name: &str) -> ScopeResult<ScopeId> {
        let mut scope = self
            .scopes
            .get_mut(parent)
            .ok_or(ScopeErrorKind::InvalidScopeId)?;
        let id = scope.append(SymbolTable::new(name)).id();
        Ok(id)
    }

    ////////////////////////////////////////////////////////////////////////////////
    // Cursor func
    pub fn cursor(&mut self, s: ScopeId) -> ScopeCursor<V, ID> {
        ScopeCursor::new(self, s)
    }

    pub fn cursor_from_path<P: Into<ScopePath>>(
        &mut self,
        path: P,
    ) -> ScopeResult<ScopeCursor<V, ID>> {
        let id = self.find_scope_abs(path)?;
        Ok(self.cursor(id))
    }

    pub fn root_cursor(&mut self) -> ScopeCursor<V, ID> {
        self.cursor(self.scopes.root().id())
    }

    ////////////////////////////////////////////////////////////////////////////////
    // find scopes
    pub fn find_scope_rel_id<P: Into<ScopePath>>(
        &self,
        base_id: ScopeId,
        path: P,
    ) -> ScopeResult<ScopeId> {
        let path: ScopePath = path.into();
        self.navigate_to_sub_scope(base_id, &path)
    }

    pub fn find_scope_rel<B: Into<ScopePath>, P: Into<ScopePath>>(
        &self,
        base: B,
        path: P,
    ) -> ScopeResult<ScopeId> {
        let base: ScopePath = base.into();
        let base_id = self.find_scope_abs(base)?;
        self.navigate_to_sub_scope(base_id, &path.into())
    }

    fn navigate_to_sub_scope(
        &self,
        mut current_scope_id: ScopeId,
        path: &ScopePath,
    ) -> ScopeResult<ScopeId> {
        if path.is_abs() {
            Err(ScopeErrorKind::RelPathNeeded)
        } else {
            let parts = path.get_parts();

            for part in parts {
                let scope = self
                    .get(current_scope_id)
                    .ok_or(ScopeErrorKind::InvalidScopeId)?;

                current_scope_id = match part.as_str() {
                    "super" => scope.parent().ok_or(ScopeErrorKind::NoParent)?,
                    _ => scope
                        .children()
                        .find(|x| x.value().get_scope_name() == part)
                        .ok_or_else(|| {
                            println!("{self}");
                            for s in scope.children() {
                                println!("{}", s.value().get_scope_name())
                            }

                            let current = scope.value().get_scope_name();
                            let err = format!("part: {part}\ncurrent: {current}\nfull: {path}\n");
                            ScopeErrorKind::ScopeNotFound(err)
                        })?,
                }
                .id();
            }

            Ok(current_scope_id)
        }
    }

    pub fn find_scope_abs<X: Into<ScopePath>>(&self, path: X) -> ScopeResult<ScopeId> {
        let path: ScopePath = path.into();
        if path.is_root() {
            Ok(self.root_id)
        } else if path.is_relative() {
            Err(ScopeErrorKind::AbsPathNeeded)
        } else {
            let parts: Vec<_> = path.get_rel_parts().iter().map(|p| p.as_str()).collect();
            let rel_path = ScopePath::from_parts(parts);
            println!("{rel_path:#?}");
            self.navigate_to_sub_scope(self.root_id, &rel_path)
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
#[allow(unused_imports)]
mod test {

    use super::*;
    use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};

    type Symbols = Scopes<String, usize>;
    type SymbolId = super::SymbolId<usize>;
    impl ValueTraits for String {}

    #[test]
    fn test_symbol_recursive() {
        let mut syms = Symbols::new();
        let id = syms.create_scope_recursive("::test::test2").unwrap();
        println!("{}", syms);
        let id_2 = syms.find_scope_abs("::test::test2").unwrap();

        println!("{}", syms);

        assert_eq!(id_2, id);
    }

    fn test_scope(syms: &Symbols, name: &str) {
        let id = syms.find_scope_abs(name).unwrap();
        let path = syms.get_scope_abs_fqn(id).unwrap();
        println!("{path}");
        assert_eq!(path, name);
    }

    #[test]
    fn test_fqn_retrieval() {}

    #[test]
    fn test_symbol_table() {
        let mut syms = Symbols::new();

        syms.create_scope("", "test").unwrap();

        let id = syms.create_scope("::test", "test2").unwrap();
        syms.create_scope("::test", "test3").unwrap();

        let found_id = syms.find_scope_abs("::test::test2").unwrap();
        let found_id_2 = syms.find_scope_rel("", "test::test2").unwrap();
        let found_id_3 = syms.find_scope_rel("::test", "test2").unwrap();

        assert_eq!(found_id, found_id_2);
        assert_eq!(id, found_id_2);
        assert_eq!(id, found_id_3);

        let path = syms.get_scope_abs_fqn(id).unwrap();
        assert_eq!(path, "::test::test2");

        test_scope(&syms, "::test::test2");
        test_scope(&syms, "::test::test3");
        test_scope(&syms, "");
    }
}
