#![forbid(unused_imports)]
use crate::{cpukind::CpuKind, frontend::LabelDefinition};
use grl_sources::Position;
use std::path::PathBuf;
use strum_macros::{EnumDiscriminants, EnumIter};
use thin_vec::ThinVec;

use crate::{
    cpu6800::frontend::NodeKind6800, cpu6809::frontend::NodeKind6809, error::ParseError,
    gazmsymbols::SymbolScopeId, semantic::AstNodeId,
};

#[derive(Debug, PartialEq, Clone)]
pub enum CpuSpecific {
    Cpu6809(NodeKind6809),
    Cpu6800(NodeKind6800),
}

use super::{BaseNode, CtxTrait};

impl CtxTrait for Position {}
pub type Node = BaseNode<AstNodeKind, Position>;

#[derive(Debug, PartialEq, Clone)]
pub enum ParsedFrom {
    Hexadecimal,
    Decimal,
    Binary,
    Character,
    Expression,
}

///Ast Node Items
#[derive(Debug, PartialEq, Clone, EnumDiscriminants)]
#[strum_discriminants(name(AstNodeKindDiscriminants), derive(EnumIter, Hash))]
pub enum AstNodeKind {
    Cpu(CpuKind),
    TargetSpecific(CpuSpecific),
    Import,
    Doc(String),
    Pc,
    BlankLine,
    Skip(usize),
    LocalAssignment(LabelDefinition),
    Assignment(LabelDefinition),
    AssignmentFromPc(LabelDefinition),
    LocalAssignmentFromPc(LabelDefinition),

    MacroCall(String),

    MacroCallProcessed {
        scope_id: u64,
        macro_id: AstNodeId,
        params_vec_of_id: ThinVec<SymbolScopeId>,
    },

    MacroDef(String, ThinVec<String>),

    /// `repeat <count> [, <index>] { body }` — the body children are
    /// assembled `count` times. The count expression is the first child;
    /// the remaining children are the body. The optional `<index>` names a
    /// loop variable bound to the current iteration (0-based) inside the body.
    Repeat {
        index: Option<String>,
    },

    /// `if <condition> { body } [else { body }]` — assembly-time
    /// conditional. The condition expression is the first child; the taken
    /// branch's statements are the remaining children, followed by an
    /// `Else` node (if present) holding the untaken branch.
    If,

    /// Container for the untaken `else` branch of an `If` node; the
    /// else-body statements are its children. Only ever a child of `If`.
    Else,

    /// `while <condition> { body }` — assembly-time loop. The condition
    /// expression is the first child (re-evaluated each iteration); the
    /// remaining children are the body.
    While,

    /// `break` — exit the innermost enclosing `repeat`/`while`/`for`
    /// immediately. Only valid inside a loop.
    Break,

    /// `continue` — skip the rest of the current iteration of the
    /// innermost enclosing loop and proceed to the next one. Only valid
    /// inside a loop.
    Continue,

    /// `for <index> in <start>..<end> { body }` — assembly-time range
    /// loop. The index is bound to each value in `start..end` (end
    /// exclusive). Children: start expression, end expression, body.
    For {
        index: String,
    },

    StructDef(String),
    StructEntry(String),

    SetPc(usize),
    SetPutOffset(isize),
    SetSection(String),
    Section(String),

    Scope(String),
    ScopeId(u64),

    Expr,
    PostFixExpr,
    BracketedExpr,

    UnaryTerm,
    Label(LabelDefinition),
    LocalLabel(LabelDefinition),

    Comment(String),

    Num(i64, ParsedFrom),

    Include(PathBuf),
    Require(PathBuf),
    IncBin(PathBuf),
    IncBinRef(PathBuf),
    GrabMem,
    IncBinResolved {
        file: PathBuf,
        r: std::ops::Range<usize>,
    },

    WriteBin(PathBuf),

    TokenizedFile(PathBuf, Option<PathBuf>),

    Errors(ThinVec<ParseError>),

    Exec,
    Org,
    Put,
    Fdb(usize),
    Fcb(usize),
    Fcc(String),
    Rmb,
    Fill,
    Zmb,
    Zmd,

