pub type AstTree = ego_tree::Tree<ItemWithPos>;
pub type AstNodeRef<'a> = ego_tree::NodeRef<'a, ItemWithPos>;
pub type AstNodeId = ego_tree::NodeId;
pub type AstNodeMut<'a> = ego_tree::NodeMut<'a, ItemWithPos>;

use std::collections::{HashMap, VecDeque};
use std::error::Error;
// use std::fmt::{Debug, DebugMap};
use std::hash::Hash;
use std::path::{PathBuf, Prefix};

use nom::bytes::complete::take_till;
use nom::InputIter;
use romloader::ResultExt;

use crate::error::{AstError, UserError};
use crate::item;
use crate::scopes::ScopeBuilder;
use crate::symbols::SymbolId;

use crate::item::{Item, Node};
use crate::locate::Position;

use crate::postfix;
use crate::sourcefile::{NodeSourceInfo, SourceFile, Sources};
use crate::symbols::SymbolTable;
use crate::util::{debug, info};

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq, Clone)]
pub struct ItemWithPos {
    pub item: Item,
    pub pos: Position,
    pub file_id: Option<AstNodeId>,
    pub id: Option<AstNodeId>,
}

impl ItemWithPos {
    pub fn new(n: &Node) -> Self {
        Self {
            item: n.item().clone(),
            pos: n.ctx().clone(),
            file_id: None,
            id: None,
        }
    }
}

pub fn add_node(parent: &mut AstNodeMut, node: &Node) {
    use super::item::Item::*;
    let ipos = ItemWithPos::new(node);
    let mut this_node = parent.append(ipos);

    this_node.value().id = Some(this_node.id());

    for n in &node.children {
        add_node(&mut this_node, n);
    }
}

pub fn make_tree(node: &Node) -> AstTree {
    let mut ret = AstTree::new(ItemWithPos::new(node));

    let mut this_node = ret.root_mut();

    this_node.value().id = Some(this_node.id());

    for c in &node.children {
        add_node(&mut ret.root_mut(), c);
    }
    ret
}

#[derive(Debug)]
pub struct Ast {
    pub tree: AstTree,
    pub sources: Sources,
    pub symbols: SymbolTable,
}

impl Ast {
    pub fn from_nodes(n: Node) -> Result<Self, UserError> {
        let (t, sources) = info("Building Ast from nodes", |_| {
            let mut tree = info("Building AST", |_| make_tree(&n));

            let sources = info("Resolving file references", |_| Sources::new(&mut tree));

            (tree, sources)
        });

        Self::new(t, sources)
    }

    pub fn new(tree: AstTree, sources: Sources) -> Result<Self, UserError> {
        let mut ret = Self {
            tree,
            sources,
            symbols: Default::default(),
        };

        ret.rename_locals();

        let _ = ret.postfix_expressions();

        ret.evaluate()?;

        Ok(ret)
    }

    pub fn get_tree(&self) -> &AstTree {
        &self.tree
    }
    pub fn get_tree_mut(&mut self) -> &AstTree {
        &mut self.tree
    }

