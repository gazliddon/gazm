#![forbid(unused_imports)]
use grl_sources::Position;
use std::{fmt::Display, path::PathBuf, str::FromStr};
use thin_vec::ThinVec;
use crate::frontend::LabelDefinition;

use crate::{
    assembler::AssemblerCpuTrait, cpu6800::frontend::NodeKind6800, cpu6809::frontend::NodeKind6809,
    error::ParseError, gazmsymbols::SymbolScopeId, semantic::AstNodeId,
};

use super::{BaseNode, CtxTrait};

impl CtxTrait for Position {}

pub type Node<C> = BaseNode<Item<C>, Position>;

#[derive(Debug, PartialEq, Clone)]
pub enum StructMemberType {
    Byte,
    Word,
    DWord,
    QWord,
    UserType(String),
}

impl FromStr for StructMemberType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ret = match s {
            "byte" => StructMemberType::Byte,
            "word" => StructMemberType::Word,
            "dword" => StructMemberType::DWord,
            "qword" => StructMemberType::QWord,
            _ => StructMemberType::UserType(s.to_string()),
        };

        Ok(ret)
    }
}

impl StructMemberType {
    pub fn to_size_item<C>(&self) -> Item<C::NodeKind>
    where
        C: AssemblerCpuTrait,
    {
        use Item::*;
        use ParsedFrom::Expression;
        match self {
            Self::Byte => Num(1, Expression),
            Self::Word => Num(2, Expression),
            Self::DWord => Num(4, Expression),
            Self::QWord => Num(8, Expression),
            Self::UserType(name) => Label(LabelDefinition::Text(format!("{name}.size"))),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct StructEntry {
    pub name: String,
    pub item_type: StructMemberType,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ParsedFrom {
    Hexadecimal,
    Decimal,
    Binary,
    Character,
    Expression,
}

#[derive(Debug, PartialEq, Clone)]
enum CpuSpecific {
    Cpu6809(NodeKind6809),
    Cpu6800(NodeKind6800),
}

///Ast Node Items
#[derive(Debug, PartialEq, Clone)]
pub enum Item<C>
where
    C: PartialEq + Clone + std::fmt::Debug,
{
    CpuSpecific(C),
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

    StructDef(String),
    StructEntry(String),

    SetPc(usize),
    SetPutOffset(isize),

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
    Block,
}

impl<C> Item<C>
where
    C: std::fmt::Debug + Clone + PartialEq,
{
    pub fn zero() -> Self {
        Item::Num(0, ParsedFrom::Expression)
    }

    pub fn from_number(n: i64, p: ParsedFrom) -> Self {
        Item::Num(n, p)
    }

    pub fn is_expr(&self) -> bool {
        matches!(self, Self::Expr | Self::BracketedExpr)
    }
    pub fn is_number(&self) -> bool {
        matches!(self, Self::Num(..))
    }

    pub fn unrwap_number(&self) -> Option<i64> {
        if let Item::Num(n, _) = self {
            Some(*n)
        } else {
            None
        }
    }

    pub fn unwrap_macro_def(&self) -> Option<(&String, &[String])> {
        if let Item::MacroDef(name, params) = self {
            Some((name, params))
        } else {
            None
        }
    }

    pub fn unwrap_include(&self) -> Option<&PathBuf> {
        if let Item::Include(n) = self {
            Some(n)
        } else {
            None
        }
    }
    pub fn unwrap_label_text(&self) -> Option<&str> {
        use Item::*;
        match self {
            Label(x) | LocalLabel(x) => x.get_text(),
            _ => None,
        }
    }

    pub fn unwrap_label_id(&self) -> Option<SymbolScopeId> {
        use Item::*;
        use LabelDefinition::Scoped;

        match self {
            Label(Scoped(id)) | LocalLabel(Scoped(id)) => Some(*id),
            _ => None,
        }
    }
}

pub fn join_vec<I: Display>(v: &[I], sep: &str) -> String {
    let ret: Vec<_> = v.iter().map(|x| x.to_string()).collect();
    ret.join(sep)
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
