#![forbid(unused_imports)]

use itertools::Itertools;

use crate::ast::*;
use crate::frontend::{
    Item,
    item6809,
    item6809::MC6809::{self, OpCode},
};

impl<'a> std::fmt::Display for AstCtx<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let wrapped = DisplayWrapper {
            node: self.get_tree().root(),
        };
        write!(f, "{wrapped}")
    }
}

struct DisplayWrapper<'a> {
    node: AstNodeRef<'a>,
}

impl<'a> From<AstNodeRef<'a>> for DisplayWrapper<'a> {
    fn from(ast: AstNodeRef<'a>) -> Self {
        Self { node: ast }
    }
}

pub fn as_string(n: AstNodeRef) -> String {
    let x: DisplayWrapper = n.into();
    x.to_string()
}

impl<'a> std::fmt::Display for DisplayWrapper<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Item::*;

        let node = self.node;
        let item = &node.value().item;

        let to_string = |n: AstNodeRef| -> String {
            let x: DisplayWrapper = n.into();
            x.to_string()
        };

        let child_item = |n: usize| node.children().nth(n).map(|x| &x.value().item);

        let child_string = |n: usize| {
            if let Some(v) = node.children().nth(n) {
                to_string(v)
            } else {
                format!("ERR {:?}", node.value().item)
            }
        };

        let join_kids = |sep| {
            let v: Vec<_> = node.children().map(to_string).collect();
            v.join(sep)
        };

        let ret: String = match item {
            LocalAssignmentFromPc(name) => format!("!{name} equ *"),
            AssignmentFromPc(name) => format!("{name} equ *"),

            Pc => "*".to_string(),

            Label(name) => format!("{name}"),
            LocalLabel(name) => format!("!{name}"),
            Comment(comment) => format!("; {comment}"),

            // QuotedString(test) => format!("\"{}\"", test),
            // Register(r) => r.to_string(),
            // RegisterList(vec) => {
            //     let vec: Vec<_> = vec.iter().map(|r| r.to_string()).collect();
            //     vec.join(",")
            // }
            LocalAssignment(name) | Assignment(name) => {
                format!("{} equ {}", name, child_string(0))
            }

            Expr => join_kids(""),

            PostFixExpr => join_kids(" "),

            Include(file) => format!("include \"{}\"", file.to_string_lossy()),

            Num(n, _) => n.to_string(),
            // UnaryMinus => "-".to_string(),
            UnaryTerm => {
                format!("!{:?} {:?}", child_item(0), child_item(1))
            }

            Mul => "*".to_string(),
            Div => "/".to_string(),
            Add => "+".to_string(),
            Sub => "-".to_string(),
            BitAnd => "&".to_string(),
            BitOr => "|".to_string(),
            BitXor => "^".to_string(),

            Org => {
                format!("org {}", child_string(0))
            }

            Put => {
                format!("put {}", child_string(0))
            }

            BracketedExpr => {
                format!("({})", join_kids(""))
            }

            TokenizedFile(..) => {

                format!("TokFile:\n{}", join_kids("\n"))
            }

            Cpu(OpCode(_, ins, item6809::AddrModeParseType::Inherent)) => ins.action.clone(),

            StructDef(name) => {
                let body = join_kids(",\n");
                format!("struct {name} {{\n {body}\n}}")
            }

            StructEntry(name) => {
                format!("{name} : {}", child_string(0))
            }

            ShiftR => ">>".into(),

            ShiftL => "<<".into(),
            Fcc(text) => format!("{text:?}"),
            Fdb(_) | Fcb(_) => {
                format!("fcb {}", join_kids(","))
            }

            Fill => {
                let body = join_kids(",");
                format!("fill {body}")
            }

            Cpu(OpCode(_, instruction, amode)) => {
                use item6809::AddrModeParseType::*;

                let ind = |s: String, indirect: &bool| -> String {
                    if *indirect {
                        format!("[{s}]")
                    } else {
                        s
                    }
                };

                let operand = match amode {
                    RegisterSet => {
                        let rset = &node.first_child().unwrap().value().item;
                        if let Cpu(MC6809::RegisterSet(regs)) = rset {
                            let r = regs
                                .iter()
                                .sorted()
                                .map(|r| r.to_string())
                                .collect_vec()
                                .join(",");
                            r.to_string()
                        } else {
                            panic!("Whut!")
                        }
                    }
                    Immediate => format!("#{}", child_string(0)),
                    Direct => format!("<{}", child_string(0)),
                    Extended(..) => child_string(0),
                    Indexed(index_mode, indirect) => {
                        use item6809::IndexParseType::*;
                        match index_mode {
                            ConstantByteOffset(r, v) | Constant5BitOffset(r, v) => {
                                ind(format!("{v},{r}"), indirect)
                            }

                            ConstantWordOffset(r, v) => ind(format!("{v},{r}"), indirect),
                            PcOffsetWord(v) => ind(format!("{v},PC"), indirect),
                            PcOffsetByte(v) => ind(format!("{v},PC"), indirect),
                            ConstantOffset(r) => ind(format!("{},{r}", child_string(0)), indirect),
                            Zero(r) => ind(format!(",{r}"), indirect),
                            SubSub(r) => ind(format!(",--{r}"), indirect),
                            Sub(r) => format!(",-{r}"),
                            PlusPlus(r) => ind(format!(",{r}++"), indirect),
                            Plus(r) => format!(",{r}+"),
                            AddA(r) => ind(format!("A,{r}"), indirect),
                            AddB(r) => ind(format!("B,{r}"), indirect),
                            AddD(r) => ind(format!("D,{r}"), indirect),
                            PCOffset => ind(format!("{},PC", child_string(0)), indirect),
                            ExtendedIndirect => format!("[{}]", child_string(0)),
                        }
                    }
                    _ => format!("{:?} NOT IMPLEMENTED", instruction.addr_mode),
                };

                format!("{} {operand}", instruction.action)
            }

            _ => format!("{item:?} not implemented"),
        };

        write!(f, "{ret}")
    }
}