    Mul,
    Div,
    Add,
    Sub,
    BitAnd,
    BitOr,
    BitXor,
    ShiftR,
    ShiftL,
    UnaryGreaterThan,

    /// Comparison operators: evaluate to 1 (true) or 0 (false), usable in
    /// `if`/`while` conditions and any expression.
    Equal,
    NotEqual,
    LessThan,
    LessThanEqual,
    GreaterThan,
    GreaterThanEqual,

    /// Logical operators: true (1) when both operands are non-zero /
    /// when either operand is non-zero. Both operands are always
    /// evaluated — there is no short-circuiting.
    LogicalAnd,
    LogicalOr,
    Block,

    /// Floating-point literal. Floats are transient expression values
    /// (the target is 8-bit): arithmetic on them happens at assembly
    /// time, and an explicit conversion such as `round()` is required to
    /// turn a float back into an emitted integer.
    Fnum(f64, ParsedFrom),

    /// Compile-time function call, e.g. `sin(x)`. The callee is a
    /// builtin (currently `sin`, `cos`, `round`); children are the
    /// argument expressions. Functions are not macros: they execute at
    /// assembly time and produce a value.
    Call(String),

    /// `assert <condition> [, message]` — compile-time check. The
    /// condition is evaluated during sizing; a zero (false) result fails
    /// the assembly with the message, if any. Children: the condition
    /// expression, then the message's value expressions (see `MsgPart`).
    Assert(Vec<MsgPart>),

    /// `log <message>` — print a message during assembly (sizing time).
    /// Children: the message's value expressions (see `MsgPart`).
    Log(Vec<MsgPart>),
}

/// One part of an interpolated `log`/`assert` message. A message is a
/// sequence of literal text and `{expr}` value parts, e.g.
/// `log "table: " {sizeof(Proc)} " bytes"`. `Value(i)` is the *i*-th
/// expression child of the node: for `Assert` the condition is child 0
/// and values start at child 1; for `Log` values start at child 0.
#[derive(Debug, Clone, PartialEq)]
pub enum MsgPart {
    /// Literal text, printed verbatim.
    Text(String),
    /// An expression child whose evaluated value is formatted into the
    /// message.
    Value(usize),
}

/// Identity of a binary operator, so precedence lives in one table
/// (`binary_operator`) while the integer and float semantics live in one
/// place (the evaluator). Keep the variants in step with the table.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BinaryOp {
    Mul,
    Div,
    Add,
    Sub,
    ShiftL,
    ShiftR,
    BitAnd,
    BitXor,
    BitOr,
    Equal,
    NotEqual,
    LessThan,
    LessThanEqual,
    GreaterThan,
    GreaterThanEqual,
    LogicalAnd,
    LogicalOr,
}

impl AstNodeKind {
    pub fn zero() -> Self {
        AstNodeKind::Num(0, ParsedFrom::Expression)
    }

    pub fn from_number(n: i64, p: ParsedFrom) -> Self {
        AstNodeKind::Num(n, p)
    }

    pub fn is_expr(&self) -> bool {
        matches!(self, Self::Expr | Self::BracketedExpr)
    }
    pub fn is_number(&self) -> bool {
        matches!(self, Self::Num(..) | Self::Fnum(..))
    }

    /// Binary operators: precedence and operator identity in one table,
    /// so the precedence rules and the evaluator semantics can never
    /// drift apart. A future operator is added here once; `GetPriority`
    /// and the postfix evaluator both consume this. The integer and
    /// float `apply` semantics live in the evaluator, keyed by
    /// `BinaryOp`.
    pub fn binary_operator(&self) -> Option<(usize, BinaryOp)> {
        use AstNodeKind::*;
        match self {
            Mul => Some((12, BinaryOp::Mul)),
            Div => Some((12, BinaryOp::Div)),
            Add => Some((11, BinaryOp::Add)),
            Sub => Some((11, BinaryOp::Sub)),
            ShiftL => Some((10, BinaryOp::ShiftL)),
            ShiftR => Some((10, BinaryOp::ShiftR)),
            BitAnd => Some((9, BinaryOp::BitAnd)),
            BitXor => Some((8, BinaryOp::BitXor)),
            BitOr => Some((7, BinaryOp::BitOr)),
            Equal => Some((6, BinaryOp::Equal)),
            NotEqual => Some((6, BinaryOp::NotEqual)),
            LessThan => Some((5, BinaryOp::LessThan)),
            LessThanEqual => Some((5, BinaryOp::LessThanEqual)),
            GreaterThan => Some((5, BinaryOp::GreaterThan)),
            GreaterThanEqual => Some((5, BinaryOp::GreaterThanEqual)),
            LogicalAnd => Some((4, BinaryOp::LogicalAnd)),
            LogicalOr => Some((3, BinaryOp::LogicalOr)),
            _ => None,
        }
    }

