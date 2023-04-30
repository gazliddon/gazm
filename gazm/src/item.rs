use std::fmt::Display;
use std::{collections::HashSet, path::PathBuf};
use thin_vec::ThinVec;

use crate::ast::AstNodeId;
use crate::error::ParseError;
use crate::locate::span_to_pos;
use crate::locate::Span;
use crate::macros::MacroCall;
use crate::node::{BaseNode, CtxTrait};
use emu::cpu::{IndexedFlags, RegEnum};
use emu::isa::Instruction;
use emu::utils::sources::Position;

use emu::utils::sources::SymbolScopeId;

impl<'a> CtxTrait for Span<'a> {}

pub type Node = BaseNode<Item, Position>;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum IndexParseType {
    ConstantOffset(RegEnum), //             arg,R
    Plus(RegEnum),           //             ,R+                    2 0 |
    PlusPlus(RegEnum),       //             ,R++                   3 0 |
    Sub(RegEnum),            //             ,-R                    2 0 |
    SubSub(RegEnum),         //             ,--R                   3 0 |
    Zero(RegEnum),           //             ,R                     0 0 |
    AddB(RegEnum),           //             (+/- B),R              1 0 |
    AddA(RegEnum),           //             (+/- A),R              1 0 |
    AddD(RegEnum),           //             (+/- D),R              4 0 |
    PCOffset,                //             (+/- 7 bit offset),PC  1 1 |
    ExtendedIndirect,        //  [expr]
    Constant5BitOffset(RegEnum, i8),
    ConstantByteOffset(RegEnum, i8),
    ConstantWordOffset(RegEnum, i16),
    PcOffsetWord(i16),
    PcOffsetByte(i8),
}

impl IndexParseType {
    pub fn has_operand(&self) -> bool {
        use IndexParseType::*;

        match self {
            ConstantOffset(..) => true, //             arg,R
            Plus(..) => false,          //             ,R+                    2 0 |
            PlusPlus(..) => false,      //             ,R++                   3 0 |
            Sub(..) => false,           //             ,-R                    2 0 |
            SubSub(..) => false,        //             ,--R                   3 0 |
            Zero(..) => false,          //             ,R                     0 0 |
            AddB(..) => false,          //             (+/- B),R              1 0 |
            AddA(..) => false,          //             (+/- A),R              1 0 |
            AddD(..) => false,          //             (+/- D),R              4 0 |
            PCOffset => true,           //             (+/- 7 bit offset),PC  1 1 |
            ExtendedIndirect => true,   //  [expr]
            Constant5BitOffset(..) => true,
            ConstantByteOffset(..) => true,
            ConstantWordOffset(..) => true,
            PcOffsetWord(..) => true,
            PcOffsetByte(..) => true,
        }
    }
}

fn rbits(r: RegEnum) -> u8 {
    let rnum = {
        match r {
            RegEnum::X => 0,
            RegEnum::Y => 1,
            RegEnum::U => 2,
            RegEnum::S => 3,
            _ => panic!("internal error"),
        }
    };

    rnum << 5
}

fn add_reg(bits: u8, r: RegEnum) -> u8 {
    (bits & !(3 << 5)) | rbits(r)
}

fn add_ind(bits: u8, ind: bool) -> u8 {
    let ind_bit = IndexedFlags::IND.bits();
    let ind_val = if ind { ind_bit } else { 0u8 };

    (bits & !ind_bit) | ind_val
}

