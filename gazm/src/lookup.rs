#![forbid(unused_imports)]
use std::collections::HashMap;

use crate::{
    frontend::{AstNodeKind, LabelDefinition},
    gazmsymbols::{SymbolScopeId, SymbolTree},
    semantic::{iter_refs_recursive, Ast, AstNodeId},
};

use grl_sources::Position;

use itertools::Itertools;

#[derive(Clone, Debug)]
pub struct LabelUsageAndDefintions {
    reference_pos_and_id: Vec<(Position, SymbolScopeId)>,
    pub symbols: SymbolTree,
    symbol_id_to_definition_pos: HashMap<SymbolScopeId, Position>,
    pos_node_id: Vec<(Position, AstNodeId)>,
    // tree: AstTree,
    docs: HashMap<AstNodeId, String>,
}

impl LabelUsageAndDefintions {
    pub fn new(tree: &Ast, _syms: &SymbolTree, docs: HashMap<AstNodeId, String>) -> Self
    {
        use AstNodeKind::*;

        let mut reference_pos_and_id: Vec<(Position, SymbolScopeId)> = vec![];
        let mut symbol_id_to_definition: HashMap<SymbolScopeId, Position> = HashMap::new();

        for n in iter_refs_recursive(tree.as_ref().root()) {
            let v = n.value();

            match &v.item {
                Label(LabelDefinition::Scoped(id)) | LocalLabel(LabelDefinition::Scoped(id)) => {
                    reference_pos_and_id.push((v.pos, *id))
                }

                LocalAssignment(LabelDefinition::Scoped(id))
                | Assignment(LabelDefinition::Scoped(id))
                | AssignmentFromPc(LabelDefinition::Scoped(id)) => {
                    symbol_id_to_definition.insert(*id, v.pos);
                }

                _ => (),
            }
        }

        // Create a list of position -> node id
        // sorted by length, smallest first
        // smallest will be the enclosing span
        let pos_node_id = iter_refs_recursive(tree.as_ref().root())
            .map(|n| (n.value().pos, n.id()))
            .sorted_by(|(a, _), (b, _)| Ord::cmp(&a.range().len(), &b.range().len()))
            .collect();

        Self {
            reference_pos_and_id,
            symbols: _syms.clone(),
            symbol_id_to_definition_pos: symbol_id_to_definition,
            pos_node_id,
            docs,
        }
    }

    pub fn find_symbol_docs(&self, symbol_id: SymbolScopeId) -> Option<String> {
        self.symbol_id_to_definition_pos
            .get(&symbol_id)
            .and_then(|p| self.find_docs(p))
    }

    /// Find nodes that overlaps with this position
    pub fn find_nodes_from_pos(&self, pos: &Position) -> Vec<AstNodeId> {
        let ret = self
            .pos_node_id
            .iter()
            .filter_map(|(p, id)| if p.overlaps(pos) { Some(*id) } else { None })
            .collect();
        ret
    }

    /// Find node that contains this position
    /// will return the smallest node that intersects with this position
    pub fn find_node_from_pos(&self, pos: &Position) -> Option<AstNodeId> {
        self.pos_node_id
            .iter()
            .find(|(p, _)| p.overlaps(pos))
            .map(|(_p, id)| *id)
    }

    pub fn find_docs(&self, pos: &Position) -> Option<String> {
        self.find_node_from_pos(pos)
            .and_then(|id| self.docs.get(&id))
            .cloned()
    }

    /// Find all references to this symbol
    pub fn find_references(&self, id: SymbolScopeId) -> Vec<(Position, SymbolScopeId)> {
        self.reference_pos_and_id
            .iter()
            .filter(|(_, ths_id)| *ths_id == id)
            .cloned()
            .collect()
    }

    /// Find all to this posiiton
    pub fn find_references_from_pos(&self, pos: &Position) -> Vec<(Position, SymbolScopeId)> {
        if let Some(id) = self.find_symbol_id_at_pos(pos) {
            self.find_references(id)
        } else {
            vec![]
        }
    }

    pub fn find_symbol_referenced_at_pos(&self, pos: &Position) -> Option<SymbolScopeId> {
        self.reference_pos_and_id
            .iter()
            .find(|(p, _)| p.overlaps(pos))
            .map(|(_, id)| *id)
    }

    pub fn find_scope_at_pos(&self, _p: &Position) -> u64 {
        panic!()
    }

    pub fn find_symbol_defined_at_pos(&self, pos: &Position) -> Option<SymbolScopeId> {
        self.symbol_id_to_definition_pos
            .iter()
            .find(|(_, p)| p.overlaps(pos))
            .map(|(id, _)| *id)
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
