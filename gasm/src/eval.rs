use romloader::ResultExt;
use serde_json::to_string;

use crate::ast::{Ast, AstNodeId, AstNodeMut, AstNodeRef, ItemWithPos, to_priority};
use crate::item::Item;
use crate::postfix::GetPriotity;
use romloader::Stack;

use std::backtrace::Backtrace;
use std::fmt::{Display, format};
use std::{collections::HashMap, hash::Hash};

use crate::error::{AstError, UserError};
use romloader::sources::{SymbolError, SymbolTable, Position, SymbolQuery};


use crate::astformat::as_string;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum EvalErrorEnum {
    #[error("Unexpected Op")]
    UnexpectedOp,
    #[error("Symbol not found {0}")]
    SymbolNotFoud(String),
    #[error("Contains unresolved reference to PC")]
    CotainsPcReference,
    #[error("Expected a number")]
    ExpectedANumber,
    #[error("Unhandled unary term")]
    UnhandledUnaryTerm,
    #[error("Can't evaluate node")]
    UnableToEvaluate,
}

#[derive(Error, Debug, Clone)]
pub struct EvalError {
    node : AstNodeId,
    pos:   Position,
    #[source]
    pub source: EvalErrorEnum,
}

impl EvalError {
    pub fn new(source: EvalErrorEnum, node: AstNodeRef) -> Self {
        Self {
            node: node.id(),
            pos: node.value().pos.clone(),
            source
        }
    }
}

impl Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.source)
    }
}

impl From<EvalError> for AstError {
    fn from(err: EvalError) -> Self {
        AstError::from_node_id(err.source.to_string(),err.node, err.pos)
    }
}

fn number_or_error(i: Item, n: AstNodeRef) -> Result<Item, EvalError> {
    if let Item::Number(_) = i {
        Ok(i)
    } else {
        Err( EvalError::new(EvalErrorEnum::ExpectedANumber, n))
    }
}

impl GetPriotity for Item {
    fn priority(&self) -> Option<usize> {
        match self {
            Item::Mul => Some(5),
            Item::Div => Some(5),
            Item::Add => Some(4),
            Item::Sub => Some(4),
            Item::And => Some(3),
            Item::ShiftRight => Some(2),
            Item::ShiftLeft => Some(2),
            _ => None,
        }
    }
}

/// Evaluates a node and returns an item
/// Node can only contain
///  - Labels that can resolve to a value
///  - Numbers
///  - PostFixExpr containing only labels and numbers
///  - UnaryTerm
///  - Must eval to a number
fn eval_internal(symbols: &SymbolTable, n: AstNodeRef) -> Result<Item, EvalError> {
    use Item::*;

    let i = &n.value().item;

    let rez = match i {
        PostFixExpr => eval_postfix(symbols, n)?,

        Label(name) => symbols.get_value(name).map(Item::number).map_err(|_| {
            EvalError::new(EvalErrorEnum::SymbolNotFoud(name.to_string()), n)
        })?,

        Pc => {
            symbols.get_value("*").map(Item::number).map_err(|_| {
                EvalError::new(EvalErrorEnum::CotainsPcReference, n)
            })?
        }

        UnaryTerm => {
            let ops = n.children().nth(0).unwrap();
            let num = n.children().nth(1).unwrap();
            let r = eval_internal(symbols, num)?;

            let num = r.get_number().unwrap();

            let num = &match ops.value().item {
                Item::Sub => Item::Number(-num),
                _ => {
                    return Err(EvalError::new(EvalErrorEnum::UnhandledUnaryTerm, n))
                }
            };

            num.clone()
        }

        Number(_) => i.clone(),

        _ => {
            return Err(
                EvalError::new(EvalErrorEnum::UnableToEvaluate, n)
                );
        }
    };

    // If this isn't a number return an error
    if let Item::Number(_) = rez {
        Ok(rez)
    } else {
        Err(
            EvalError::new(EvalErrorEnum::ExpectedANumber, n)
            )
    }
}



/// Evaluates a postfix expression
fn eval_postfix(symbols: &SymbolTable, n: AstNodeRef) -> Result<Item, EvalError> {
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
    use std::panic;

    for (cn, i) in &items {
        if i.is_op() {
            let (rhs, lhs) = s.pop_pair();

            let lhs = lhs.get_number().unwrap();
            let rhs = rhs.get_number().unwrap();

            let res = panic::catch_unwind(|| {
                let res = match i {
                    Mul => lhs * rhs,
                    Div => lhs / rhs,
                    Add => lhs + rhs,
                    Sub => lhs - rhs,
                    And => lhs & rhs,
                    ShiftLeft => lhs << (rhs as u64),
                    ShiftRight => lhs >> (rhs as u64),
                    _ => return Err(EvalError::new(EvalErrorEnum::UnexpectedOp, *cn)),
                };
                Ok(res)
            })
            .map_err(|_| 
                { EvalError::new(EvalErrorEnum::UnableToEvaluate, *cn) })??;

            s.push(Number(res))

        } else {
            s.push(i.clone());
        }
    }

    Ok(s.pop())
}

pub fn eval(symbols: &SymbolTable, n: AstNodeRef) -> Result<i64, EvalError> {
    let ret = eval_internal(symbols, n)?;
    Ok(ret.get_number().unwrap())
}
