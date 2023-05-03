use std::collections::VecDeque;

pub type ScopeId = u64;

#[derive(Debug, PartialEq, Clone)]
pub struct ScopeEntry {
    pub name: String,
    pub predecessors: VecDeque<ScopeId>,
    pub scope_id: ScopeId,
    pub fqn: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct StackTree<T> {
    tree: ego_tree::Tree<T>,
    current_node: ego_tree::NodeId,
}

impl<T> StackTree<T> {
    pub fn new(root: T) -> Self {
        let tree = ego_tree::Tree::new(root);
        let id = tree.root().id();

        Self {
            tree,
            current_node: id,
        }
    }

    pub fn get_current_node(&self) -> ego_tree::NodeId {
        self.current_node
    }

    pub fn add(&mut self, name: T) -> ego_tree::NodeId {
        let mut node = self.tree.get_mut(self.current_node).unwrap();
        let ret = node.append(name);
        ret.id()
    }

    pub fn push(&mut self, name: T) -> ego_tree::NodeId {
        let id = self.add(name);
        self.current_node = id;
        id
    }

    pub fn pop(&mut self) -> ego_tree::NodeId {
        let mut top = self.tree.get_mut(self.current_node).unwrap();

        if let Some(p) = top.parent() {
            self.current_node = p.id();
        }

        self.get_current_node()
    }

    pub fn get_tree(&self) -> &ego_tree::Tree<T> {
        &self.tree
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct ScopeStore {
    scope_id: ScopeId,
    scopes: Vec<ScopeEntry>,
}

#[allow(dead_code)]
impl ScopeStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, id: ScopeId) -> Option<&ScopeEntry> {
        self.scopes.get(id as usize)
    }

    fn get_predecessors_fqn(&self, predecessors: &VecDeque<ScopeId>) -> String {
        predecessors
            .iter()
            .map(|id| self.get(*id).unwrap().name.clone())
            .collect::<Vec<_>>()
            .join("/")
    }

    pub fn add_new_scope(&mut self, name: &str, predecessors: &VecDeque<ScopeId>) -> &ScopeEntry {
        let scope_id = self.scopes.len() as ScopeId;

        let scope_entry = ScopeEntry {
            name: name.to_string(),
            scope_id,
            predecessors: predecessors.clone(),
            fqn: self.get_predecessors_fqn(predecessors),
        };
        self.scopes.push(scope_entry);
        self.get(scope_id).unwrap()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ScopeBuilder {
    scope_tree: StackTree<String>,
}
impl Default for ScopeBuilder {
    fn default() -> Self {
        let scope_tree = StackTree::new(String::new());
        Self { scope_tree }
        
    }
}

impl ScopeBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_current_scope(&self) -> ego_tree::NodeId {
        self.scope_tree.get_current_node()
    }

    pub fn push_new(&mut self, name: &str) -> ego_tree::NodeId {
        self.scope_tree.push(name.to_string())
    }

    pub fn pop(&mut self) -> ego_tree::NodeId {
        self.scope_tree.pop()
    }

    pub fn get_current_fqn(&self) -> String {
        self.get_fqn(self.get_current_scope())
    }

    pub fn get_fqn(&self, id: ego_tree::NodeId) -> String {
        let t = self.scope_tree.get_tree();

        let mut node = t.get(id).unwrap();

        let mut string = node.value().clone();

        while node.parent().is_some() {
            let parent = node.parent().unwrap();

            string = format!("{}/{}", parent.value(), string);
            node = parent;
        }
        string
    }
}
