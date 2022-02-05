use romloader::ResultExt;

use crate::ast::{to_priority, Ast, AstNodeMut, AstNodeRef, ItemWithPos};
use crate::item::Item;
use crate::postfix::GetPriotity;
use romloader::Stack;

use std::{collections::HashMap, hash::Hash};

use crate::error::AstError;
use romloader::sources::{SymbolError, SymbolTable};

fn number_or_error(i: Item, n: AstNodeRef) -> Result<Item, AstError> {
    if let Item::Number(_) = i {
        Ok(i)
    } else {
        Err(AstError::from_node("Expected a number", n))
    }
}

/// Evaluates a node and returns an item
/// Node can only contain
///  - Labels that can resolve to a value
///  - Numbers
///  - PostFixExpr containing only labels and numbers
///  - Must eval to a number
pub fn eval_internal(symbols: &SymbolTable, n: AstNodeRef) -> Result<Item, AstError> {
    use Item::*;

    let i = &n.value().item;

    let rez = match i {
        PostFixExpr => eval_postfix(symbols, n)?,

        Label(name) => symbols
            .get_value(name)
            .map(Item::number)
            .map_err(|_| {
                println!("{:#?}", symbols);
                let msg = format!("Couldn't find symbol {}", name);
                AstError::from_node(&msg, n)
            })?,

        Number(_) => i.clone(),

        _ => return Err(AstError::from_node("Unable to evaluate ", n)),
    };

    // If this isn't a number return an error
    if let Item::Number(_) = rez {
        Ok(rez)
    } else {
        Err(AstError::from_node("Expected a number", n))
    }
}
pub fn eval(symbols: &SymbolTable, n: AstNodeRef) -> Result<i64, AstError> {
    let ret = eval_internal(symbols, n)?;
    Ok(ret.get_number().unwrap())
}

/// Evaluates a postfix expression
fn eval_postfix(symbols: &SymbolTable, n: AstNodeRef) -> Result<Item, AstError> {
    use Item::*;
    let mut s: Stack<Item> = Stack::new();

    let mut items: Vec<(AstNodeRef, Item)> = vec![];

    {
        for c in n.children() {
            let i = &c.value().item;

            let item = if i.is_op() {
                i.clone()
            } else {
                eval_internal(symbols, c)?.clone()
            };

            items.push((c, item));
        }
    }

    for (cn, i) in &items {
        if i.is_op() {
            let (lhs, rhs) = s.pop_pair();

            let lhs = lhs.get_number().unwrap();
            let rhs = rhs.get_number().unwrap();

            let res = match i {
                Mul => rhs * lhs,
                Div => rhs / lhs,
                Add => rhs + lhs,
                Sub => rhs - lhs,
                _ => return Err(AstError::from_node("Unexpected op ", *cn)),
            };

            s.push(Number(res))
        } else {
            s.push(i.clone());
        }
    }

    Ok(s.pop())
}
