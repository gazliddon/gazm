pub type AstTree = ego_tree::Tree<ItemWithPos>;
pub type AstNodeRef<'a> = ego_tree::NodeRef<'a, ItemWithPos>;
pub type AstNodeId = ego_tree::NodeId;
pub type AstNodeMut<'a> = ego_tree::NodeMut<'a, ItemWithPos>;
// use std::fmt::{Debug, DebugMap};
//
// lkd dk dlk
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::{default, iter, vec};

use crate::error::{AstError, UserError};
use crate::eval::{EvalError, EvalErrorEnum};
use crate::parse::util::match_escaped_str;
use crate::scopes::ScopeBuilder;

use crate::ctx::Context;
use crate::item::{Item, LabelDefinition, Node};

use crate::{messages::*, node};
use emu::utils::eval::{to_postfix, GetPriority, PostFixer};
use emu::utils::sources::{
    Position, SourceErrorType, SourceInfo, SymbolQuery, SymbolScopeId, SymbolTree, SymbolWriter,
};
use emu::utils::symbols;

////////////////////////////////////////////////////////////////////////////////
fn get_kids_ids(tree: &AstTree, id: AstNodeId) -> Vec<AstNodeId> {
    tree.get(id).unwrap().children().map(|c| c.id()).collect()
}

// fn replace_node(tree: &mut AstTree, old_node_id: AstNodeId, new_node_id: AstNodeId) {
//     let mut old_node = tree.get_mut(old_node_id).expect("can't retrieve old node");
//     old_node.insert_id_after(new_node_id);
//     old_node.detach();
// }

// fn replace_node_take_children(tree: &mut AstTree, old_node_id: AstNodeId, new_node_id: AstNodeId) {
//     let mut replacement_node = tree
//         .get_mut(new_node_id)
//         .expect("Can't fetch replacement node");
//     replacement_node.reparent_from_id_append(old_node_id);
//     replace_node(tree, old_node_id, new_node_id)
// }

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

pub fn create_ast_node(tree: &mut AstTree, node: &Node) -> AstNodeId {
    let ipos = ItemWithPos::new(node);

    let mut this_node = tree.orphan(ipos);

    for n in &node.children {
        add_node(&mut this_node, n);
    }
    this_node.id()
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
    pub macro_defs: Vec<AstNodeId>,
    pub ctx: &'a mut Context,
}

/// Iterate through the nodes recursively, depth first
fn get_recursive<F>(node: AstNodeRef, f: &mut F)
where
    F: FnMut(AstNodeRef),
{
    f(node);

    for n in node.children() {
        get_recursive(n, f)
    }
}

fn get_ids_recursive(node: AstNodeRef) -> Vec<AstNodeId> {
    let mut ret = vec![];
    get_recursive(node, &mut |x: AstNodeRef| ret.push(x.id()));
    ret
}

fn iter_ids_recursive(node: AstNodeRef) -> impl Iterator<Item = AstNodeId> {
    let mut i = get_ids_recursive(node).into_iter();

    iter::from_fn(move || i.next())
}

fn iter_refs_recursive(node: AstNodeRef) -> impl Iterator<Item = AstNodeRef> {
    let mut i = get_ids_recursive(node).into_iter();
    iter::from_fn(move || i.next().and_then(|id| node.tree().get(id)))
}

impl<'a> Ast<'a> {
    fn replace_node(&mut self, old_node_id: AstNodeId, new_node_id: AstNodeId) {
        let mut old_node = self
            .tree
            .get_mut(old_node_id)
            .expect("can't retrieve old node");
        old_node.insert_id_after(new_node_id);
        old_node.detach();
    }

    fn replace_node_take_children(&mut self, old_node_id: AstNodeId, new_node_id: AstNodeId) {
        let mut replacement_node = self
            .tree
            .get_mut(new_node_id)
            .expect("Can't fetch replacement node");
        replacement_node.reparent_from_id_append(old_node_id);
        self.replace_node(old_node_id, new_node_id)
    }

