// symtab
use crate::item::{Item, Node};


use std::collections::HashMap;

struct Symbols {
    symbols : HashMap<String, Node>,
}

enum SymbolError {
    NoValue,
    Unknown,
    AlreadyDefined(Node),
    Invalid,
    InsertionError,
}

fn get_name_and_arg(node : &Node) -> SymbolResult<(&String,&Node)> {
    let name = node.get_label_name().ok_or(SymbolError::Invalid)?;
    let child = node.get_child(1).ok_or(SymbolError::Invalid)?;
    Ok((name, child))
}

type SymbolResult<'a, T> = Result<T, SymbolError>;


impl<'a> Default for Symbols {
    fn default() -> Self {
        Self {
            symbols: Default::default()
        }
    }
}

impl Symbols {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set_or_replace(&mut self, _node: &Node) -> SymbolResult<Node> {
        todo!()
        // let (name,_) = get_name_and_arg(node)?;
        // self.symbols.insert(name.clone(), node.clone()).ok_or(SymbolError::InsertionError)
    }

    pub fn set(&mut self, _node : &Node) -> SymbolResult<()> {
        todo!()
        // let (name,_) = get_name_and_arg(node)?;
        // let previous_def = self.get(name);

        // if let Ok(previous_def) = previous_def {
        //     Err(SymbolError::AlreadyDefined(previous_def.clone()))
        // } else {
        //     self.symbols.insert(name.clone(), node.clone());
        //     Ok(())
        // }
    }

    pub fn get(&self, name: &str) -> SymbolResult<&Node> {
        self.symbols.get(&name.to_string()).ok_or(SymbolError::Unknown)
    }

    pub fn get_value(&self, _name: &str) -> SymbolResult<&Node> {
        todo!()
        // let node = self.get(name)?;
        // let (_,arg) = get_name_and_arg(node)?;
        // Ok(arg)
    }
}
