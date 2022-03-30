use super::{Position, SymbolError, SymbolInfo, SymbolQuery, SymbolWriter};

use std::{collections::HashMap, fmt::Display};

mod new {
    use std::collections::HashMap;

    #[derive(Debug, PartialEq, Clone)]
    pub enum SymbolError {
        AlreadyDefined(SymbolId),
        Mismatch { expected: i64 },
        NotFound,
    }

    pub trait SymbolQuery<V, ID> {
        fn get_symbol_id(&self, name: &str) -> Option<ID>;
        fn get_symbol(&self, id: ID) -> Option<&V>;

        fn get_symbol_from_name(&self, name: &str) -> Option<&V> {
            self.get_symbol_id(name).and_then(|id| self.get_symbol(id))
        }

        fn symbol_exists_from_name(&self, name: &str) -> bool {
            self.get_symbol_from_name(name).is_some()
        }
    }

    pub trait SymbolWriter<V: Default, ID>: SymbolQuery<V, ID> {
        fn add_symbol_with_value(&mut self, name: &str, value: V) -> Result<ID, SymbolError>;
        fn remove_symbol(&mut self, id: ID) -> Result<(), SymbolError>;
        fn get_symbol_mut(&mut self, id: ID) -> Result<&mut V, SymbolError>;

        fn add_symbol(&mut self, name: &str) -> Result<ID, SymbolError> {
            self.add_symbol_with_value(name, V::default())
        }

        fn remove_symbol_name(&mut self, name: &str) -> Result<(), SymbolError> {
            let id = self.get_symbol_id(name).ok_or(SymbolError::NotFound)?;
            self.remove_symbol(id)
        }
    }

    #[derive(Debug, PartialEq, Hash, Eq, Clone, Copy)]
    pub struct SymbolId(u64);

    impl From<u64> for SymbolId {
        fn from(v: u64) -> Self {
            SymbolId(v)
        }
    }

    pub struct SymbolTable<V> {
        scope: String,
        name_to_id: HashMap<String, SymbolId>,
        id_to_name: HashMap<SymbolId, String>,
        pub id_to_value: HashMap<SymbolId, V>,
        id: u64,
    }

    impl<T> SymbolQuery<T, SymbolId> for SymbolTable<T> {
        fn get_symbol_id(&self, name: &str) -> Option<SymbolId> {
            self.name_to_id.get(name).cloned()
        }

        fn get_symbol(&self, id: SymbolId) -> Option<&T> {
            self.id_to_value.get(&id)
        }
    }

    impl<V: Default> SymbolWriter<V, SymbolId> for SymbolTable<V> {
        fn add_symbol_with_value(&mut self, name: &str, value: V) -> Result<SymbolId, SymbolError> {
            if let Some(id) = self.get_symbol_id(name) {
                Err(SymbolError::AlreadyDefined(id))
            } else {
                let id = self.get_next_id();
                self.name_to_id.insert(name.to_string(), id);
                self.id_to_value.insert(id, value);
                self.id_to_name.insert(id, name.to_string());
                Ok(id)
            }
        }

        fn remove_symbol(&mut self, id: SymbolId) -> Result<(), SymbolError> {
            self.get_symbol(id).ok_or(SymbolError::NotFound)?;
            let name = self.id_to_name.get(&id).unwrap();
            self.name_to_id.remove(name);
            self.id_to_value.remove(&id);
            self.id_to_name.remove(&id);
            Ok(())
        }

        fn get_symbol_mut(&mut self, id: SymbolId) -> Result<&mut V, SymbolError> {
            self.id_to_value.get_mut(&id).ok_or(SymbolError::NotFound)
        }
    }

    impl<T> Default for SymbolTable<T> {
        fn default() -> Self {
            Self::new("no name")
        }
    }

    impl<V> SymbolTable<V> {
        pub fn get_scope_name(&self) -> &str {
            &self.scope
        }

        pub fn new(name: &str) -> Self {
            Self {
                scope: name.to_string(),
                id: 1,
                name_to_id: Default::default(),
                id_to_name: Default::default(),
                id_to_value: Default::default(),
            }
        }

        fn get_next_id(&mut self) -> SymbolId {
            let ret = self.id;
            self.id += 1;
            ret.into()
        }
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub type ScopeId = ego_tree::NodeId;
    pub type ScopeRef<'a, V> = ego_tree::NodeRef<'a, SymbolTable<V>>;
    pub type ScopeRefMut<'a, V> = ego_tree::NodeMut<'a, SymbolTable<V>>;

    struct Scopes<V> {
        scopes: ego_tree::Tree<SymbolTable<V>>,
        root_id: ScopeId,
    }

    enum ScopePath {
        Abs(Vec<String>),
        ThisScope(String),
        Relative(Vec<String>),
    }

    impl ScopePath {
        pub fn new(name: &str) -> Self {
            let split: Vec<_> = name.split("::").collect();
            if split.len() == 1 {
                // is this a scoped name?
                // no, just ask current scope
                ScopePath::ThisScope(split[0].to_owned())
            } else {
                let split: Vec<_> = split.into_iter().map(|s| s.to_owned()).collect();
                if split[0] == "root" || split[0] == "" {
                    ScopePath::Abs(split)
                } else {
                    ScopePath::Relative(split)
                }
            }
        }
    }

    impl<V> Scopes<V> {
        pub fn new() -> Self {
            let syms = SymbolTable::new("root");
            let scopes = ego_tree::Tree::new(syms);
            let root_id = scopes.root().id();
            Self { scopes, root_id }
        }

        pub fn get(&self, id: ScopeId) -> Option<ScopeRef<'_, V>> {
            self.scopes.get(id)
        }

