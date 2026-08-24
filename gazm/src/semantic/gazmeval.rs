#![forbid(unused_imports)]

use std::fmt::Display;
use thiserror::Error;

use crate::{
    error::AstError,
    frontend::{AstNodeKind, BinaryOp, LabelDefinition, ParsedFrom},
    gazmsymbols::{SymbolError, SymbolTreeReader},
    semantic::{AstNodeId, AstNodeRef},
};

use grl_eval::GetPriority;
use grl_sources::Position;
use grl_utils::Stack;

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
    #[error("Unknown function {0}")]
    UnknownFunction(String),
    #[error("Function {0} takes 1 argument")]
    WrongArity(String),
    #[error("sizeof expects a struct name")]
    ExpectedStructName,
    #[error("Unknown struct {0}")]
    UnknownStruct(String),
    #[error("Expression evaluates to a float; use round() to convert it to an integer")]
    FloatResult,
    #[error("Bitwise and shift operators require integer operands")]
    FloatBitwiseOp,
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

impl GetPriority for AstNodeKind {
    fn priority(&self) -> Option<usize> {
        self.binary_operator().map(|(priority, _)| priority)
    }
}

/// Evaluates a node and returns an item
/// Node can only contain
///  - Labels that can resolve to a value
///  - Numbers
///  - PostFixExpr containing only labels and numbers
///  - UnaryTerm
///  - Must eval to a number
///
/// `struct_sizes` maps struct scope ids to their total byte size
/// (`sizeof(Name)` reads it).
fn eval_internal(
    symbols: &SymbolTreeReader,
    struct_sizes: &std::collections::HashMap<u64, usize>,
    n: AstNodeRef,
) -> Result<AstNodeKind, EvalError> {
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
        PostFixExpr => eval_postfix(symbols, struct_sizes, n)?,

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
            let r = eval_internal(symbols, struct_sizes, num)?;

            match (&ops.value().item, r) {
                (AstNodeKind::Sub, AstNodeKind::Num(num, p)) => AstNodeKind::Num(-num, p),
                (AstNodeKind::Sub, AstNodeKind::Fnum(f, p)) => AstNodeKind::Fnum(-f, p),
                _ => return Err(EvalError::new(EvalErrorEnum::UnhandledUnaryTerm, n)),
            }
        }

        Num(_, _) => i.clone(),
        Fnum(_, _) => i.clone(),

        // Compile-time function call: builtins execute at assembly time
        // and produce a value. Floats stay transient here; emitting them
        // requires an explicit conversion such as round(). The builtin
        // set is in one place, keyed by name, with per-builtin arity.
        Call(name) => {
            let args: Vec<AstNodeRef> = n.children().collect();
            let (arity_min, arity_max) = match name.as_str() {
                "min" | "max" | "atan2" | "pow" => (2, 2),
                _ => (1, 1),
            };
            if args.len() < arity_min || args.len() > arity_max {
                return Err(EvalError::new(EvalErrorEnum::WrongArity(name.clone()), n));
            }

            // `sizeof(StructName)` — total struct size in bytes. The
            // argument is a struct name, not a value: resolve it to a
            // scope (chain-first, top-level fallback) and read the size
            // from the registry. A lone name parses as an Expr (postfix
            // by eval time) wrapping the label.
            if name == "sizeof" {
                let arg = args[0];
                let struct_name = match &arg.value().item {
                    Label(LabelDefinition::Text(name)) => name.clone(),
                    PostFixExpr => {
                        match arg.first_child().map(|c| c.value().item.clone()).as_ref() {
                            Some(Label(LabelDefinition::Text(name))) => name.clone(),
                            _ => return Err(EvalError::new(EvalErrorEnum::ExpectedStructName, n)),
                        }
                    }
                    _ => return Err(EvalError::new(EvalErrorEnum::ExpectedStructName, n)),
                };
                let tree = symbols.syms();
                let scope = crate::semantic::ast::find_visible_scope(
                    tree,
                    symbols.current_scope(),
                    &struct_name,
                )
                .ok_or_else(|| {
                    EvalError::new(EvalErrorEnum::UnknownStruct(struct_name.clone()), n)
                })?;
                let size = *struct_sizes.get(&scope).ok_or_else(|| {
                    EvalError::new(EvalErrorEnum::UnknownStruct(struct_name.clone()), n)
                })?;
                AstNodeKind::Num(size as i64, ParsedFrom::Expression)
            } else {
                // 1-arg and 2-arg math/bit builtins.
                let a = eval_internal(symbols, struct_sizes, args[0])?;
                match name.as_str() {
                    "sin" => apply_math(a, f64::sin),
                    "cos" => apply_math(a, f64::cos),
                    "sqrt" => apply_math(a, f64::sqrt),
                    "abs" => match a {
                        AstNodeKind::Num(x, _) => AstNodeKind::Num(x.abs(), ParsedFrom::Expression),
                        AstNodeKind::Fnum(x, _) => {
                            AstNodeKind::Fnum(x.abs(), ParsedFrom::Expression)
                        }
                        _ => unreachable!(),
                    },
                    // Integer-valued conversions (like round): float
                    // floors/ceils become integers, ints pass through.
                    "floor" => match a {
                        AstNodeKind::Num(x, _) => AstNodeKind::Num(x, ParsedFrom::Expression),
                        AstNodeKind::Fnum(x, _) => {
                            AstNodeKind::Num(x.floor() as i64, ParsedFrom::Expression)
                        }
                        _ => unreachable!(),
                    },
                    "ceil" => match a {
                        AstNodeKind::Num(x, _) => AstNodeKind::Num(x, ParsedFrom::Expression),
                        AstNodeKind::Fnum(x, _) => {
                            AstNodeKind::Num(x.ceil() as i64, ParsedFrom::Expression)
                        }
                        _ => unreachable!(),
                    },
                    "round" => match a {
                        // round() of an integer is the integer itself.
                        AstNodeKind::Num(x, _) => AstNodeKind::Num(x, ParsedFrom::Expression),
                        AstNodeKind::Fnum(x, _) => {
                            AstNodeKind::Num(x.round() as i64, ParsedFrom::Expression)
                        }
                        _ => unreachable!(),
                    },
                    // Byte splitting: integers only.
                    "hi" => match a {
                        AstNodeKind::Num(x, _) => {
                            AstNodeKind::Num((x >> 8) & 0xFF, ParsedFrom::Expression)
                        }
                        _ => return Err(EvalError::new(EvalErrorEnum::ExpectedANumber, n)),
                    },
                    "lo" => match a {
                        AstNodeKind::Num(x, _) => {
                            AstNodeKind::Num(x & 0xFF, ParsedFrom::Expression)
                        }
                        _ => return Err(EvalError::new(EvalErrorEnum::ExpectedANumber, n)),
                    },
                    // 2-arg: min/max keep ints ints (promoting when a
                    // float is involved); atan2/pow are float math.
                    "min" | "max" => {
                        let b = eval_internal(symbols, struct_sizes, args[1])?;
                        match (&a, &b) {
                            (AstNodeKind::Num(x, _), AstNodeKind::Num(y, _)) => {
                                let v = if name == "min" { x.min(y) } else { x.max(y) };
                                AstNodeKind::Num(*v, ParsedFrom::Expression)
                            }
                            _ => {
                                let (x, y) = (number_to_f64(&a), number_to_f64(&b));
                                let v = if name == "min" { x.min(y) } else { x.max(y) };
                                AstNodeKind::Fnum(v, ParsedFrom::Expression)
                            }
                        }
                    }
                    "atan2" => {
                        let b = eval_internal(symbols, struct_sizes, args[1])?;
                        let (y, x) = (number_to_f64(&a), number_to_f64(&b));
                        AstNodeKind::Fnum(y.atan2(x), ParsedFrom::Expression)
                    }
                    "pow" => {
                        let b = eval_internal(symbols, struct_sizes, args[1])?;
                        let (x, y) = (number_to_f64(&a), number_to_f64(&b));
                        AstNodeKind::Fnum(x.powf(y), ParsedFrom::Expression)
                    }
                    _ => {
                        return Err(EvalError::new(
                            EvalErrorEnum::UnknownFunction(name.clone()),
                            n,
                        ))
                    }
                }
            }
        }

        _ => {
            return Err(EvalError::new(EvalErrorEnum::UnableToEvaluate, n));
        }
    };

    // If this isn't a number return an error
    if rez.is_number() {
        Ok(rez)
    } else {
        Err(EvalError::new(EvalErrorEnum::ExpectedANumber, n))
    }
}