impl IndexParseType {
    pub fn get_index_byte(&self, indirect: bool) -> u8 {
        use IndexParseType::*;

        match *self {
            Plus(r) => {
                let mut bits = 0b1000_0000;
                bits = add_reg(bits, r);
                bits
            }

            PlusPlus(r) => {
                let mut bits = 0b1000_0001;
                bits = add_reg(bits, r);
                bits = add_ind(bits, indirect);
                bits
            }

            Sub(r) => {
                let mut bits = 0b1000_0010;
                bits = add_reg(bits, r);
                bits
            }

            SubSub(r) => {
                let mut bits = 0b1000_0011;
                bits = add_reg(bits, r);
                bits = add_ind(bits, indirect);
                bits
            }

            Zero(r) => {
                let mut bits = 0b1000_0100;
                bits = add_reg(bits, r);
                bits = add_ind(bits, indirect);
                bits
            }

            AddA(r) => {
                let mut bits = 0b1000_0110;
                bits = add_reg(bits, r);
                bits = add_ind(bits, indirect);
                bits
            }

            AddB(r) => {
                let mut bits = 0b1000_0101;
                bits = add_reg(bits, r);
                bits = add_ind(bits, indirect);
                bits
            }

            AddD(r) => {
                let mut bits = 0b1000_1011;
                bits = add_reg(bits, r);
                bits = add_ind(bits, indirect);
                bits
            }

            PcOffsetByte(_) => {
                let mut bits = 0b1000_1100;
                bits = add_ind(bits, indirect);
                bits
            }

            PcOffsetWord(_) => {
                let mut bits = 0b1000_1101;
                bits = add_ind(bits, indirect);
                bits
            }

            ExtendedIndirect => 0b1001_1111,

            Constant5BitOffset(r, off) => {
                let mut bits = 0b0000_0000;
                bits = add_reg(bits, r);
                bits = add_ind(bits, indirect);
                bits |= off as u8 & 0x1f;
                bits
            }

            ConstantByteOffset(r, _) => {
                let mut bits = 0b1000_1000;
                bits = add_reg(bits, r);
                bits = add_ind(bits, indirect);
                bits
            }

            ConstantWordOffset(r, _) => {
                let mut bits = 0b1000_1001;
                bits = add_reg(bits, r);
                bits = add_ind(bits, indirect);
                bits
            }

            PCOffset | ConstantOffset(..) => panic!("Internal error"),
        }
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum AddrModeParseType {
    Indexed(IndexParseType, bool),
    Direct,
    Extended(bool), // if set then extended mode was forced, do not opt for DP
    Relative,
    Inherent,
    Immediate,
    RegisterSet,
    RegisterPair(RegEnum, RegEnum),
}

impl AddrModeParseType {
    pub fn has_operand(&self) -> bool {
        use AddrModeParseType::*;

        match self {
            Direct => true,
            Extended(..) => true,
            Relative => true,
            Inherent => false,
            Immediate => true,
            RegisterSet => true,
            RegisterPair(..) => false,
            Indexed(x, _) => x.has_operand(),
        }
    }
}

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
            Self::UserType(name) => Label(format!("{name}.size").into()),
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
    Scoped(SymbolScopeId),
}

impl From<SymbolScopeId> for LabelDefinition {
    fn from(value: SymbolScopeId) -> Self {
        Self::Scoped(value)
    }
}
impl From<&String> for LabelDefinition {
    fn from(value: &String) -> Self {
        Self::Text(value.clone())
    }
}
impl From<String> for LabelDefinition {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl std::fmt::Display for LabelDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LabelDefinition::Scoped(x) => write!(f, "Scoped({},{})", x.scope_id, x.scope_id),
            LabelDefinition::Text(x) => write!(f, "Text({x})"),
        }
    }
}


///Ast Node Items
#[derive(Debug, PartialEq, Clone)]
pub enum Item {
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
    Pc,

    UnaryTerm,

    RegisterSet(HashSet<RegEnum>),

    Label(LabelDefinition),
    LocalLabel(LabelDefinition),

    Comment(String),
    Number(i64, ParsedFrom),

    OpCode(String, Instruction, AddrModeParseType),
    Operand(AddrModeParseType),
    OperandIndexed(IndexParseType, bool),
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
    SetDp,

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
    pub fn operand_from_index_mode(imode: IndexParseType, indirect: bool) -> Self {
        Self::OperandIndexed(imode, indirect)
    }

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
}

impl BaseNode<Item, Position> {
    pub fn from_item_pos(item: Item, p: Position) -> Self {
        Self::new(item, p)
    }

    pub fn from_item_span(item: Item, sp: Span) -> Self {
        Self::new(item, span_to_pos(sp))
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

            Label(LabelDefinition::Text(name)) | LocalLabel(LabelDefinition::Text(name)) => {
                name.clone()
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

            OpCode(txt,_ins, addr_type) => {
                format!("{txt} {:?}",  addr_type)
            }

            TokenizedFile(file, _) => {
                let header = format!("; included file {}", file.to_string_lossy());
                let children: Vec<String> = self.children.iter().map(|n| format!("{n}")).collect();
                format!("{}\n{}", header, children.join("\n"))
            }

            SetDp => {
                let children: Vec<String> = self.children.iter().map(|n| format!("{n}")).collect();
                children.join("\n")
            }

            _ => format!("{item:?} not implemented"),
        };

        write!(f, "{ret}")
    }
}
