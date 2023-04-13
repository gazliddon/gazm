use emu::utils;
use utils::eval;

use crate::ast::{AstNodeId, AstNodeRef};
use crate::item::{Item, LabelDefinition, ParsedFrom};
use eval::GetPriority;

use std::fmt::Display;

use crate::error::AstError;
use utils::sources::{Position, SymbolQuery};
use utils::Stack;

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
    node: AstNodeId,
    pos: Position,
    #[source]
    pub source: EvalErrorEnum,
}

impl EvalError {
    pub fn new(source: EvalErrorEnum, node: AstNodeRef) -> Self {
        Self {
            node: node.id(),
            pos: node.value().pos.clone(),
            source,
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
        AstError::from_node_id(err.source.to_string(), err.node, err.pos)
    }
}

impl GetPriority for Item {
    fn priority(&self) -> Option<usize> {
        use Item::*;
        match self {
            Div => Some(12),
            Mul => Some(12),
            Add => Some(11),
            Sub => Some(11),
            ShiftLeft => Some(10),
            ShiftRight => Some(10),
            BitAnd => Some(9),
            BitXor => Some(8),
            BitOr => Some(7),

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
fn eval_internal(symbols: &dyn SymbolQuery, n: AstNodeRef) -> Result<Item, EvalError> {
    use Item::*;

    let i = &n.value().item;

    let rez = match i {
        PostFixExpr => eval_postfix(symbols, n)?,

        Label(LabelDefinition::Scoped(id)) => {
            symbols
                .get_value_from_id(*id)
                .map(|n| Item::number(n, ParsedFrom::FromExpr))
                .map_err(|_| {
                    let name = symbols
                        .get_symbol_info_from_id(*id)
                        .expect("Interal error")
                        .name
                        .clone();
                    EvalError::new(EvalErrorEnum::SymbolNotFoud(name), n)
                })?
        }

        Label(LabelDefinition::Text(name)) => symbols
            .get_value(name)
            .map(|n| Item::number(n, ParsedFrom::FromExpr))
            .map_err(|_| EvalError::new(EvalErrorEnum::SymbolNotFoud(name.to_string()), n))?,

        Pc => symbols
            .get_value("*")
            .map(|n| Item::number(n, ParsedFrom::FromExpr))
            .map_err(|_| EvalError::new(EvalErrorEnum::CotainsPcReference, n))?,

        UnaryTerm => {
            let mut c = n.children();
            let ops = c.next().unwrap();
            let num = c.next().unwrap();
            let r = eval_internal(symbols, num)?;

            let num = r.get_number().unwrap();

            let num = &match ops.value().item {
                Item::Sub => Item::Number(-num, crate::item::ParsedFrom::FromExpr),
                _ => return Err(EvalError::new(EvalErrorEnum::UnhandledUnaryTerm, n)),
            };

            num.clone()
        }

        Number(_, _) => i.clone(),

        _ => {
            return Err(EvalError::new(EvalErrorEnum::UnableToEvaluate, n));
        }
    };

    // If this isn't a number return an error
    if let Item::Number(_, _) = rez {
        Ok(rez)
    } else {
        Err(EvalError::new(EvalErrorEnum::ExpectedANumber, n))
    }
}

/// Evaluates a postfix expression
fn eval_postfix(symbols: &dyn SymbolQuery, n: AstNodeRef) -> Result<Item, EvalError> {
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
                    BitAnd => lhs & rhs,
                    BitXor => lhs ^ rhs,
                    BitOr => lhs | rhs,
                    ShiftLeft => lhs << (rhs as u64),
                    ShiftRight => lhs >> (rhs as u64),
                    _ => return Err(EvalError::new(EvalErrorEnum::UnexpectedOp, *cn)),
                };
                Ok(res)
            })
            .map_err(|_| EvalError::new(EvalErrorEnum::UnableToEvaluate, *cn))??;

            s.push(Number(res, crate::item::ParsedFrom::FromExpr))
        } else {
            s.push(i.clone());
        }
    }

    Ok(s.pop())
}

pub fn eval(symbols: &dyn SymbolQuery, n: AstNodeRef) -> Result<i64, EvalError> {
    let ret = eval_internal(symbols, n)?;
    Ok(ret.get_number().unwrap())
}
