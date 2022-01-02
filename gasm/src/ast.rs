
type AstTree = ego_tree::Tree<ItemWithPos>;
type AstNodeRef<'a> = ego_tree::NodeRef<'a,ItemWithPos>;
type AstNodeId = ego_tree::NodeId;
type AstNodeMut<'a> = ego_tree::NodeMut<'a,ItemWithPos>;

use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::fmt::{Debug, DebugMap};
use std::hash::Hash;
use std::path::PathBuf;
use std::sync::mpsc::channel;

use romloader::ResultExt;

use crate::symbols::ScopeBuilder;

use super::item::{Node, Item};
use super::locate::Position;

////////////////////////////////////////////////////////////////////////////////
struct SourceFile {
    file : PathBuf,
    source: String,
}

impl SourceFile {
    pub fn new(file : &PathBuf, source: &String) -> Self {
        Self {file : file.clone(), source: source.clone()}
    }

    fn get_span(&self,p : &Position) -> Option<&str> {
        let start = p.start;
        let end = p.end;
        let len = self.source.len();

        if start < len &&  end < len {
            Some( &self.source[start..end] )
        } else {

            println!("Massive error");
            println!("{}", len);
            println!("{:?}", p);
            None
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq, Clone)]
pub struct ItemWithPos {
    pub item : Item,
    pub pos : Position,
    pub file_id : Option<AstNodeId>
}

impl ItemWithPos {
    pub fn new(n : &Node) -> Self{
        Self {
            item: n.item().clone(),
            pos : n.ctx().clone(),
            file_id : None,
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

pub fn add_node(tree : &mut AstNodeMut, node: &Node) {
    use super::item::Item::*;

    let ipos = node.into();
    let mut this_node = tree.append(ipos);

    for n in &node.children {
        add_node(&mut this_node,&n);
    }
}

pub fn make_tree(node : &Node) -> AstTree {
    let mut ret  = AstTree::new(node.into());
    add_node(&mut ret.root_mut(), node);
    ret
}

// Add a source file to the hash if this is a source node
// return true if it did
fn create_source_file(t : &AstTree, node_id : AstNodeId) -> Option<SourceFile> {
    t.get(node_id)
        .unwrap()
        .value()
        .item.get_my_tokenized_file().map(|(f,_,s)|
                                          SourceFile::new(f,s))
}

fn set_file_ids(t : &mut AstTree, node_id : AstNodeId, file_node_id : AstNodeId, mapper : &mut HashMap<AstNodeId, SourceFile>) {
    let mut node = t.get_mut(node_id).unwrap();
    node.value().file_id = Some(file_node_id);

    let children : Vec<_> = t.get(node_id).unwrap().children().map(|n| n.id()).collect();

    for c in children {
        if let Some(source) = create_source_file(&t, c) {
            mapper.insert(c, source);
            set_file_ids(t, c, c, mapper)
        } else {
            set_file_ids(t, c, file_node_id, mapper)
        }
    }
}

fn add_file_references(ast : &mut AstTree) -> HashMap<AstNodeId, SourceFile> {
    let root_id = ast.root().id();
    let mut hm = HashMap::new();
    set_file_ids(ast,root_id, root_id, &mut hm);
    hm
}

struct NodeSourceInfo<'a> {
    fragment: &'a str,
    line: usize,
    col: usize,
    id : AstNodeId,
    source_file : &'a SourceFile,
    file: PathBuf,
}

pub struct Ast {
    tree : AstTree,
    id_to_source_file : HashMap<AstNodeId, SourceFile>,
}

enum LabelValues {
    Value(i64),
    Expr(AstNodeId),
    Pc,
}

impl Debug for SourceFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut x = f.debug_struct("SourceFile");
        x.field("file", &self.file.to_string_lossy());
        x.finish()
    }
}

impl Ast {
    pub fn from_nodes(n : Node) -> Self {
        let x = super::messages::messages();

        x.debug("Building AST");
        x.indent();
        let mut tree = make_tree(&n);
        x.deindent();

        x.debug("Resolving file references");
        x.indent();
        let id_to_source_file = add_file_references(&mut tree);
        x.deindent();

        let mut ret = Self {
            tree,
            id_to_source_file,
        };

        x.debug("Scoping local labels");
        x.indent();
        ret.rename_locals();
        x.deindent();

        ret
    }

    pub fn get_tree(&self) -> &AstTree {
        &self.tree
    }

    fn get_source_info_from_node<'a>(&'a self, node : &'a AstNodeRef)-> Result<NodeSourceInfo<'a>, ()> {
        let v= node.value();
        let pos = &v.pos;
        let file_id = v.file_id.ok_or(())?;

        let source_file = self.id_to_source_file.get(&file_id).ok_or(())?;
        let fragment = source_file.get_span(pos).ok_or(())?;

        let ret = NodeSourceInfo {
            col : pos.col,
            line: pos.line,
            id : node.id(),
            fragment,
            source_file,
            file : source_file.file.clone(),
        };

        Ok(ret)
    }

    fn rename_locals(&mut self) {
        
        use Item::*;
        let mut scopes = ScopeBuilder::new();
        let node = self.tree.root().id();

        rename_locals(&mut self.tree, node, &mut scopes);

        let x = super::messages::messages();

        for n in self.tree.nodes() {
            let v = &n.value();

            match &v.item {
                LocalAssignmentFromPc(name) | LocalLabel(name) => {
                    let info = self.get_source_info_from_node(&n).unwrap();
                    let msg = format!("{} -> {}",info.fragment, name,);
                    x.debug(&msg);
                },
                _ => ()
            }
        }

    }

    fn symbolise(&mut self) {
    }
}

fn rename_locals(ast : &mut AstTree, id : AstNodeId, scopes : &mut ScopeBuilder) {
    use super::item::Item::*;

    let mut n = ast.get_mut(id).unwrap();
    let mut v = n.value();

    match &v.item {
        AssignmentFromPc(name) => {
            scopes.pop();
            scopes.push_new(name);
        },

        LocalAssignmentFromPc(name) => {
            let new_name = format!("{}/{}", scopes.get_current_fqn(), name);
            v.item = LocalAssignmentFromPc(new_name);
        },

        LocalLabel(name)=> {
            let new_name = format!("{}/{}", scopes.get_current_fqn(), name);
            v.item = LocalLabel(new_name);
        },

        TokenizedFile(_,_,_) => {
            scopes.pop();
        },

        _ =>()
    };

    let n = ast.get(id).unwrap();
    let ids : Vec<AstNodeId> = n.children().map(|n| n.id()).collect();

    for id in ids {
        rename_locals(ast, id, scopes)
    }
}