    pub fn new(tree: AstTree, ctx: &'a mut Context) -> Result<Self, UserError> {
        let mut ret = Self {
            tree,
            ctx,
            macro_defs: vec![],
        };

        ret.inline_includes()?;
        ret.rename_locals();
        ret.postfix_expressions()?;
        ret.process_macros()?;
        ret.generate_struct_symbols()?;
        ret.scope_assignments()?;
        ret.scope_labels()?;
        ret.evaluate_assignments()?;

        Ok(ret)
    }

    pub fn from_nodes(ctx: &'a mut Context, node: &Node) -> Result<AstTree, UserError> {
        let tree = make_tree(node);
        let r = Self::new(tree, ctx)?;
        Ok(r.tree)
    }

    /// Find all of the includes in this AST and replace with the
    /// with inlines included files tokens
    fn inline_includes(&mut self) -> Result<(), UserError> {
        use Item::*;

        // Loop over the ast until we have replaced all of the includes
        // each include can have includes in it as well
        loop {
            // Get all of the include ids
            let include_ids: Vec<_> = iter_refs_recursive(self.tree.root())
                .filter_map(|node| node.value().item.get_include().map(|p| (node.id(), p)))
                .collect();

            // Break if there's no includes
            if include_ids.is_empty() {
                break;
            }

            // Go through the ids and get the tokens to insert into this AST
            for (id, path) in include_ids.iter() {
                let actual_file = self.ctx.get_full_path(&path).unwrap();
                let tokens = self.ctx.get_tokens(&actual_file);

                if tokens.is_none() {
                    println!(
                        "Can't find include tokens for {}",
                        actual_file.to_string_lossy()
                    );
                    panic!()
                }

                let tokens = tokens.unwrap();

                let new_node_id = create_ast_node(&mut self.tree, tokens);
                self.replace_node(*id, new_node_id);
            }
        }
        Ok(())
    }
    pub fn process_macros(&mut self) -> Result<(), UserError> {
        // TODO: should be written in a way that can detect
        // redefinitions of a macro
        use std::collections::HashMap;
        use Item::*;

        // Make a hash of macro definitions
        // longer term this needs scoping
        // and detach all of them from the main tree

        let mut mdefs: HashMap<String, (AstNodeId, Vec<String>)> = Default::default();

        for macro_id in iter_ids_recursive(self.tree.root()) {
            let macro_node = self.tree.get(macro_id).unwrap();
            match &macro_node.value().item {
                MacroDef(name, params) => {
                    mdefs.insert(name.to_string(), (macro_node.id(), params.clone()));
                    self.tree.get_mut(macro_id).expect("Can't get macro mut node").detach();
                }
                _ => (),
            }
        }

        self.ctx.asm_out.symbols.set_root();

        // and a vec of macro calls
        let mcalls = iter_refs_recursive(self.tree.root())
            .enumerate()
            .filter_map(|(i, macro_call_node)| {
                let syms = &mut self.ctx.asm_out.symbols;
                let val = &macro_call_node.value();
                match &val.item {
                    Scope(name) => {
                        syms.set_root();
                        syms.set_scope(name);
                        None
                    }

                    MacroCall(name) => {
                        // Create a unique name for this macro application scope
                        let caller_scope_name = format!("%MACRO%_{name}_{i}");
                        // Create the scope
                        let macro_caller_scope_id = syms.set_scope(&caller_scope_name);

                        // TODO: need a failure case if we can't find the macro definition
                        let (macro_id, params) =
                            mdefs.get(name).expect("Expected to find macro definition");

                        syms.pop_scope();

                        Some((
                            macro_id,
                            macro_caller_scope_id,
                            params,
                            macro_call_node.id(),
                            val.pos.clone(),
                        ))
                    }
                    _ => None,
                }
            })
            .collect::<Vec<_>>();

        // Create new nodes to replace the current macro call nodes
        // let mut nodes_to_change: Vec<(AstNodeId, AstNodeId)> = vec![];
        for (macro_id, macro_caller_scope_id, params, caller_node_id, pos) in mcalls.into_iter() {
            let params_vec = self.ctx.asm_out.symbols.add_symbols_to_scope(macro_caller_scope_id, &params).unwrap();

            // Create the node we'll replace
            let replacement_node_id = self
                .tree
                .orphan(ItemWithPos {
                    item: MacroCallProcessed {
                        macro_id: *macro_id,
                        scope_id: macro_caller_scope_id,
                        params_vec_of_id: params_vec,
                    },
                    pos,
                })
                .id();

            self.replace_node_take_children(caller_node_id, replacement_node_id);
        }

        Ok(())
    }

