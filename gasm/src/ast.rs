
pub type AstTree = ego_tree::Tree<ItemWithPos>;
pub type AstNodeRef<'a> = ego_tree::NodeRef<'a,ItemWithPos>;
pub type AstNodeId = ego_tree::NodeId;
pub type AstNodeMut<'a> = ego_tree::NodeMut<'a,ItemWithPos>;

use std::collections::{HashMap, VecDeque};
use std::error::Error;
// use std::fmt::{Debug, DebugMap};
use std::hash::Hash;
use std::path::{PathBuf, Prefix};

use nom::InputIter;
use nom::bytes::complete::take_till;
use romloader::ResultExt;

use crate::item;
use crate::symbols::ScopeBuilder;

use super::item::{Node, Item};
use super::locate::Position;

use super::sourcefile::SourceFile;
use super::postfix;



////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq, Clone)]
pub struct ItemWithPos {
    pub item : Item,
    pub pos : Position,
    pub file_id : Option<AstNodeId>,
    pub id: Option<AstNodeId>,
}

impl ItemWithPos {
    pub fn new(n : &Node) -> Self{
        Self {
            item: n.item().clone(),
            pos : n.ctx().clone(),
            file_id : None,
            id : None,
        }
    }
}

impl From<&Box<Node>> for ItemWithPos {
    fn from(n: &Box<Node>) -> Self {
        Self::new(n.as_ref())
    }
}

impl From<&Node> for ItemWithPos {
    fn from(n: &Node) -> Self {
        Self::new(n)
    }
}

pub fn add_node(parent : &mut AstNodeMut, node: &Node) {
    use super::item::Item::*;
    let ipos = node.into();
    let mut this_node = parent.append(ipos);

    this_node.value().id = Some(this_node.id());

    for n in &node.children {
        add_node(&mut this_node,&n);
    }
}

pub fn make_tree(node : &Node) -> AstTree {
    let mut ret  = AstTree::new(node.into());

    let mut this_node = ret.root_mut();


    this_node.value().id = Some(this_node.id());

    for c in &node.children {
        add_node(&mut ret.root_mut(), &c);
    }
    ret
}

// Add a source file to the hash if this is a source node
// return true if it did
fn get_tokenize_file(t : &AstTree, node_id : AstNodeId) -> Option<SourceFile> {
    t.get(node_id)
        .unwrap()
        .value()
        .item.get_my_tokenized_file().map(|(f,_,s)|
                                          SourceFile::new(f,s))
}

fn set_file_ids(t : &mut AstTree, node_id : AstNodeId, file_node_id : AstNodeId, mapper : &mut HashMap<AstNodeId, SourceFile>) {
    let mut file_node_id = file_node_id;

    if let Some(source) = get_tokenize_file(&t, node_id) {
        file_node_id = node_id;
        mapper.insert(node_id, source);
    }

    let mut node = t.get_mut(node_id).unwrap();
    node.value().file_id = Some(file_node_id);

    let children : Vec<_> = t.get(node_id).unwrap().children().map(|n| n.id()).collect();

    for c in children {
        set_file_ids(t, c, file_node_id, mapper)
    }
}

fn add_file_references(ast : &mut AstTree) -> HashMap<AstNodeId, SourceFile> {
    let root_id = ast.root().id();
    let mut hm = HashMap::new();
    set_file_ids(ast,root_id, root_id, &mut hm);
    hm
}

#[derive(Debug, Clone)]
struct NodeSourceInfo<'a> {
    fragment: &'a str,
    line_str: &'a str,
    line: usize,
    col: usize,
    source_file : &'a SourceFile,
    file: PathBuf,
}

#[derive(Debug)]
pub struct Ast {
    tree : AstTree,
    id_to_source_file : HashMap<AstNodeId, SourceFile>,
}

enum LabelValues {
    Value(i64),
    Expr(AstNodeId),
    Pc,
}

pub fn debug<F,Y>(text : &str, mut f : F)  -> Y
    where 
    F : FnMut(&mut super::messages::Messages) -> Y
{
        let x = super::messages::messages();
        x.debug(text);
        x.indent();
        let r = f(x);
        x.deindent();
        r
}

pub fn info<F,Y>(text : &str, mut f : F)  -> Y
    where 
    F : FnMut(&mut super::messages::Messages) -> Y
{
        let mut x = super::messages::messages();
        x.info(text);
        x.indent();
        let r = f(&mut x);
        x.deindent();
        r
}

impl std::fmt::Display for Ast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let wrapped = DisplayWrapper { node: self.tree.root() };
        write!(f, "{}", wrapped)
    }
}

impl Ast {
    pub fn from_nodes(n : Node) -> Self {
        let (t,id) = info("Building Ast from nodes", |_| {

            let mut tree = info("Building AST", |_| {
                make_tree(&n)
            });

            let id_to_source_file = info("Resolving file references", |_| {
                add_file_references(&mut tree)
            });

            (tree, id_to_source_file)
        });

        Self::new(t, id)
    }

