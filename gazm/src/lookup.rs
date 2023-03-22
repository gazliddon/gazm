// Lookup where labels are defined and referenced
use std::collections::HashMap;
use emu::utils::sources::Position;
use crate::ast::AstTree;
use crate::item::Item;


struct LabelUsageAndDefintions {
    label_to_definition_pos: HashMap<String, Position>,
    labels: Vec<(Position, String)>,
}

impl LabelUsageAndDefintions {
    pub fn new(tree: &AstTree) -> Self {
        let mut label_to_definition_pos: HashMap<String, Position> = HashMap::new();
        let mut labels: Vec<(Position, String)> = Default::default();

        use Item::*;

        for v in tree.values() {
            let pos = v.pos.clone();
            match &v.item {
                LocalAssignment(name) | Assignment(name) | AssignmentFromPc(name) => {
                    label_to_definition_pos.insert(name.clone(), pos);
                }

                Label(name) | LocalLabel(name) => {
                    labels.push((pos, name.clone()));
                }

                _ => (),
            }
        }

        Self {
            label_to_definition_pos,
            labels,
        }
    }

    pub fn find_definiton(&self, _name: &str) -> Option<Position> {
        None
    }

    pub fn find_label(&self, _p: Position) -> Option<String> {
        None
    }
}
