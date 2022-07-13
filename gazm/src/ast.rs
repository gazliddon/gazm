pub type AstTree = ego_tree::Tree<ItemWithPos>;
pub type AstNodeRef<'a> = ego_tree::NodeRef<'a, ItemWithPos>;
pub type AstNodeId = ego_tree::NodeId;
pub type AstNodeMut<'a> = ego_tree::NodeMut<'a, ItemWithPos>;

// use std::fmt::{Debug, DebugMap};

use std::slice::SliceIndex;
use std::vec;

use crate::error::{AstError, UserError};
use crate::eval::{EvalError, EvalErrorEnum};
use crate::scopes::ScopeBuilder;

use crate::ctx::Context;
use crate::item::{Item, Node};

use crate::postfix;
use crate::{messages::*, node};
use emu::utils::sources::{Position, SourceInfo, SymbolQuery, SymbolWriter};

////////////////////////////////////////////////////////////////////////////////

fn get_kids_ids(tree: &AstTree, id: AstNodeId) -> Vec<AstNodeId> {
    tree.get(id).unwrap().children().map(|c| c.id()).collect()
}

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
    pub fn from_nodes(ctx: &'a mut Context, node: Node ) -> Result<AstTree, UserError> {
        let tree = make_tree(&node);
        let r =  Self::new(tree, ctx)?;
        Ok(r.tree)
    }

    pub fn new(tree: AstTree, ctx: &'a mut Context) -> Result<Self, UserError> {
        let mut ret = Self { tree, ctx };

        ret.rename_locals();

        ret.process_macros()?;

        ret.postfix_expressions()?;

        ret.generate_struct_symbols()?;

        ret.evaluate_assignments()?;

        Ok(ret)
    }

    pub fn process_macros(&mut self) -> Result<(), UserError> {
        // TODO should be written in a way that can detect
        // redefinitions of a macro
        use std::collections::HashMap;
        use Item::*;

        // Make a hash of macro definitions
        // longer term this needs scoping
        let mdefs = self
            .tree
            .nodes()
            .filter_map(|n| match &n.value().item {
                MacroDef(name, ..) => Some((name.to_string(), n.id())),
                _ => None,
            })
            .collect::<HashMap<_, _>>();

        // and a vec of macro calls
        let mcalls = self
            .tree
            .nodes()
            .enumerate()
            .filter_map(|(i, n)| {
                let val = &n.value();
                match &val.item {
                    MacroCall(name) => Some((i, name.to_string(), n.id(), val.pos.clone())),
                    _ => None,
                }
            })
            .collect::<Vec<_>>();

        let mut nodes_to_change: Vec<(AstNodeId, AstNodeId)> = vec![];

        for (caller_num, name, caller_id, pos) in mcalls.into_iter() {
            // TODO need a failure case if we can't find the macro definition
            let macro_id = mdefs.get(&name).ok_or_else(|| {
                let mess = format!("Can't find macro definition for {name}");
                self.user_error(mess, caller_id)
            })?;

            let caller_scope = format!("%MACRO%_{name}_{caller_num}");

            let item = MacroCallProcessed {
                macro_id: *macro_id,
                scope: caller_scope,
            };

            let mut new_node = self.tree.orphan(ItemWithPos { pos, item });
            new_node.reparent_from_id_append(caller_id);
            nodes_to_change.push((caller_id, new_node.id()));
        }

        for (from_id, to_id) in nodes_to_change {
            self.tree.get_mut(from_id).unwrap().insert_id_after(to_id);
        }

        Ok(())
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

    fn eval_node(&self, id: AstNodeId) -> Result<i64, UserError> {
        use super::eval::eval;
        let node = self.tree.get(id).unwrap();
        let item = &node.value().item;
        let err = |m| self.node_error(m, id, true);

        if let Item::PostFixExpr = item {
            eval(&self.ctx.symbols, node).map_err(|e| self.convert_error(e.into()))
        } else {
            Err(err(&format!(
                "Incorrect item type for evalulation : {item:?}"
            )))
        }
    }

    fn eval_node_child(&self, id: AstNodeId) -> Result<i64, UserError> {
        let err = |m| self.node_error(m, id, true);
        let node = self.tree.get(id).unwrap();
        let first_child = node
            .first_child()
            .ok_or_else(|| err("Can't find a child node"))?;

        self.eval_node(first_child.id())
    }

    pub fn add_symbol<S>(&mut self, value: i64, name: S, id: AstNodeId) -> Result<u64, UserError>
    where
        S: Into<String>,
    {
        self.ctx
            .symbols
            .add_symbol_with_value(&name.into(), value)
            .map_err(|e| {
                let msg = format!("Symbol error {:?}", e);
                self.user_error(&msg, id)
            })
    }

    fn generate_struct_symbols(&mut self) -> Result<(), UserError> {
        info("Generating symbols for struct definitions", |x| {
            use Item::*;
            let ids: Vec<_> = self.tree.nodes().map(|n| n.id()).collect();

            // let mut symbols = self.symbols.clone();
            for id in ids {
                let item = &self.tree.get(id).unwrap().value().item.clone();

                if let StructDef(name) = item {
                    let mut current = 0;
                    x.info(format!("Generating symbols for {name}"));

                    let kids_ids = get_kids_ids(&self.tree, id);

                    for c_id in kids_ids {
                        let i = &self.tree.get(c_id).unwrap().value().item;

                        if let StructEntry(entry_name) = i {
                            let value = self.eval_node_child(c_id)?;
                            let scoped_name = format!("{}.{}", name, entry_name);
                            self.add_symbol(current, &scoped_name, c_id)?;
                            x.info(format!("Struct: Set {scoped_name} to {current}"));
                            current += value;
                        }
                    }

                    let scoped_name = format!("{name}.size");
                    self.add_symbol(current, &scoped_name, id)?;
                }
            }

            Ok(())
        })
    }

    fn evaluate_assignments(&mut self) -> Result<(), UserError> {
        status("Evaluating assignments", |x| {
            use super::eval::eval;
            use Item::*;

            let mut pc_references = vec![];

            let ids: Vec<_> = self.tree.nodes().map(|n| n.id()).collect();

            for id in ids {
                let item = self.tree.get(id).unwrap().value().item.clone();

                match &item {
                    Scope(scope) => {
                        self.ctx.symbols.set_root();
                        if scope != "root" {
                            self.ctx.symbols.set_scope(scope);
                        }
                    }

                    Assignment(name) => {

                        let n = self.tree.get(id).unwrap();
                        let cn = n.first_child().unwrap();
                        let res = eval(&self.ctx.symbols, cn);
                        let c_id = cn.id();

                        match res {
                            Ok(value) => {
                                self.add_symbol(value, name, c_id)?;
                                let si = self.get_source_info_from_node_id(id).unwrap();
                                let scope = self.ctx.symbols.get_current_scope_fqn();
                                let msg = format!("{scope}::{} = {} :  {} {}", name.clone(), value, si.file.to_string_lossy(),si.line_str);
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
                                let msg = format!("Evaluating assignments: {}", ast_err);
                                return Err(self.node_error(&msg, c_id, true));
                            }
                        }
                    }
                    _ => (),
                }
            }

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