    pub fn new(tree : AstTree, id_to_source_file : HashMap<AstNodeId, SourceFile> ) -> Self {

        let mut ret = Self {
            tree,
            id_to_source_file,
        };

        ret.rename_locals();

        let _ = ret.postfix_expressions();

        ret.symbolize();

        ret
    }

    pub fn get_tree(&self) -> &AstTree {
        &self.tree
    }

    fn get_source_info_from_value<'a>(&'a self, v : &ItemWithPos)-> Result<NodeSourceInfo<'a>, String> {
        let pos = &v.pos;
        let file_id = v.file_id.ok_or("No file id!".to_string())?;

        let source_file = self.id_to_source_file.get(&file_id).ok_or(format!("Can't find file id {:?} {:?}", file_id, self.id_to_source_file))?;
        let fragment = source_file.get_span(pos)?;
        let line_str = source_file.get_line(pos)?;

        let ret = NodeSourceInfo {
            line_str,
            col : pos.col,
            line: pos.line,
            fragment,
            source_file,
            file : source_file.file.clone(),
        };

        Ok(ret)
    }

    fn get_source_info_from_node<'a>(&'a self, node : &'a AstNodeRef)-> Result<NodeSourceInfo<'a>, String> {
        self.get_source_info_from_value(node.value())
    }

    fn rename_locals(&mut self) {
        use Item::*;

        info("Scoping locals into globals", |x| {
            let mut scopes = ScopeBuilder::new();

            let rename = |fqn : &String, name: &String| {
                let ret = format!("{}/{}", fqn, name);
                x.debug(&format!("{} -> {}", name, ret));
                ret
            };

            // Expand all local labels to have a scoped name
            // and change all locals to globals

            for v in self.tree.values_mut() {
                match &v.item {
                    AssignmentFromPc(name) => {
                        scopes.pop();
                        scopes.push_new(name);
                    },

                    LocalAssignmentFromPc(name) => {
                        let new_name = rename(&scopes.get_current_fqn(), name);
                        v.item = AssignmentFromPc(new_name);
                    },

                    LocalAssignment(name) => {
                        let new_name = rename(&scopes.get_current_fqn(), name);
                        v.item = Assignment(new_name);
                    }

                    LocalLabel(name)=> {
                        let new_name = rename(&scopes.get_current_fqn(), name);
                        v.item = Label(new_name);
                    },

                    TokenizedFile(_,_,_) => {
                        scopes.pop();
                    },

                    _ =>()
                };
            }
        });
    }

    fn infix_to_postfix(&self, _nodes : Vec<AstNodeId> )-> Result<Vec<AstNodeId>,String> {
        todo!()
    }

    fn node_to_postfix(&self, node : AstNodeRef) -> Result<Vec<AstNodeId>,String> {
        use postfix::PostFixer;

        let args = node.children().map(|n| Term::new(&n)).collect();

        let mut pfix : PostFixer<Term> = postfix::PostFixer::new();
        let ret = pfix.to_postfix(args);

        let ret = ret.iter().map(|t| t.node).collect();

        Ok(ret)
    }

    fn postfix_expressions(&mut self) -> Result<(),String>{
        info("Converting expressions to poxtfix", |x| {
            use Item::*;

            let mut to_convert : Vec<( AstNodeId, Vec<AstNodeId> )> = vec![];

            // find all of the nodes that need converting
            for n in self.tree.nodes() {
                let v = n.value();

                if let Expr = v.item {
                    let new_order = self.node_to_postfix(n)?;
                    to_convert.push((n.id(), new_order));
                }
            }

            for (parent,new_children) in &to_convert {
                for c in new_children {
                    let mut c = self.tree.get_mut(*c).ok_or("Illegal node value")?;
                    c.detach();
                }

                let mut p = self.tree.get_mut(*parent).ok_or("Illegal node value")?;

                for c in new_children {
                    p.append_id(*c);
                }

                p.value().item = PostFixExpr;
            }

            x.debug(&format!("Converted {} expression(s)", to_convert.len()));

            Ok(())
        })

    }

    fn symbolize(&mut self) {
        use super::eval::Evaluator;
        use Item::*;


        info("Working out assignment dependencies", |x| {
            let mut assignment_to_node : HashMap<String, AstNodeRef> = HashMap::new();

            for n in self.tree.nodes() {
                match &n.value().item {
                    Assignment(name) => {
                        assignment_to_node.insert(name.clone(), n);
                    }
                    _ => ()
                }
            }

            for (name, n) in assignment_to_node.iter() {
                let this_id = n.id();
                let mut edges : Vec<_> = vec![];
                for c in n.descendants() {
                    match &c.value().item {
                        Item::Label(label_name) => {
                            let msg = format!("( {} {} )", name, label_name);
                            x.debug(&msg);
                            let edge = (c.id(),this_id);
                            edges.push(edge)
                        }
                        _ => ()
                    }
                }
            }
            x.error("TODO");


        });

        // let mut eval = Evaluator::new();
        // info("Evaluate assignments", |x| {
        //     // Evaluate all of the assignments
        //     for n in self.tree.nodes() {
        //         match &n.value().item {
        //             Assignment(name) => {
        //                 match eval.eval(&n) {
        //                     Ok(res) => {
        //                         let si = self.get_source_info_from_node(&n).expect("Cannot find node");
        //                         let inf = format!("{} -> {:?}",si.fragment, res);
        //                         x.debug(&inf);
        //                     },
        //                     Err(e) => {
        //                         let err = format!("Couldn't evaluate assignment: {}", e);
        //                         x.error(&err);
        //                     }
        //                 }
        //             },
        //             _ => ()
        //         }
        //     }
        // });

        //
    }
}








