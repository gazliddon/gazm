use std::collections::HashMap;

use crate::{
    ast::{iter_refs_recursive, AstNodeId, AstTree},
    gazmsymbols::{
        ScopeId, SymbolError, SymbolInfo, SymbolResolutionBarrier, SymbolScopeId, SymbolTree,
    },
    item::{Item, LabelDefinition},
};

use sources::Position;
use utils::Stack;

#[allow(dead_code)]
pub struct Navigator<'a> {
    syms: &'a SymbolTree,
    current_scope_id: ScopeId,
    scope_stack: Stack<ScopeId>,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NavError {
    UnableToPop,
    ScopeIdNotFound(ScopeId),
    SymbolError(SymbolError),
}

#[allow(dead_code)]
impl<'a> Navigator<'a> {
    pub fn new(syms: &'a SymbolTree) -> Self {
        Self {
            syms,
            current_scope_id: syms.get_root_scope_id(),
            scope_stack: Default::default(),
        }
    }

    pub fn new_with_id(syms: &'a SymbolTree, id: ScopeId) -> Result<Self, NavError> {
        let mut ret = Self::new(syms);
        ret.set_scope(id)?;
        Ok(ret)
    }

    fn check_scope(&self, scope: ScopeId) -> Result<(), NavError> {
        if self.syms.scope_exists(scope) {
            Ok(())
        } else {
            Err(NavError::SymbolError(SymbolError::NoValue))
        }
    }

    pub fn get_current_scope_id(&self) -> u64 {
        self.current_scope_id
    }

    pub fn resolve_label(&self, _name: &str) -> Result<SymbolScopeId, SymbolError> {
        self.syms.resolve_label(
            _name,
            self.current_scope_id,
            SymbolResolutionBarrier::default(),
        )
    }

    pub fn set_scope(&mut self, scope_id: u64) -> Result<(), NavError> {
        self.check_scope(scope_id)?;
        self.current_scope_id = scope_id;
        Ok(())
    }

    pub fn push_scope(&mut self, scope_id: u64) -> Result<(), NavError> {
        self.check_scope(scope_id)?;
        self.scope_stack.push(scope_id);
        Ok(())
    }

    pub fn pop_scope(&mut self) -> Result<u64, NavError> {
        self.scope_stack.pop().ok_or(NavError::UnableToPop)
    }
}

#[derive(Clone, Debug)]

pub struct LabelUsageAndDefintions {
    reference_pos_and_id: Vec<(Position, SymbolScopeId)>,
    pub symbols: SymbolTree,
    symbol_id_to_definition_pos: HashMap<SymbolScopeId, Position>,
    pos_node_id: Vec<(Position, AstNodeId)>,
    tree: AstTree,
    docs: HashMap<AstNodeId, String>,
}

impl LabelUsageAndDefintions {
    pub fn new(tree: &AstTree, _syms: &SymbolTree, docs: HashMap<AstNodeId, String>) -> Self {
        use Item::*;

        let mut reference_pos_and_id: Vec<(Position, SymbolScopeId)> = vec![];
        let mut symbol_id_to_definition: HashMap<SymbolScopeId, Position> = HashMap::new();

        for n in iter_refs_recursive(tree.root()) {
            let v = n.value();
            let pos = v.pos.clone();

            match &v.item {
                Label(LabelDefinition::Scoped(id)) | LocalLabel(LabelDefinition::Scoped(id)) => {
                    reference_pos_and_id.push((pos, *id))
                }

                LocalAssignment(LabelDefinition::Scoped(id))
                | Assignment(LabelDefinition::Scoped(id))
                | AssignmentFromPc(LabelDefinition::Scoped(id)) => {
                    symbol_id_to_definition.insert(*id, pos);
                }

                _ => (),
            }
        }

        let mut pos_node_id = vec![];

        for n in iter_refs_recursive(tree.root()) {
            let v = n.value();
            if matches!(
                v.item,
                Label(LabelDefinition::Scoped(..))
                    | AssignmentFromPc(..)
                    | Assignment(..)
                    | LocalAssignment(..)
                    | LocalAssignmentFromPc(..)
                    | LocalLabel(LabelDefinition::Scoped(..))
            ) {
                pos_node_id.push((v.pos.clone(), n.id()))
            }
        }

        Self {
            reference_pos_and_id,
            symbols: _syms.clone(),
            symbol_id_to_definition_pos: symbol_id_to_definition,
            tree: tree.clone(),
            pos_node_id,
            docs,
        }
    }

    pub fn find_symbol_docs(&self, symbol_id: SymbolScopeId) -> Option<String> {
        self.symbol_id_to_definition_pos
            .get(&symbol_id)
            .and_then(|p| self.find_docs(p))
    }

    pub fn find_node_id_from_pos(&self, pos: &Position) -> Vec<AstNodeId> {
        let ret = self
            .pos_node_id
            .iter()
            .filter_map(|(p, id)| if p.overlaps(pos) { Some(*id) } else { None })
            .collect();
        ret
    }

    pub fn find_docs(&self, pos: &Position) -> Option<String> {
        let nodes = self.find_node_id_from_pos(pos);
        let node_id = nodes.get(0)?;
        self.docs.get(node_id).cloned()
    }

    pub fn find_references(&self, id: SymbolScopeId) -> Vec<(Position, SymbolScopeId)> {
        self.reference_pos_and_id
            .iter()
            .filter(|(_, ths_id)| *ths_id == id)
            .cloned()
            .collect()
    }

    pub fn find_references_from_pos(&self, pos: &Position) -> Vec<(Position, SymbolScopeId)> {
        if let Some(id) = self.find_symbol_id_at_pos(pos) {
            self.find_references(id)
        } else {
            vec![]
        }
    }

    pub fn find_symbol_referenced_at_pos(&self, p: &Position) -> Option<SymbolScopeId> {
        for (pos, id) in self.reference_pos_and_id.iter() {
            if p.overlaps(pos) {
                return Some(*id);
            }
        }
        None
    }

    pub fn find_scope_at_pos(&self, _p: &Position) -> u64 {
        panic!()
    }

    pub fn find_symbol_defined_at_pos(&self, p: &Position) -> Option<SymbolScopeId> {
        for (id, pos) in self.symbol_id_to_definition_pos.iter() {
            if p.overlaps(pos) {
                return Some(*id);
            }
        }
        None
    }

    /// Finds a symbol id at Pos
    /// Searches both references and definitions
    pub fn find_symbol_id_at_pos(&self, p: &Position) -> Option<SymbolScopeId> {
        self.find_symbol_referenced_at_pos(p)
            .or_else(|| self.find_symbol_defined_at_pos(p))
    }

    pub fn find_definition(&self, id: SymbolScopeId) -> Option<&Position> {
        self.symbol_id_to_definition_pos.get(&id)
    }
}
