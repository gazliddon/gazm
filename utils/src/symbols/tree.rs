use std::collections::HashMap;

use super::ScopePath;
use super::SymbolTable;
use super::{IdTraits, SymbolErrorKind, ValueTraits};
use super::{SymbolReader, SymbolResult, SymbolWriter};

pub type ScopeRef<'a, V, ID> = ego_tree::NodeRef<'a, SymbolTable<V, ID>>;
pub type ScopeMut<'a, V, ID> = ego_tree::NodeMut<'a, SymbolTable<V, ID>>;
pub type ScopeId = ego_tree::NodeId;



// Indices of a symbol in the tree
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolId<ID: IdTraits> {
    scope_id: ScopeId,
    symbol_id: ID,
}

#[derive(Debug)]
pub struct SymbolInfo<'a, V: ValueTraits> {
    name: &'a str,
    value: &'a V,
}

#[derive(Debug)]
pub struct ScopeNode<V: ValueTraits, ID: IdTraits> {
    symbols: SymbolTable<V, ID>,
    aliases: HashMap<String, ScopeId>,
}

#[derive(Debug)]
pub struct Scopes<V: ValueTraits, ID: IdTraits> {
    scopes: ego_tree::Tree<SymbolTable<V, ID>>,
    root_id: ego_tree::NodeId,
}


impl<V: ValueTraits, ID: IdTraits> Default for Scopes<V, ID> {
    fn default() -> Self {
        Self::new()
    }
}

fn find_sub_scope<V: ValueTraits, ID: IdTraits>(n: ScopeRef<V, ID>, name: &str) -> Option<ScopeId> {
    for x in n.children() {
        if x.value().get_scope_name() == name {
            return Some(x.id());
        }
    }

    None
}


impl<V: ValueTraits, ID: IdTraits> std::fmt::Display for Scopes<V, ID> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.write(f)
    }
}

impl<V: ValueTraits, ID: IdTraits> Scopes<V, ID> {

    fn write_lo(&self, id : ScopeId, indent : usize, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        let sc = self.scopes.get(id).unwrap();

        let pad = " ".repeat(indent * 2);

        std::writeln!(f, "{}{}", pad, sc.value().get_scope_name())?;

        for c in sc.children() {
            self.write_lo(c.id(), indent + 1, f)?;
        }
        Ok(())
    }

    pub fn write(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { 
        self.write_lo(self.root_id, 0, f)
    }

    pub fn create_scope<F: Into<ScopePath>>(
        &mut self,
        parent: F,
        scope_name: &str,
    ) -> SymbolResult<ID, ScopeId> {
        let base: ScopePath = parent.into();
        let id = self.get_scope_id_from_fqn(&base)?;

        self.new_scope(id, scope_name)
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

    pub fn scope_rel_to_abs(&self, _base: &ScopePath, _rel: &ScopePath) -> Option<ScopePath> {
        panic!()
    }

    pub fn get_scope_id_from_fqn(&mut self, base: &ScopePath) -> SymbolResult<ID, ScopeId> {
        if base.is_relative() {
            Err(SymbolErrorKind::AbsPathNeeded)
        } else {
            let mut n = self.root_cursor();
            n.navigate_relative(base.get_parts())
        }
    }

    pub fn get_symbol_info<'a>(&'a self, id: &SymbolId<ID>) -> Option<SymbolInfo<'a, V>> {
        let scope_ref = self.scopes.get(id.scope_id).unwrap();
        let name = scope_ref.value().get_symbol_name(&id.symbol_id).unwrap();

        scope_ref
            .value()
            .get_symbol(id.symbol_id)
            .map(|s| SymbolInfo { name, value: s })
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

    pub fn new_scope(&mut self, parent: ScopeId, name: &str) -> SymbolResult<ID, ScopeId> {
        let mut scope = self.scopes.get_mut(parent).expect("!");
        let id = scope.append(SymbolTable::new(name)).id();
        Ok(id)
    }

    pub fn cursor(&mut self, s: ScopeId) -> ScopeCursor<V, ID> {
        ScopeCursor {
            scopes: self,
            current_scope: s,
        }
    }

    pub fn root_cursor(&mut self) -> ScopeCursor<V, ID> {
        self.cursor(self.scopes.root().id())
    }

    pub fn get_symbol_id(&self, _name: &str) -> Option<SymbolId<ID>> {
        panic!()
    }

    pub fn add_symbol_named(&mut self, _fqn: &str, _val: V) -> SymbolResult<ScopeId, SymbolId<ID>> {
        panic!()
    }


    pub fn find_scope_rel<B: Into<ScopePath>,P: Into<ScopePath>>(&self, base: B, path: P) -> SymbolResult<ID, ScopeId> {
        let base : ScopePath = base.into();
        let path : ScopePath = path.into();

        if base.is_relative() { 
            Err(SymbolErrorKind::AbsPathNeeded)
            
        } else if path.is_abs() {
            Err(SymbolErrorKind::RelPathNeeded)
        } else {

            let abs = ScopePath::from_base_path(&base, &path).unwrap();
            self.find_scope_abs(abs)

        }
    }

    pub fn find_scope_abs<X: Into<ScopePath>>(&self, path: X) -> SymbolResult<ID, ScopeId> {
        let path : ScopePath = path.into();

        if path.is_relative() {
            Err(SymbolErrorKind::AbsPathNeeded)
        } else {
        let parts = path.get_parts();

        let mut root_id = self.root_id;

        for part in parts {
            let scope = self.get(root_id).ok_or(SymbolErrorKind::NotFound)?;
            root_id = scope.children().find(|x| x.value().get_scope_name() == part).map(|z| z.id()).ok_or(SymbolErrorKind::NotFound)?;
        }

        Ok(root_id)
        }
    }
}

pub struct ScopeCursor<'a, V: ValueTraits, ID: IdTraits> {
    scopes: &'a mut Scopes<V, ID>,
    current_scope: ScopeId,
}