/// Evaluates a postfix expression
fn eval_postfix(
    symbols: &SymbolTreeReader,
    struct_sizes: &std::collections::HashMap<u64, usize>,
    n: AstNodeRef,
) -> Result<AstNodeKind, EvalError> {
    use AstNodeKind::*;

    let mut s: Stack<AstNodeKind> = Stack::with_capacity(1024);
    let mut items: Vec<(AstNodeRef, AstNodeKind)> = Vec::with_capacity(1024);

    {
        for c in n.children() {
            let i = &c.value().item;

            let item = if i.is_op() {
                i.clone()
            } else {
                eval_internal(symbols, struct_sizes, c)?.clone()
            };

            items.push((c, item));
        }
    }

    for (cn, i) in &items {
        if i.is_op() {
            let (rhs, lhs) = s.pop_pair().expect("Can't pop pair!");

            let (_, op) = i
                .binary_operator()
                .ok_or_else(|| EvalError::new(EvalErrorEnum::UnexpectedOp, *cn))?;

            // Float promotion: if either operand is a float, both are
            // widened to f64 and the float semantics apply. Comparisons
            // and logicals always produce integer 0/1, so they can feed
            // `if`/`while` conditions directly.
            let result = match (&lhs, &rhs) {
                (Num(a, _), Num(b, _)) => Num(apply_int(op, *a, *b), ParsedFrom::Expression),
                _ => {
                    let lf = number_to_f64(&lhs);
                    let rf = number_to_f64(&rhs);
                    apply_float(op, lf, rf)
                        .map_err(|_| EvalError::new(EvalErrorEnum::FloatBitwiseOp, *cn))?
                }
            };

            s.push(result)
        } else {
            s.push(i.clone());
        }
    }

    s.pop().ok_or(EvalError::new(EvalErrorEnum::CantPopTop, n))
}