        fn get_scope_walker(&self, id: ScopeId) -> impl Iterator<Item = ScopeId> + '_ {
            let mut current_node = self.get(id);

            let walker = std::iter::from_fn(move || {
                let id = current_node?.id();
                current_node = current_node.unwrap().parent();
                Some(id)
            });
            walker
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
    }

    ////////////////////////////////////////////////////////////////////////////////

    struct ScopeNavigator<'a, V> {
        scopes: &'a mut Scopes<V>,
        current_scope: ScopeId,
    }

    #[derive(Debug, PartialEq, Clone)]
    struct ScopedSymbolId {
        pub scope: ScopeId,
        pub symbol: SymbolId,
    }

    impl<'a, V> ScopeNavigator<'a, V> {
        pub fn get_current_scope_id(&self) -> ScopeId {
            self.current_scope
        }

        pub fn get_current_scope(&self) -> ScopeRef<V> {
            let id = self.get_current_scope_id();
            self.scopes.get(id).unwrap()
        }

        pub fn get_current_scope_fqn(&self) -> String {
            self.scopes.get_scope_abs_fqn(self.current_scope).unwrap()
        }

        pub fn change_scope_by_name(&mut self, _name: &str) -> Result<ScopeId, String> {
            Err("whoops".to_owned())
        }

        fn navigate_relative(&mut self, _path : &Vec<String> ) -> Result<ScopeId, String> {
            panic!();
        }

        fn navigate_abs(&mut self, path : &Vec<String>) -> Result<ScopeId,String> {
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

    impl<'a, V> SymbolQuery<V, ScopedSymbolId> for ScopeNavigator<'a, V> {
        fn get_symbol_id(&self, name: &str) -> Option<ScopedSymbolId> {

            let scope_path = ScopePath::new(name);

            match &scope_path {
                ScopePath::Abs(_) => (),
                ScopePath::ThisScope(_) => (),
                ScopePath::Relative(_) => ()
            };

            panic!();
        }

        fn get_symbol(&self, id: ScopedSymbolId) -> Option<&V> {
            self.scopes
                .get(id.scope)
                .and_then(|s| s.value().get_symbol(id.symbol))
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Holds information about symbols
#[derive(Debug, PartialEq, Clone)]
pub struct SymbolTable {
    scope: String,
    pub info: HashMap<u64, SymbolInfo>,
    name_to_id: HashMap<String, u64>,
    ref_name_to_value: HashMap<String, i64>,
    id: u64,
}

pub enum SymbolKind {
    Undefined,
    Number,
    MacroDefinition { node: u64 },
}

struct Symbol {
    kind: SymbolKind,
    pos: Position,
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}
impl Display for SymbolTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Scope: {}",self.scope)?;

        for (name, id) in &self.name_to_id {
            let val = self.info.get(&id).unwrap();
            match &val.value {
                Some(val) => writeln!(f,"{name} = {:04X} ({})",val,val)?,
                _ => writeln!(f, "{name} = undefined", )?,
            }
        }
        Ok(())
    }
}

impl SymbolQuery for SymbolTable {
    fn get_symbol_info(&self, name: &str) -> Result<&SymbolInfo, SymbolError> {
        let id = self.name_to_id.get(name).ok_or(SymbolError::NotFound)?;
        self.get(*id)
    }
}

impl SymbolWriter for SymbolTable {
    fn add_symbol_with_value(&mut self, name: &str, value: i64) -> Result<u64, SymbolError> {
        let nstr: String = name.into();
        let id = self.add_symbol(&nstr)?;
        self.set_value(id, value)?;

        if let Some(expected) = self.ref_name_to_value.get(&nstr) {
            if *expected != value {
                return Err(SymbolError::Mismatch {
                    expected: *expected,
                });
            }
        }

        Ok(id)
    }

    fn remove_symbol_name(&mut self, name: &str) {
        if let Some(id) = self.name_to_id.get(name).cloned() {
            self.name_to_id.remove(name);
            self.info.remove(&id);
        }
    }

    fn add_symbol(&mut self, name: &str) -> Result<u64, SymbolError> {
        let name: String = name.into();

        if let Some(id) = self.name_to_id.get(&name) {
            Err(SymbolError::AlreadyDefined(*id))
        } else {
            let x_id = self.get_next_id();

            self.name_to_id.insert(name.clone(), x_id);

            let info = SymbolInfo { name, value: None };

            self.info.insert(x_id, info);
            Ok(x_id)
        }
    }

    fn add_reference_symbol(&mut self, name: &str, val: i64) {
        let res = self.ref_name_to_value.insert(name.to_string(), val);
        assert!(res.is_none());
    }
}

impl SymbolTable {
    pub fn new() -> Self {
        Self::new_with_scope("No Scope")
    }

    pub fn get_scope_name(&self) -> &str {
        &self.scope
    }

    pub fn new_with_scope(name: &str) -> Self {
        Self {
            scope: name.to_string(),
            info: Default::default(),
            name_to_id: Default::default(),
            ref_name_to_value: Default::default(),
            id: 1,
        }
    }

    fn get(&self, id: u64) -> Result<&SymbolInfo, SymbolError> {
        self.info.get(&id).ok_or(SymbolError::NotFound)
    }

    fn get_mut(&mut self, id: u64) -> Result<&mut SymbolInfo, SymbolError> {
        self.info.get_mut(&id).ok_or(SymbolError::NotFound)
    }

    fn symbol_exists(&self, id: u64) -> bool {
        self.get(id).is_ok()
    }

    fn set_value(&mut self, id: u64, value: i64) -> Result<(), SymbolError> {
        let i = self.get_mut(id)?;
        i.value = Some(value);
        Ok(())
    }

    fn get_next_id(&mut self) -> u64 {
        let ret = self.id;
        self.id += 1;
        ret
    }
}