impl<'a, V: ValueTraits, ID: IdTraits> ScopeCursor<'a, V, ID> {
    pub fn go_root(&mut self) {
        self.current_scope = self.scopes.root_id;
    }

    pub fn go_up(&mut self) {
        if let Some(id) = self.get_parent() {
            self.current_scope = id;
        }
    }

    pub fn get_current_scope(&self) -> ScopeId {
        self.current_scope
    }

    pub fn get_current_scope_node(&self) -> ScopeRef<V, ID> {
        let id = self.get_current_scope();
        self.scopes.get(id).unwrap()
    }

    pub fn get_current_scope_node_mut(&mut self) -> ScopeMut<V, ID> {
        let id = self.get_current_scope();
        self.scopes.get_mut(id).unwrap()
    }

    pub fn get_current_scope_fqn(&self) -> String {
        self.scopes.get_scope_abs_fqn(self.current_scope).unwrap()
    }

    pub fn change_scope_by_name(&mut self, _name: &str) -> SymbolResult<ID, ScopeId> {
        todo!("Todo")
    }

    pub fn get_parent(&mut self) -> Option<ScopeId> {
        self.get_current_scope_node().parent().map(|x| x.id())
    }

    fn navigate_relative_lo(&mut self, path: &[String]) -> SymbolResult<ID, ScopeId> {
        for name in path.iter() {
            let n = self.get_current_scope_node();

            let id = match name.as_str() {
                "super" => self.get_parent().ok_or(SymbolErrorKind::NotFound)?,
                _ => find_sub_scope(n, name).ok_or(SymbolErrorKind::NotFound)?,
            };

            self.current_scope = id;
        }

        Ok(self.get_current_scope())
    }

    pub fn navigate_relative(&mut self, path: &[String]) -> SymbolResult<ID, ScopeId> {
        let mut ret = self.scopes.root_cursor();
        ret.current_scope = self.current_scope;
        let id = ret.navigate_relative_lo(path)?;
        self.current_scope = id;
        Ok(id)
    }

    pub fn navigate_abs(&mut self, path: &[String]) -> SymbolResult<ID, ScopeId> {
        let mut ret = self.scopes.root_cursor();
        let ret = ret.navigate_relative_lo(path)?;
        self.current_scope = ret;
        Ok(ret)
    }

    fn navigate(&mut self, fqn: &ScopePath) -> SymbolResult<ID, ScopeId> {
        if fqn.is_relative() {
            self.navigate_relative(fqn.get_parts())
        } else {
            self.navigate_abs(fqn.get_parts())
        }
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


////////////////////////////////////////////////////////////////////////////////
#[allow(unused_imports)]
mod test {

    use super::*;
    use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};

    type Symbols = Scopes<String, usize>;
    type SymbolId = crate::symbols::SymbolId<usize>;

    impl ValueTraits for String {}

    #[test]
    fn test_symbol_table() {
        let mut syms = Symbols::new();
        let _id = syms.create_scope("", "test").unwrap();
        let id = syms.create_scope("::test", "test2").unwrap();

        let found_id = syms.find_scope_abs("::test::test2").unwrap();
        let found_id_2 = syms.find_scope_rel("", "test::test2").unwrap();


        println!("id : {:?}", id);
        println!("found_id : {:?}", found_id);
        println!("found_id_2 : {:?}", found_id_2);

        let path = syms.get_scope_abs_fqn(id);
        println!("{:?}", path);

        println!("About to fail!!!!");

        assert!(false)
    }
}
