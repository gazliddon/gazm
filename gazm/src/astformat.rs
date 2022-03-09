use crate::ast::*;
use crate::item::{self, Item};

impl<'a> std::fmt::Display for Ast<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let wrapped = DisplayWrapper {
            node: self.get_tree().root(),
        };
        write!(f, "{}", wrapped)
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
            let v = node.children().nth(n).unwrap();
            to_string(v)
        };

        let join_kids = |sep| {
            let v: Vec<_> = node.children().map(to_string).collect();
            v.join(sep)
        };

        let ret: String = match item {
            LocalAssignmentFromPc(name) | AssignmentFromPc(name) => {
                format!("{} equ {}", name, child_string(0))
            }

            Pc => "*".to_string(),

            Label(name) => format!("Label: {name}"),
            LocalLabel(name) => format!("LocalLable: {name}"),

            Comment(comment) => comment.clone(),

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

            Number(n) => n.to_string(),
            // UnaryMinus => "-".to_string(),
            UnaryTerm => {
                format!("!{:?} {:?}", child_item(0), child_item(1))
            }

            Mul => "*".to_string(),
            Div => "/".to_string(),
            Add => "+".to_string(),
            Sub => "-".to_string(),
            And => "&".to_string(),
            Or => "|".to_string(),
            Xor => "^".to_string(),

            Org => {
                format!("org {}", child_string(0))
            }

            Put => {
                format!("put {}", child_string(0))
            }

            BracketedExpr => {
                format!("({})", join_kids(""))
            }

            TokenizedFile(_, _) => join_kids("\n"),

            OpCode(ins, item::AddrModeParseType::Inherent) => ins.action.clone(),

            StructDef(name) => {
                let body = join_kids(",\n");
                format!("struct {name} {{\n {body}\n}}")
            }

            StructEntry(name) => {
                format!("{name} : {}", child_string(0))
            }

            ShiftRight => ">>".into(),

            ShiftLeft => "<<".into(),

            OpCode(ins, amode) => {
                use item::AddrModeParseType::*;

                let ind = |s: String, indirect: &bool| -> String {
                    if *indirect {
                        format!("[{}]", s)
                    } else {
                        s
                    }
                };

                let operand = match amode {
                    Immediate => format!("#{}", child_string(0)),
                    Direct => format!("<{}", child_string(0)),
                    Extended(..) => child_string(0),
                    Indexed(imode, indirect) => {
                        use item::IndexParseType::*;
                        match imode {
                            ConstantByteOffset(r, v) | Constant5BitOffset(r, v) => {
                                ind(format!("{},{}", v, r), indirect)
                            }

                            ConstantWordOffset(r, v) => ind(format!("{},{}", v, r), indirect),

                            PcOffsetWord(v) => ind(format!("{},PC", v), indirect),
                            PcOffsetByte(v) => ind(format!("{},PC", v), indirect),
                            ConstantOffset(r) => {
                                ind(format!("{},{}", child_string(0), r), indirect)
                            }
                            Zero(r) => ind(format!(",{}", r), indirect),
                            SubSub(r) => ind(format!(",--{}", r), indirect),
                            Sub(r) => format!(",-{}", r),
                            PlusPlus(r) => ind(format!(",{}++", r), indirect),
                            Plus(r) => format!(",{}+", r),
                            AddA(r) => ind(format!("A,{}", r), indirect),
                            AddB(r) => ind(format!("B,{}", r), indirect),
                            AddD(r) => ind(format!("D,{}", r), indirect),
                            PCOffset => ind(format!("{},PC", child_string(0)), indirect),
                            ExtendedIndirect => format!("[{}]", child_string(0)),
                        }
                    }
                    _ => format!("{:?} NOT IMPLEMENTED", ins.addr_mode),
                };

                format!("{} {}", ins.action, operand)
            }
            _ => format!("{:?} not implemented", item),
        };

        write!(f, "{}", ret)
    }
}
