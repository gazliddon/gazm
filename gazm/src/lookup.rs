// Lookup where labels are defined and referenced
use crate::ast::AstTree;
use crate::item::Item;
use emu::utils::sources::Position;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct LabelUsageAndDefintions {
    label_to_definition_pos: HashMap<String, Position>,
    label_references: Vec<(String, Position)>,
}

use log::info;

impl LabelUsageAndDefintions {
    pub fn new(tree: &AstTree) -> Self {
        let mut label_to_definition_pos: HashMap<String, Position> = HashMap::new();
        let mut labels: Vec<(String, Position)> = Default::default();

        use Item::*;

        for v in tree.values() {
            let pos = v.pos.clone();
            match &v.item {
                LocalAssignment(name) | Assignment(name) | AssignmentFromPc(name) => {
                    label_to_definition_pos.insert(name.clone(), pos);
                }
                Label(name) | LocalLabel(name) => {
                    labels.push((name.to_string(), pos));
                }

                _ => (),
            }
        }

        Self {
            label_to_definition_pos,
            label_references: labels,
        }
    }

    pub fn find_references(&self, _name: &str) -> Option<Vec<(String, Position)>> {
        let ret: Vec<(String, Position)> = self
            .label_references
            .iter()
            .filter_map(|pair| if pair.0 == _name { Some(pair) } else { None })
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
            .find(|(s, _)| s.to_string() == _name)
            .map(|(_, p)| p)
    }

    pub fn find_definition_from_pos(&self, pos: &Position) -> Option<&str> {
        for (k,v) in self.label_to_definition_pos.iter() {

            if v.overlaps(pos) {
                return Some(k)
            }
        }

        None
    }
    pub fn find_label_or_defintion(&self, pos: &Position) -> Option<&str> {
        let label = self.label_references
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
}
