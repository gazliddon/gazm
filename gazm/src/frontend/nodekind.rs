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
    Rmd,
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
        matches!(self, Self::Num(..))
    }

    /// Binary operators: precedence and evaluation in one table, so the
    /// precedence rules and the evaluator semantics can never drift apart.
    /// A future operator is added here once; `GetPriority` and the postfix
    /// evaluator both consume this.
    pub fn binary_operator(&self) -> Option<(usize, fn(i64, i64) -> i64)> {
        use AstNodeKind::*;
        match self {
            Mul => Some((12, |l, r| l * r)),
            Div => Some((12, |l, r| l / r)),
            Add => Some((11, |l, r| l + r)),
            Sub => Some((11, |l, r| l - r)),
            ShiftL => Some((10, |l, r| l << (r as u64))),
            ShiftR => Some((10, |l, r| l >> (r as u64))),
            BitAnd => Some((9, |l, r| l & r)),
            BitXor => Some((8, |l, r| l ^ r)),
            BitOr => Some((7, |l, r| l | r)),
            Equal => Some((6, |l, r| (l == r) as i64)),
            NotEqual => Some((6, |l, r| (l != r) as i64)),
            LessThan => Some((5, |l, r| (l < r) as i64)),
            LessThanEqual => Some((5, |l, r| (l <= r) as i64)),
            GreaterThan => Some((5, |l, r| (l > r) as i64)),
            GreaterThanEqual => Some((5, |l, r| (l >= r) as i64)),
            LogicalAnd => Some((4, |l, r| (l != 0 && r != 0) as i64)),
            LogicalOr => Some((3, |l, r| (l != 0 || r != 0) as i64)),
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
