#![forbid(unused_imports)]

use std::fmt::Display;
use thiserror::Error;

use crate::{
    assembler::AssemblerCpuTrait,
    error::AstError,
    frontend::{AstNodeKind, LabelDefinition, ParsedFrom},
    gazmsymbols::{SymbolError, SymbolTreeReader},
    semantic::{AstNodeId, AstNodeRef},
};

use grl_eval::GetPriority;
use grl_sources::{grl_utils::Stack, Position};

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
    #[error("Can't pop top!")]
    CantPopTop,
}

#[derive(Error, Debug, Clone)]
pub struct EvalError {
    node: AstNodeId,
    pos: Position,
    #[source]
    pub source: EvalErrorEnum,
}

impl EvalError {
    pub fn new<C>(source: EvalErrorEnum, node: AstNodeRef<C>) -> Self
    where
        C: AssemblerCpuTrait,
    {
        Self {
            node: node.id(),
            pos: node.value().pos,
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

impl<C> GetPriority for AstNodeKind<C>
where
    C: std::fmt::Debug + Clone + PartialEq,
{
    fn priority(&self) -> Option<usize> {
        use AstNodeKind::*;
        match self {
            Div => Some(12),
            Mul => Some(12),
            Add => Some(11),
            Sub => Some(11),
            ShiftL => Some(10),
            ShiftR => Some(10),
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
fn eval_internal<C>(
    symbols: &SymbolTreeReader,
    n: AstNodeRef<C>,
) -> Result<AstNodeKind<C::NodeKind>, EvalError>
where
    C: AssemblerCpuTrait,
{
    use AstNodeKind::*;

    let i = &n.value().item;

    let get_sym_value = |name: &str, e| {
        symbols
            .get_symbol_info(name)
            .and_then(|si| si.value.ok_or(SymbolError::NoValue))
            .map(|n| AstNodeKind::from_number(n, ParsedFrom::Expression))
            .map_err(|_| EvalError::new(e, n))
    };

    let rez = match i {
        PostFixExpr => eval_postfix(symbols, n)?,

        Label(LabelDefinition::Scoped(id)) => {
            symbols
                .get_symbol_info_from_id(*id)
                .and_then(|si| si.value.ok_or(SymbolError::NoValue))
                .map(|n| AstNodeKind::from_number(n, ParsedFrom::Expression))
                .map_err(|_| {
                    // let name = symbols
                    //     .get_symbol_info_from_id(*id)
                    //     .expect("Interal error")
                    //     .name()
                    //     .to_string();
                    // EvalError::new(EvalErrorEnum::SymbolNotFoud(name), n);
                    EvalError::new(EvalErrorEnum::CotainsPcReference, n)
                })?
        }

        Label(LabelDefinition::Text(name)) => {
            get_sym_value(name, EvalErrorEnum::SymbolNotFoud(name.to_string()))?
        }

        Pc => get_sym_value("*", EvalErrorEnum::CotainsPcReference)?,

        UnaryTerm => {
            let mut c = n.children();
            let ops = c.next().unwrap();
            let num = c.next().unwrap();
            let r = eval_internal(symbols, num)?;

            let num = r.unrwap_number().unwrap();

            let num = &match ops.value().item {
                AstNodeKind::Sub => AstNodeKind::Num(-num, ParsedFrom::Expression),
                _ => return Err(EvalError::new(EvalErrorEnum::UnhandledUnaryTerm, n)),
            };

            num.clone()
        }

        Num(_, _) => i.clone(),

        _ => {
            return Err(EvalError::new(EvalErrorEnum::UnableToEvaluate, n));
        }
    };

    // If this isn't a number return an error
    if let AstNodeKind::Num(_, _) = rez {
        Ok(rez)
    } else {
        Err(EvalError::new(EvalErrorEnum::ExpectedANumber, n))
    }
}

/// Evaluates a postfix expression
fn eval_postfix<C>(
    symbols: &SymbolTreeReader,
    n: AstNodeRef<C>,
) -> Result<AstNodeKind<C::NodeKind>, EvalError>
where
    C: AssemblerCpuTrait,
{
    use AstNodeKind::*;

    let mut s: Stack<AstNodeKind<C::NodeKind>> = Stack::with_capacity(1024);
    let mut items: Vec<(AstNodeRef<C>, AstNodeKind<C::NodeKind>)> = Vec::with_capacity(1024);

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
            let (rhs, lhs) = s.pop_pair().expect("Can't pop pair!");

            let lhs = lhs.unrwap_number().unwrap();
            let rhs = rhs.unrwap_number().unwrap();

            let result = match i {
                Mul => lhs * rhs,
                Div => lhs / rhs,
                Add => lhs + rhs,
                Sub => lhs - rhs,
                BitAnd => lhs & rhs,
                BitXor => lhs ^ rhs,
                BitOr => lhs | rhs,
                ShiftL => lhs << (rhs as u64),
                ShiftR => lhs >> (rhs as u64),
                _ => return Err(EvalError::new(EvalErrorEnum::UnexpectedOp, *cn)),
            };

            s.push(Num(result, ParsedFrom::Expression))
        } else {
            s.push(i.clone());
        }
    }

    s.pop().ok_or(EvalError::new(EvalErrorEnum::CantPopTop, n))
}

pub fn eval<C>(symbols: &SymbolTreeReader, n: AstNodeRef<C>) -> Result<i64, EvalError>
where
    C: AssemblerCpuTrait,
{
    let ret = eval_internal(symbols, n)?;
    Ok(ret.unrwap_number().unwrap())
}