pub fn eval(
    symbols: &SymbolTreeReader,
    struct_sizes: &std::collections::HashMap<u64, usize>,
    n: AstNodeRef,
) -> Result<i64, EvalError> {
    let ret = eval_internal(symbols, struct_sizes, n)?;
    match ret {
        AstNodeKind::Num(n, _) => Ok(n),
        // A float reaching the boundary (fcb/fdb/equ/conditions...) is an
        // error: emitting one requires an explicit conversion.
        AstNodeKind::Fnum(..) => Err(EvalError::new(EvalErrorEnum::FloatResult, n)),
        _ => Err(EvalError::new(EvalErrorEnum::ExpectedANumber, n)),
    }
}

/// Apply a builtin math function to a number (int or float); the result
/// is always a float.
fn apply_math(value: AstNodeKind, f: fn(f64) -> f64) -> AstNodeKind {
    match value {
        AstNodeKind::Num(x, _) => AstNodeKind::Fnum(f(x as f64), ParsedFrom::Expression),
        AstNodeKind::Fnum(x, _) => AstNodeKind::Fnum(f(x), ParsedFrom::Expression),
        _ => unreachable!(),
    }
}

/// The integer semantics of a binary operator. Mirrors `apply_float`;
/// both live here so operator behaviour stays in one place.
fn apply_int(op: BinaryOp, l: i64, r: i64) -> i64 {
    use BinaryOp::*;
    match op {
        Mul => l * r,
        Div => l / r,
        Add => l + r,
        Sub => l - r,
        ShiftL => l << (r as u64),
        ShiftR => l >> (r as u64),
        BitAnd => l & r,
        BitXor => l ^ r,
        BitOr => l | r,
        Equal => (l == r) as i64,
        NotEqual => (l != r) as i64,
        LessThan => (l < r) as i64,
        LessThanEqual => (l <= r) as i64,
        GreaterThan => (l > r) as i64,
        GreaterThanEqual => (l >= r) as i64,
        LogicalAnd => (l != 0 && r != 0) as i64,
        LogicalOr => (l != 0 || r != 0) as i64,
    }
}

/// The float semantics of a binary operator. Comparisons and logicals
/// yield integer 0/1 (usable in conditions); bitwise/shift operators
/// have no float meaning and are rejected.
fn apply_float(op: BinaryOp, l: f64, r: f64) -> Result<AstNodeKind, ()> {
    use BinaryOp::*;
    Ok(match op {
        Mul => AstNodeKind::Fnum(l * r, ParsedFrom::Expression),
        Div => AstNodeKind::Fnum(l / r, ParsedFrom::Expression),
        Add => AstNodeKind::Fnum(l + r, ParsedFrom::Expression),
        Sub => AstNodeKind::Fnum(l - r, ParsedFrom::Expression),
        Equal => AstNodeKind::Num((l == r) as i64, ParsedFrom::Expression),
        NotEqual => AstNodeKind::Num((l != r) as i64, ParsedFrom::Expression),
        LessThan => AstNodeKind::Num((l < r) as i64, ParsedFrom::Expression),
        LessThanEqual => AstNodeKind::Num((l <= r) as i64, ParsedFrom::Expression),
        GreaterThan => AstNodeKind::Num((l > r) as i64, ParsedFrom::Expression),
        GreaterThanEqual => AstNodeKind::Num((l >= r) as i64, ParsedFrom::Expression),
        LogicalAnd => AstNodeKind::Num((l != 0.0 && r != 0.0) as i64, ParsedFrom::Expression),
        LogicalOr => AstNodeKind::Num((l != 0.0 || r != 0.0) as i64, ParsedFrom::Expression),
        ShiftL | ShiftR | BitAnd | BitXor | BitOr => return Err(()),
    })
}

fn number_to_f64(n: &AstNodeKind) -> f64 {
    match n {
        AstNodeKind::Num(x, _) => *x as f64,
        AstNodeKind::Fnum(x, _) => *x,
        _ => unreachable!(),
    }
}