    fn get_source_info_from_node<'a>(
        &'a self,
        node: &'a AstNodeRef,
    ) -> Result<NodeSourceInfo<'a>, String> {
        self.sources.get_source_info_from_value(node.value())
    }

    fn get_source_info_from_node_id(&self, id: AstNodeId) -> Result<NodeSourceInfo, String> {
        let n = self.tree.get(id).unwrap();
        self.sources.get_source_info_from_value(n.value())
    }

    fn rename_locals(&mut self) {
        use Item::*;

        info("Scoping locals into globals", |x| {
            let mut scopes = ScopeBuilder::new();

            let rename = |fqn: &String, name: &String| {
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
                    }

                    LocalAssignmentFromPc(name) => {
                        let new_name = rename(&scopes.get_current_fqn(), name);
                        v.item = AssignmentFromPc(new_name);
                    }

                    LocalAssignment(name) => {
                        let new_name = rename(&scopes.get_current_fqn(), name);
                        v.item = Assignment(new_name);
                    }

                    LocalLabel(name) => {
                        let new_name = rename(&scopes.get_current_fqn(), name);
                        v.item = Label(new_name);
                    }

                    TokenizedFile(_, _, _) => {
                        scopes.pop();
                    }

                    _ => (),
                };
            }
        });
    }

    fn node_to_postfix(&self, node: AstNodeRef) -> Result<Vec<AstNodeId>, String> {
        use postfix::PostFixer;

        let args = node.children().map(|n| Term::new(&n)).collect();

        let mut pfix: PostFixer<Term> = postfix::PostFixer::new();
        let ret = pfix.get_postfix(args);

        let ret = ret.iter().map(|t| t.node).collect();

        Ok(ret)
    }

    fn postfix_expressions(&mut self) -> Result<(), String> {
        info("Converting expressions to poxtfix", |x| {
            use Item::*;

            let mut to_convert: Vec<(AstNodeId, Vec<AstNodeId>)> = vec![];

            // find all of the nodes that need converting
            for n in self.tree.nodes() {
                let v = n.value();

                if let Expr = v.item {
                    let new_order = self.node_to_postfix(n)?;
                    to_convert.push((n.id(), new_order));
                }
            }

            for (parent, new_children) in &to_convert {
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

    fn convert_error(&self, e: AstError) -> UserError {
        let si = self.get_source_info_from_node_id(e.node_id).unwrap();
        UserError::from_ast_error(e, &si)
    }

    pub fn user_error<S>(&self, msg: S, id: AstNodeId) -> UserError
    where
        S: Into<String>,
    {
        let n = self.tree.get(id).unwrap();
        let e = AstError::from_node(msg, n);
        self.convert_error(e)
    }

    pub fn eval(&self, symbols: &SymbolTable, id: AstNodeId) -> Result<i64, UserError> {
        use super::eval::eval;
        let node = self.tree.get(id).unwrap();
        eval(symbols, node.first_child().unwrap()).map_err(|e| self.convert_error(e))
    }

    pub fn add_symbol<S>(
        &self,
        symbols: &mut SymbolTable,
        value: i64,
        name: S,
        id: AstNodeId,
    ) -> Result<SymbolId, UserError>
    where
        S: Into<String>,
    {
        symbols
            .add_symbol_with_value(&name.into(), value, id)
            .map_err(|e| {
                let msg = format!("Symbol error {:?}", e);
                self.user_error(&msg, id)
            })
    }

    fn evaluate(&mut self) -> Result<(), UserError> {
        info("Evaluating assignments", |x| {
            use super::eval::eval;
            use Item::*;

            let mut symbols = self.symbols.clone();

            for n in self.tree.nodes() {
                if let Assignment(name) = &n.value().item {
                    let id = n.id();

                    let value = self.eval(&symbols, id)?;
                    self.add_symbol(&mut symbols, value, name, id)?;

                    let msg = format!("{} = {}", name.clone(), value);
                    x.debug(&msg);
                }
            }

            self.symbols = symbols;

            Ok(())
        })
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq)]
struct Term {
    node: AstNodeId,
    priority: Option<usize>,
}

impl postfix::GetPriotity for Term {
    fn priority(&self) -> Option<usize> {
        self.priority
    }
}

pub fn to_priority(i: &Item) -> Option<usize> {
    use Item::*;
    match i {
        Div => Some(5),
        Mul => Some(5),
        Add => Some(4),
        Sub => Some(4),
        And => Some(2),
        Or => Some(2),
        Xor => Some(2),
        _ => None,
    }
}

impl Term {
    pub fn new(node: &AstNodeRef) -> Self {
        Self {
            node: node.id(),
            priority: to_priority(&node.value().item),
        }
    }
}