////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq)]
struct Term {
    node : AstNodeId,
    priority: Option<usize>
}

impl postfix::GetPriotity for Term {
    fn priority(&self) -> Option<usize> {
        self.priority
    }
}

pub fn to_priority(i : &Item) -> Option<usize> {
    use Item::*;
    match i {
        Div => Some( 5 ),
        Mul => Some( 5 ),
        Add => Some( 4 ),
        Sub => Some( 4 ),
        And => Some( 2 ),
        Or => Some( 2 ),
        Xor => Some(2),
        _ => None,
    }
}

impl Term {
    pub fn new(node: &AstNodeRef) -> Self {
        Self {
            node : node.id(),
            priority: to_priority(&node.value().item)
        }
    }

}


////////////////////////////////////////////////////////////////////////////////
use std::fmt::Display;
pub fn join_vec<I : Display>(v : &Vec<I>, sep : &str) -> String {
    let ret : Vec<_> = v.iter().map(|x| x.to_string()).collect();
    ret.join(sep)

}

struct DisplayWrapper<'a> {
    node : AstNodeRef<'a>
}

impl<'a> Into<DisplayWrapper<'a>> for AstNodeRef<'a> {
    fn into(self) -> DisplayWrapper<'a> {
        DisplayWrapper {
            node: self
        }
    }
}

impl<'a> std::fmt::Display for DisplayWrapper<'a> {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Item::*;

        let node = self.node;
        let item = &node.value().item;

        let to_string = | n : AstNodeRef | -> String {
            let x : DisplayWrapper = n.into();
            x.to_string()
        };

        let child = |n : usize| {
            let v = node.children().nth(n).unwrap();
            to_string(v)
        };

        let join_kids = |sep| {
            let v : Vec<_> = node.children().map(to_string).collect();
            v.join(sep)
        };

        let ret : String = match item {
            LocalAssignmentFromPc(name) | AssignmentFromPc(name) => {
                format!("{} equ {}", name, child(0))
            },

            Pc => "*".to_string(),

            Label(name) | LocalLabel(name) => name.clone(),

            Comment(comment) => comment.clone(),
            QuotedString(test) => format!("\"{}\"", test),
            Register(r) => r.to_string(),

            RegisterList(vec) => {
                let vec : Vec<_> = vec.iter().map(|r| r.to_string()).collect();
                vec.join(",")
            },

            LocalAssignment(name) |
                Assignment(name) => {
                    format!("{} equ {}", name, child( 0 ))
                },

            Expr => {
                join_kids("")
            }

            PostFixExpr => {
                join_kids(" ")
            }

            Include(file) => format!("include \"{}\"",file.to_string_lossy()),

            Number(n) => {
                n.to_string()
            }
            UnaryMinus => "-".to_string(),
            UnaryTerm => {
                panic!()
            }

            Mul => "*".to_string(),
            Div => "/".to_string(),
            Add => "+".to_string(),
            Sub => "-".to_string(),
            And => "&".to_string(),
            Or => "|".to_string(),
            Xor => "^".to_string(),
            Org => {
                format!("org {}", child( 0 ) )
            },

            BracketedExpr => {
                format!("({})", join_kids(""))
            }

            TokenizedFile(_, _, _) => {
                join_kids("\n")
            }

            OpCode(ins) => {
                use emu::isa::AddrModeEnum::*;

                let operand = match ins.addr_mode {
                    Immediate8 | Immediate16 => format!("#{}", child(0)),
                    Direct => format!("<{}", child(0)),
                    Inherent => "".to_string(),
                    _ => format!("{:?} NOT IMPLEMENTED", ins.addr_mode)
                };

                format!("{} {}", ins.action, operand)
            }

            _ => format!("{:?} not implemented", item)
        };

        write!(f, "{}", ret)
    }
}

