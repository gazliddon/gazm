pub type AstTree = ego_tree::Tree<ItemWithPos>;
pub type AstNodeRef<'a> = ego_tree::NodeRef<'a, ItemWithPos>;
pub type AstNodeId = ego_tree::NodeId;
pub type AstNodeMut<'a> = ego_tree::NodeMut<'a, ItemWithPos>;

// use std::fmt::{Debug, DebugMap};

use crate::error::{AstError, UserError};
use crate::eval::{EvalError, EvalErrorEnum};
use crate::scopes::ScopeBuilder;

use crate::ctx::Context;
use crate::item::{Item, Node};

use crate::messages::*;
use crate::postfix;
use utils::sources::{Position, SourceInfo, SymbolId, SymbolQuery, SymbolWriter};

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq, Clone)]
pub struct ItemWithPos {
    pub item: Item,
    pub pos: Position,
}

impl ItemWithPos {
    pub fn new(n: &Node) -> Self {
        Self {
            item: n.item().clone(),
            pos: n.ctx().clone(),
        }
    }
}

pub fn add_node(parent: &mut AstNodeMut, node: &Node) {
    let ipos = ItemWithPos::new(node);
    let mut this_node = parent.append(ipos);

    for n in &node.children {
        add_node(&mut this_node, n);
    }
}

pub fn make_tree(node: &Node) -> AstTree {
    let mut ret = AstTree::new(ItemWithPos::new(node));

    for c in &node.children {
        add_node(&mut ret.root_mut(), c);
    }
    ret
}

#[derive(Debug)]
pub struct Ast<'a> {
    pub tree: AstTree,
    pub ctx: &'a mut Context,
}

impl<'a> Ast<'a> {
    pub fn from_nodes(node: Node, ctx: &'a mut Context) -> Result<Self, UserError> {
        let tree = make_tree(&node);
        Self::new(tree, ctx)
    }

    pub fn new(tree: AstTree, ctx: &'a mut Context) -> Result<Self, UserError> {
        let mut ret = Self { tree, ctx };

        ret.rename_locals();

        ret.postfix_expressions()?;

        ret.generate_struct_symbols()?;

        ret.evaluate_assignments()?;

        Ok(ret)
    }

    pub fn get_tree(&self) -> &AstTree {
        &self.tree
    }
    pub fn get_tree_mut(&mut self) -> &mut AstTree {
        &mut self.tree
    }

