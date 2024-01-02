#![forbid(unused_imports)]
use grl_sources::Position;
use std::{fmt::Display, path::PathBuf, str::FromStr};
use thin_vec::ThinVec;

use crate::{semantic::AstNodeId, error::ParseError, gazmsymbols::SymbolScopeId};

use super::{
    item6809::MC6809::{self, OpCode, SetDp},
    BaseNode, CtxTrait,
};

impl CtxTrait for Position {}

pub type Node = BaseNode<Item, Position>;

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
    pub fn to_size_item(&self) -> Item {
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

#[derive(Debug, PartialEq, Hash, Clone)]
pub enum LabelDefinition {
    Text(String),
    TextScoped(String),
    Scoped(SymbolScopeId),
}

impl LabelDefinition {
    pub fn get_text(&self) -> Option<&str> {
        match self {
            LabelDefinition::TextScoped(x) | LabelDefinition::Text(x) => Some(x),
            LabelDefinition::Scoped(_) => None,
        }
    }

    pub fn get_id(&self) -> Option<SymbolScopeId> {
        use LabelDefinition::*;
        match self {
            TextScoped(..) | LabelDefinition::Text(..) => None,
            Scoped(id) => Some(*id),
        }
    }

    pub fn map_string<F>(&self, f: F) -> Self
    where
        F: FnOnce(&str) -> String,
    {
        use LabelDefinition::*;
        match self {
            TextScoped(x) => LabelDefinition::TextScoped(f(x)),
            Text(x) => LabelDefinition::Text(f(x)),
            Scoped(_) => self.clone(),
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            LabelDefinition::TextScoped(x) | LabelDefinition::Text(x) => x.clone(),
            LabelDefinition::Scoped(x) => format!("{x:?}"),
        }
    }
}

impl From<SymbolScopeId> for LabelDefinition {
    fn from(value: SymbolScopeId) -> Self {
        Self::Scoped(value)
    }
}

impl std::fmt::Display for LabelDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use LabelDefinition::*;
        match self {
            Scoped(x) => write!(f, "Scoped({},{})", x.scope_id, x.symbol_id),
            TextScoped(x) => write!(f, "{x}"),
            Text(x) => write!(f, "{x}"),
        }
    }
}

///Ast Node Items
#[derive(Debug, PartialEq, Clone)]
pub enum Item {
    Import,
    Cpu(MC6809),
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

impl From<Item> for grl_sources::ItemType {
    fn from(value: Item) -> Self {
        use grl_sources::ItemType::*;
        match value {
            Item::Cpu(m) => match m {
                MC6809::Operand(..) => Other,
                MC6809::RegisterSet(..) => Other,
                MC6809::OperandIndexed(..) | MC6809::OpCode(..) => OpCode,
                MC6809::SetDp => Command,
            },
            _ => Other,
        }
    }
}

impl Item {
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

impl BaseNode<Item, Position> {
    pub fn from_item_pos<P: Into<Position>>(item: Item, p: P) -> Self {
        Self::new(item, p.into())
    }

    pub fn from_number_pos<P: Into<Position>>(n: i64, pos: P) -> Self {
        Self::new(Item::Num(n, ParsedFrom::Expression), pos.into())
    }
    pub fn with_pos(self, sp: Position) -> Self {
        let mut ret = self;
        ret.ctx = sp;
        ret
    }
}

pub fn join_vec<I: Display>(v: &[I], sep: &str) -> String {
    let ret: Vec<_> = v.iter().map(|x| x.to_string()).collect();
    ret.join(sep)
}

impl Display for BaseNode<Item, Position> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Item::*;

        let item = &self.item;

        let join_children = |sep| join_vec(&self.children, sep);

        let ret: String = match item {
            AssignmentFromPc(name) | LocalAssignmentFromPc(name) => {
                format!("{name} equ *")
            }

            Pc => "*".to_string(),

            Label(name) | LocalLabel(name) => {
                format!("{name}")
            }

            Comment(comment) => comment.clone(),
            // QuotedString(test) => format!("\"{}\"", test),
            // Register(r) => r.to_string(),

            // RegisterList(vec) => join_vec(vec, ","),
            LocalAssignment(name) | Assignment(name) => {
                format!("{} equ {}", name, self.children[0])
            }

            Expr => join_children(""),

            Include(file) => format!("include \"{}\"", file.to_string_lossy()),

            Num(n, p) => match &p {
                ParsedFrom::Hexadecimal => format!("${n:x}"),
                ParsedFrom::Expression | ParsedFrom::Decimal | ParsedFrom::Character => {
                    n.to_string()
                }
                ParsedFrom::Binary => format!("%{n:b}"),
            },
            UnaryTerm => join_children(""),

            Mul => "*".to_string(),
            Div => "/".to_string(),
            Add => "+".to_string(),
            Sub => "-".to_string(),
            BitAnd => "&".to_string(),
            BitOr => "|".to_string(),
            BitXor => "^".to_string(),
            Org => {
                format!("org {}", self.children[0])
            }

            BracketedExpr => {
                format!("({})", join_children(""))
            }

            TokenizedFile(file, ..) => {
                let header = format!("; included file {}", file.to_string_lossy());
                let children: Vec<String> = self.children.iter().map(|n| format!("{n}")).collect();
                format!("{}\n{}", header, children.join("\n"))
            }

            Cpu(OpCode(txt, _ins, addr_type)) => {
                format!("{txt} {addr_type:?}")
            }

            Cpu(SetDp) => {
                let children: Vec<String> = self.children.iter().map(|n| format!("{n}")).collect();
                children.join("\n")
            }

            _ => format!("{item:?} not implemented"),
        };

        write!(f, "{ret}")
    }
}

////////////////////////////////////////////////////////////////////////////////
