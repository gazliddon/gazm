// Lookup where labels are defined and referenced
use crate::ast::AstTree;
use crate::item::Node;
use crate::item::{Item, LabelDefinition};
use emu::utils::sources::{Position, SymbolScopeId, SymbolTree, SymbolTable, SymbolInfo, SymbolError};
use emu::utils::Stack;
use std::collections::{HashMap, VecDeque};

pub struct Navigator<'a> {
    syms: &'a SymbolTree,
    current_scope_id: u64,
    scope_stack: Stack<u64>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NavError {
    UnableToPop,
    ScopeIdNotFound(u64),
    SymbolError(SymbolError)
}

impl<'a> Navigator<'a> {
    pub fn new(syms: &'a SymbolTree) -> Self {
        Self {
            syms,
            current_scope_id: syms.get_root_id(),
            scope_stack: Default::default(),
        }
    }
    
    pub fn new_with_id(syms: &'a SymbolTree, id: u64) -> Result<Self,NavError> {
        let mut ret = Self::new(syms);
        ret.set_scope(id)?;
        Ok(ret)
    }

    fn get_scope(&self, scope_id: u64) -> Result<&SymbolTable, NavError> {
        self.syms
            .get_scope_symbols_from_id(scope_id)
            .map_err( NavError::SymbolError)
    }

    fn check_scope(&self, scope_id: u64) -> Result<(), NavError> {
        self.get_scope(scope_id).map(|_|())
    }

    pub fn get_current_scope_id(&self) -> u64 {
        self.current_scope_id
    }

    pub fn resolve_label(&self, _name: &str) -> Result<&SymbolInfo, SymbolError> {
        self.syms.resolve_label(_name, self.current_scope_id)
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
    label_to_definition_pos: HashMap<String, Position>,
    label_references: Vec<(String, Position)>,

    nodes_by_id: HashMap<usize, Node>,
}

use log::info;


impl LabelUsageAndDefintions {
    pub fn new(tree: &AstTree) -> Self {
        use Item::*;

        let mut label_to_definition_pos: HashMap<String, Position> = HashMap::new();
        let mut labels: Vec<(String, Position)> = Default::default();


        for v in tree.values() {
            let pos = v.pos.clone();
            match &v.item {
                LocalAssignment(LabelDefinition::Text(name))
                | Assignment(LabelDefinition::Text(name))
                | AssignmentFromPc(LabelDefinition::Text(name)) => {
                    label_to_definition_pos.insert(name.clone(), pos);
                }
                Label(LabelDefinition::Text(name)) | LocalLabel(LabelDefinition::Text(name)) => {
                    labels.push((name.to_string(), pos));
                }

                _ => (),
            }
        }

        Self {
            label_to_definition_pos,
            label_references: labels,
            nodes_by_id: Default::default(),
        }
    }

    pub fn find_references(&self, name: &str) -> Option<Vec<(String, Position)>> {
        let ret: Vec<(String, Position)> = self
            .label_references
            .iter()
            .filter(|pair| pair.0 == name)
            .cloned()
            .collect();

        if ret.is_empty() {
            None
        } else {
            Some(ret)
        }
    }

    pub fn find_definition(&self, _name: &str) -> Option<&Position> {
        self.label_to_definition_pos
            .iter()
            .find(|(s, _)| *s == _name)
            .map(|(_, p)| p)
    }

    pub fn find_definition_from_pos(&self, pos: &Position) -> Option<&str> {
        for (k, v) in self.label_to_definition_pos.iter() {
            if v.overlaps(pos) {
                return Some(k);
            }
        }

        None
    }
    pub fn find_label_or_defintion(&self, pos: &Position) -> Option<&str> {
        let label = self
            .label_references
            .iter()
            .find(|p| p.1.overlaps(pos))
            .map(|p| p.0.as_str());

        if label.is_none() {
            self.find_definition_from_pos(pos)
        } else {
            label
        }
    }

    pub fn find_label(&self, pos: &Position) -> Option<&str> {
        self.label_references
            .iter()
            .find(|p| p.1.overlaps(pos))
            .map(|p| p.0.as_str())
    }

    pub fn find_node(&self, _line: usize, _col: usize, _file_id: usize) {}
}

enum AstNodeKind {
    Label(SymbolScopeId),
    Command
}

pub struct AstSearchResult {
}