    fn get_source_info_from_node_id(&self, id: AstNodeId) -> Result<SourceInfo, String> {
        let n = self.tree.get(id).unwrap();
        self.ctx.sources().get_source_info(&n.value().pos)
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

                    TokenizedFile(_, _) => {
                        scopes.pop();
                    }

                    _ => (),
                };
            }
        });
    }

    fn node_to_postfix(&self, node: AstNodeRef) -> Result<Vec<AstNodeId>, String> {
        use postfix::PostFixer;

        let args: Vec<_> = node.children().map(|n| Term::new(&n)).collect();

        let mut pfix: PostFixer<Term> = postfix::PostFixer::new();
        let ret = pfix.get_postfix(args.clone()).map_err(|s| {
            let args: Vec<String> = args
                .iter()
                .map(|a| format!("{:?}", self.tree.get(a.node).unwrap().value().item))
                .collect();
            format!(
                "\n{:?}\n {}",
                self.tree.get(s.node).unwrap().value(),
                args.join("\n")
            )
        })?;

        let ret = ret.iter().map(|t| t.node).collect();

        Ok(ret)
    }

    // TODO!
    // Make this and other functions return an appropriate
    // error rather tha a string

    fn postfix_expressions(&mut self) -> Result<(), UserError> {
        info("Converting expressions to poxtfix", |x| {
            use Item::*;

            let mut to_convert: Vec<(AstNodeId, Vec<AstNodeId>)> = vec![];

            // find all of the nodes that need converting
            for n in self.tree.nodes() {
                let v = n.value();

                match v.item {
                    BracketedExpr | Expr => {
                        let new_order = self.node_to_postfix(n).map_err(|s| {
                            let si = self.get_source_info_from_node_id(n.id()).unwrap();
                            let msg = format!("Can't convert to postfix: {}", s);
                            UserError::from_text(msg, &si, true)
                        })?;

                        to_convert.push((n.id(), new_order));
                    }
                    _ => (),
                }
            }

            for (parent, new_children) in &to_convert {
                for c in new_children {
                    if let Some(mut c) = self.tree.get_mut(*c) {
                        c.detach();
                    } else {
                        let si = self.get_source_info_from_node_id(*c).unwrap();
                        return Err(UserError::from_text(
                            "Can't get a mutatable node",
                            &si,
                            true,
                        ));
                    }
                }

                let mut p = self.tree.get_mut(*parent).unwrap();

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

    fn node_error(&self, msg: &str, id: AstNodeId, is_failure: bool) -> UserError {
        let node = self.tree.get(id).unwrap();
        let si = &self.get_source_info_from_node_id(node.id()).unwrap();
        UserError::from_text(msg, si, is_failure)
    }

    fn eval_node_child(&self, symbols: &dyn SymbolQuery, id: AstNodeId) -> Result<i64, UserError> {
        use super::eval::eval;
        let node = self.tree.get(id).unwrap();

        let err = |m| self.node_error(m, id, true);

        let first_child = node
            .first_child()
            .ok_or_else(|| err("Can't find a child node"))?;

        let child_item = &first_child.value().item;

        if let Item::PostFixExpr = child_item {
            eval(symbols, first_child).map_err(|e| self.convert_error(e.into()))
        } else {
            Err(err(&format!(
                "Incorrect item type for evalulation : {child_item:?}"
            )))
        }
    }

    pub fn add_symbol<S>(
        &self,
        symbols: &mut dyn SymbolWriter,
        value: i64,
        name: S,
        id: AstNodeId,
    ) -> Result<SymbolId, UserError>
    where
        S: Into<String>,
    {
        symbols
            .add_symbol_with_value(&name.into(), value)
            .map_err(|e| {
                let msg = format!("Symbol error {:?}", e);
                self.user_error(&msg, id)
            })
    }

    fn generate_struct_symbols(&mut self) -> Result<(), UserError> {
        info("Generating symbols for struct definitions", |x| {
            use Item::*;

            let mut symbols = self.ctx.symbols.clone();

            // let mut symbols = self.symbols.clone();
            for n in self.tree.nodes() {
                let item = &n.value().item;

                if let StructDef(name) = item {
                    let mut current = 0;
                    x.info(format!("Generating symbols for {name}"));

                    for c in n.children() {
                        if let StructEntry(entry_name) = &c.value().item {
                            let id = c.id();
                            let value = self.eval_node_child(&self.ctx.symbols, id)?;
                            let scoped_name = format!("{}.{}", name, entry_name);
                            self.add_symbol(&mut symbols, current, &scoped_name, id)?;
                            x.info(format!("Struct: Set {scoped_name} to {current}"));
                            current += value;
                        }
                    }

                    let scoped_name = format!("{name}.size");

                    self.add_symbol(&mut symbols, current, &scoped_name, n.id())?;
                }
            }

            self.ctx.symbols = symbols;
            Ok(())
        })
    }

    fn evaluate_assignments(&mut self) -> Result<(), UserError> {
        status("Evaluating assignments", |x| {
            use super::eval::eval;
            use Item::*;

            let mut symbols = self.ctx.symbols.clone();

            let mut pc_references = vec![];

            for n in self.tree.nodes() {
                if let Assignment(name) = &n.value().item {
                    // let id = n.id();

                    let cnode = n.first_child().unwrap();

                    let res = eval(&symbols, cnode);

                    match res {
                        Ok(value) => {
                            self.add_symbol(&mut symbols, value, name, cnode.id())?;
                            let msg = format!("{} = {}", name.clone(), value);
                            x.debug(&msg);
                        }

                        Err(EvalError {
                            source: EvalErrorEnum::CotainsPcReference,
                            ..
                        }) => {
                            pc_references.push((name.clone(), n.id()));
                            let msg = format!("Marking to convert to pc reference: {}", name);
                            x.debug(&msg);
                        }

                        Err(e) => {
                            let ast_err: AstError = e.into();
                            return Err(self.node_error(&ast_err.to_string(), cnode.id(), true));
                        }
                    }
                }
            }

            self.ctx.symbols = symbols;

            for (name, id) in pc_references {
                let mut n = self.tree.get_mut(id).unwrap();
                n.value().item = Item::AssignmentFromPc(name);
            }

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
        ShiftLeft => Some(1),
        ShiftRight => Some(1),
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
