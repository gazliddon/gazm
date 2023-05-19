use std::fmt::Display;
use std::{collections::HashSet, path::PathBuf};
use thin_vec::ThinVec;

use emu::{
    cpu::{IndexedFlags, RegEnum},
    isa::Instruction,
    utils::sources::Position,
};

use symbols::ScopedName;

use crate::{
    ast::AstNodeId,
    error::ParseError,
    item6809::{
        self,
        MC6809::{self, OpCode, SetDp},
    },
    node::{BaseNode, CtxTrait},
    parse::locate::{span_to_pos, Span},
    symbols::SymbolScopeId,
};

impl<'a> CtxTrait for Span<'a> {}

pub type Node = BaseNode<Item, Position>;

#[derive(Debug, PartialEq, Clone)]
pub enum StructMemberType {
    Byte,
    Word,
    DWord,
    QWord,
    UserType(String),
}

impl StructMemberType {
    pub fn to_size_item(&self) -> Item {
        use Item::*;
        match self {
            Self::Byte => Number(1, ParsedFrom::FromExpr),
            Self::Word => Number(2, ParsedFrom::FromExpr),
            Self::DWord => Number(4, ParsedFrom::FromExpr),
            Self::QWord => Number(8, ParsedFrom::FromExpr),
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
    Hex,
    Dec,
    Bin,
    Char(char),
    FromExpr,
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

    pub fn to_string(&self) -> String {
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
        match self {
            LabelDefinition::Scoped(x) => write!(f, "Scoped({},{})", x.scope_id, x.symbol_id),
            LabelDefinition::TextScoped(x) => write!(f, "{x}"),
            LabelDefinition::Text(x) => write!(f, "{x}"),
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
    Number(i64, ParsedFrom),

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
    ShiftRight,
    ShiftLeft,
    UnaryGreaterThan,
}

impl Item {
    pub fn from_number(n: i64, p: ParsedFrom) -> Self {
        Item::Number(n, p)
    }

    pub fn is_expr(&self) -> bool {
        matches!(self, Self::Expr | Self::BracketedExpr)
    }

    pub fn unrwap_number(&self) -> Option<i64> {
        if let Item::Number(n, _) = self {
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
        use LabelDefinition::*;
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
    pub fn from_item_pos(item: Item, p: Position) -> Self {
        Self::new(item, p)
    }

    pub fn from_item_span<I: Into<Item>>(item: I, sp: Span) -> Self {
        Self::new(item.into(), span_to_pos(sp))
    }

    pub fn from_number(n: i64, _p: ParsedFrom, sp: Span) -> Self {
        Self::from_item_span(Item::Number(n, _p), sp)
    }

    pub fn with_span(self, sp: Span) -> Self {
        let mut ret = self;
        ret.ctx = span_to_pos(sp);
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

            Number(n, p) => match &p {
                ParsedFrom::Hex => format!("${n:x}"),
                ParsedFrom::FromExpr | ParsedFrom::Dec => n.to_string(),
                ParsedFrom::Char(c) => format!("'{c}'"),
                ParsedFrom::Bin => format!("%{n:b}"),
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

            TokenizedFile(file, _) => {
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