    pub fn unrwap_number(&self) -> Option<i64> {
        if let AstNodeKind::Num(n, _) = self {
            Some(*n)
        } else {
            None
        }
    }

    pub fn unwrap_macro_def(&self) -> Option<(&String, &[String])> {
        if let AstNodeKind::MacroDef(name, params) = self {
            Some((name, params))
        } else {
            None
        }
    }

    pub fn unwrap_include(&self) -> Option<&PathBuf> {
        if let AstNodeKind::Include(n) = self {
            Some(n)
        } else {
            None
        }
    }
    pub fn unwrap_label_text(&self) -> Option<&str> {
        use AstNodeKind::*;
        match self {
            Label(x) | LocalLabel(x) => x.get_text(),
            _ => None,
        }
    }

    pub fn unwrap_label_id(&self) -> Option<SymbolScopeId> {
        use AstNodeKind::*;
        use LabelDefinition::Scoped;

        match self {
            Label(Scoped(id)) | LocalLabel(Scoped(id)) => Some(*id),
            _ => None,
        }
    }
}

// impl Display for BaseNode<Item<MC6809>, Position> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         use Item::*;

//         let item = &self.item;

//         let join_children = |sep| join_vec(&self.children, sep);

//         let ret: String = match item {
//             AssignmentFromPc(name) | LocalAssignmentFromPc(name) => {
//                 format!("{name} equ *")
//             }

//             Pc => "*".to_string(),

//             Label(name) | LocalLabel(name) => {
//                 format!("{name}")
//             }

//             Comment(comment) => comment.clone(),
//             // QuotedString(test) => format!("\"{}\"", test),
//             // Register(r) => r.to_string(),

//             // RegisterList(vec) => join_vec(vec, ","),
//             LocalAssignment(name) | Assignment(name) => {
//                 format!("{} equ {}", name, self.children[0])
//             }

//             Expr => join_children(""),

//             Include(file) => format!("include \"{}\"", file.to_string_lossy()),

//             Num(n, p) => match &p {
//                 ParsedFrom::Hexadecimal => format!("${n:x}"),
//                 ParsedFrom::Expression | ParsedFrom::Decimal | ParsedFrom::Character => {
//                     n.to_string()
//                 }
//                 ParsedFrom::Binary => format!("%{n:b}"),
//             },
//             UnaryTerm => join_children(""),

//             Mul => "*".to_string(),
//             Div => "/".to_string(),
//             Add => "+".to_string(),
//             Sub => "-".to_string(),
//             BitAnd => "&".to_string(),
//             BitOr => "|".to_string(),
//             BitXor => "^".to_string(),
//             Org => {
//                 format!("org {}", self.children[0])
//             }

//             BracketedExpr => {
//                 format!("({})", join_children(""))
//             }

//             TokenizedFile(file, ..) => {
//                 let header = format!("; included file {}", file.to_string_lossy());
//                 let children: Vec<String> = self.children.iter().map(|n| format!("{n}")).collect();
//                 format!("{}\n{}", header, children.join("\n"))
//             }

//             CpuSpecific(cpu_kind) => {
//                 handle_6809_fmt(self, cpu_kind.clone())
//             }

//             _ => format!("{item:?} not implemented"),
//         };

//         write!(f, "{ret}")
//     }
// }

////////////////////////////////////////////////////////////////////////////////