    fn get_source_info_from_node_id(&self, id: AstNodeId) -> Result<SourceInfo, SourceErrorType> {
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
            use crate::item::LabelDefinition;

            // Expand all local labels to have a scoped name
            // and change all locals to globals
            for v in self.tree.values_mut() {
                match &v.item {
                    AssignmentFromPc(LabelDefinition::Text(name)) => {
                        scopes.pop();
                        scopes.push_new(name);
                    }

                    LocalAssignmentFromPc(LabelDefinition::Text(name)) => {
                        let new_name = rename(&scopes.get_current_fqn(), name);
                        v.item = AssignmentFromPc(LabelDefinition::Text(new_name));
                    }

                    LocalAssignment(LabelDefinition::Text(name)) => {
                        let new_name = rename(&scopes.get_current_fqn(), name);
                        v.item = Assignment(LabelDefinition::Text(new_name));
                    }

                    LocalLabel(LabelDefinition::Text(name)) => {
                        let new_name = rename(&scopes.get_current_fqn(), name);
                        v.item = Label(LabelDefinition::Text(new_name));
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
        let args: Vec<_> = node.children().map(|n| Term::new(&n)).collect();

        let ret = to_postfix(&args).map_err(|s| {
            let args: Vec<String> = args
                .iter()
                .map(|a| format!("{:?}", self.tree.get(a.node).unwrap().value().item))
                .collect();
            format!("\n{:?} {:?}\n {}", s, node.value(), args.join("\n"))
        })?;

        let ret = ret.iter().map(|t| t.node).collect();

        Ok(ret)
    }

    fn postfix_expressions(&mut self) -> Result<(), UserError> {
        let ret = info("Converting expressions to poxtfix", |x| {
            use Item::*;

            let mut to_convert: Vec<(AstNodeId, Vec<AstNodeId>)> = vec![];

            // find all of the nodes that need converting
            // create a table of the order of ids desired to make the expression
            // postix
            for n in iter_refs_recursive(self.tree.root()) {
                let v = n.value();
                let parent_id = n.id();
                match &v.item {
                    BracketedExpr | Expr => {
                        let new_order = self.node_to_postfix(n).map_err(|s| {
                            let si = self.get_source_info_from_node_id(n.id()).unwrap();
                            let msg = format!("Can't convert to postfix: {}", s);
                            UserError::from_text(msg, &si, true)
                        })?;

                        to_convert.push((parent_id, new_order));
                    }
                    _ => (),
                }
            }

            for (parent, new_children) in to_convert.iter() {
                for c in new_children.iter() {
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

                for c in new_children.iter() {
                    p.append_id(*c);
                }
                p.value().item = PostFixExpr;
            }

            x.debug(&format!("Converted {} expression(s)", to_convert.len()));

            Ok(())
        });

        ret
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
            eval(&self.ctx.asm_out.symbols, node).map_err(|e| self.convert_error(e.into()))
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

    pub fn set_symbol(
        &mut self,
        value: i64,
        symbol_id: SymbolScopeId,
        _node_id: AstNodeId,
    ) -> Result<(), UserError> {
        self.ctx
            .asm_out
            .symbols
            .set_symbol(symbol_id, value)
            .map_err(|_| panic!())
    }

    pub fn add_symbol<S>(
        &mut self,
        value: i64,
        name: S,
        id: AstNodeId,
    ) -> Result<SymbolScopeId, UserError>
    where
        S: Into<String>,
    {
        self.ctx
            .asm_out
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

            self.ctx.asm_out.symbols.set_root();

            // let mut symbols = self.symbols.clone();
            for id in iter_ids_recursive(self.tree.root()) {
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

    fn scope_labels(&mut self) -> Result<(), UserError> {
        use Item::*;

        self.ctx.asm_out.symbols.set_root();

        for node_id in iter_ids_recursive(self.tree.root()) {
            let item = self.tree.get(node_id).unwrap().value().item.clone();
            // let pos = self.tree.get(node_id).unwrap().value().pos.clone();

            match &item {
                // Track scope correctly
                Scope(scope) => {
                    let symbols = &mut self.ctx.asm_out.symbols;
                    symbols.set_root();
                    symbols.set_scope(scope.as_str());
                }

                // Convert any label in tree to a lable reference
                Label(LabelDefinition::Text(name)) => {
                    let symbols = &mut self.ctx.asm_out.symbols;
                    let (id, _si) = symbols
                        .get_symbol_info(&name)
                        .expect("Internal error getting symbol info");
                    let mut x = self.tree.get_mut(node_id).unwrap();
                    x.value().item = Label(LabelDefinition::Scoped(id));
                }

                _ => (),
            }
        }

        Ok(())
    }

    /// Traverse through all assignments and reserve a label for them at the correct scope
    fn scope_assignments(&mut self) -> Result<(), UserError> {
        let res = info("Correctly scoping assignments", |_| {
            use super::eval::eval;
            use Item::*;

            self.ctx.asm_out.symbols.set_root();

            for node_id in iter_ids_recursive(self.tree.root()) {
                let symbols = &mut self.ctx.asm_out.symbols;
                let item = self.tree.get(node_id).unwrap().value().item.clone();

                match &item {
                    Scope(scope) => {
                        symbols.set_root();
                        symbols.set_scope(scope.as_str());
                    }

                    AssignmentFromPc(LabelDefinition::Text(name)) => {
                        let sym_id = symbols.add_symbol(name).unwrap();
                        let mut x = self.tree.get_mut(node_id).unwrap();
                        x.value().item = AssignmentFromPc(LabelDefinition::Scoped(sym_id));
                    }

                    Assignment(LabelDefinition::Text(name)) => {
                        let sym_id = symbols.add_symbol(name).unwrap();
                        let mut x = self.tree.get_mut(node_id).unwrap();
                        x.value().item = Assignment(LabelDefinition::Scoped(sym_id));
                    }
                    _ => (),
                }
            }

            Ok(())
        });

        res
    }

    fn evaluate_assignments(&mut self) -> Result<(), UserError> {
        info("Evaluating assignments", |_| {
            use super::eval::eval;
            use Item::*;

            self.ctx.asm_out.symbols.set_root();

            for id in iter_ids_recursive(self.tree.root()) {
                let item = self.tree.get(id).unwrap().value().item.clone();

                match &item {
                    Scope(scope) => {
                        self.ctx.asm_out.symbols.set_root();
                        if scope != "root" {
                            self.ctx.asm_out.symbols.set_scope(scope);
                        }
                    }

                    Assignment(LabelDefinition::Scoped(label_id)) => {
                        let scoped_item = LabelDefinition::Scoped(*label_id);
                        let n = self.tree.get(id).unwrap();
                        let cn = n.first_child().unwrap();
                        let res = eval(&self.ctx.asm_out.symbols, cn);
                        let c_id = cn.id();

                        match res {
                            Ok(value) => {
                                self.set_symbol(value, *label_id, c_id)?;
                            }

                            Err(EvalError {
                                source: EvalErrorEnum::CotainsPcReference,
                                ..
                            }) => {
                                let mut x = self.tree.get_mut(n.id()).unwrap();
                                x.value().item = Item::AssignmentFromPc(scoped_item);
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

            Ok(())
        })
    }
}

////////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq)]
struct Term {
    node: AstNodeId,
    priority: Option<usize>,
}

impl GetPriority for Term {
    fn priority(&self) -> Option<usize> {
        self.priority
    }
}

pub fn to_priority(i: &Item) -> Option<usize> {
    use Item::*;
    match i {
        Div => Some(12),
        Mul => Some(12),
        Add => Some(11),
        Sub => Some(11),
        ShiftLeft => Some(10),
        ShiftRight => Some(10),
        BitAnd => Some(9),
        BitXor => Some(8),
        BitOr => Some(7),
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
