pub type AstTree = ego_tree::Tree<ItemWithPos>;
pub type AstNodeRef<'a> = ego_tree::NodeRef<'a, ItemWithPos>;
pub type AstNodeId = ego_tree::NodeId;
pub type AstNodeMut<'a> = ego_tree::NodeMut<'a, ItemWithPos>;

use crate::error::{AstError, UserError};
use crate::eval::{EvalError, EvalErrorEnum};
use crate::gazm::ScopeTracker;
use crate::scopes::ScopeBuilder;

use crate::ctx::Context;
use crate::item::{Item, LabelDefinition, Node};

use crate::messages::*;
use emu::utils::eval::{to_postfix, GetPriority};
use emu::utils::sources::{
    AsmSource, Position, SourceErrorType, SourceInfo, SymbolQuery, SymbolScopeId, SymbolTree,
    SymbolWriter,
};
use tower_lsp::lsp_types::request::WillRenameFiles;

use crate::info_mess;

use std::iter;

////////////////////////////////////////////////////////////////////////////////
fn get_kids_ids(tree: &AstTree, id: AstNodeId) -> Vec<AstNodeId> {
    tree.get(id).unwrap().children().map(|c| c.id()).collect()
}

fn is_problem(p: &Position) -> bool {
    p.src == AsmSource::FileId(4) && p.line == 19 && p.col == 12
}

#[derive(Debug, PartialEq, Clone)]
pub struct ItemWithPos {
    pub item: Item,
    pub pos: Position,
}

impl ItemWithPos {
    pub fn new(n: &Node) -> Self {
        Self {
            item: n.item.clone(),
            pos: n.ctx.clone(),
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
        get_recursive(n, f);
    }
}
fn get_recursive_track_scope<F>(node: AstNodeRef, f: &mut F, scope_id: u64)
where
    F: FnMut(AstNodeRef, u64),
{
    if let Item::Scope(name) = &node.value().item {
        panic!("WHUT? {name}")
    }

    let new_scope_id = if let Item::ScopeId(new_scope_id) = node.value().item {
        new_scope_id
    } else {
        f(node, scope_id);
        scope_id
    };

    for n in node.children() {
        get_recursive_track_scope(n, f, new_scope_id);
    }
}

/// Return a vec of depth first node ids
fn get_ids_recursive(node: AstNodeRef) -> Vec<AstNodeId> {
    let mut ret = vec![];
    get_recursive(node, &mut |x: AstNodeRef| ret.push(x.id()));
    ret
}
/// Return a vec of depth first node ids
fn get_ids_recursive_track_scope(node: AstNodeRef, current_scope_id: u64) -> Vec<(AstNodeId, u64)> {
    let mut ret = vec![];
    get_recursive_track_scope(
        node,
        &mut |x: AstNodeRef, scope_id: u64| ret.push((x.id(), scope_id)),
        current_scope_id,
    );
    ret
}

/// Depth first iteration of all node ids
pub fn iter_ids_recursive(node: AstNodeRef) -> impl Iterator<Item = AstNodeId> {
    let mut i = get_ids_recursive(node).into_iter();
    iter::from_fn(move || i.next())
}
/// Depth first iteration of all node ids tracking scope
pub fn iter_ids_recursive_track_scope(
    node: AstNodeRef,
    scope_id: u64,
) -> impl Iterator<Item = (AstNodeId, u64)> {
    let mut i = get_ids_recursive_track_scope(node, scope_id).into_iter();
    iter::from_fn(move || i.next())
}

pub fn iter_refs_recursive(node: AstNodeRef) -> impl Iterator<Item = AstNodeRef> {
    let mut i = get_ids_recursive(node).into_iter();
    iter::from_fn(move || i.next().and_then(|id| node.tree().get(id)))
}
pub fn iter_refs_recursive_track_scope(
    node: AstNodeRef,
    scope_id: u64,
) -> impl Iterator<Item = (AstNodeRef, u64)> {
    let mut i = get_ids_recursive_track_scope(node, scope_id).into_iter();
    iter::from_fn(move || {
        i.next()
            .and_then(|(id, scope_id)| node.tree().get(id).map(|n| (n, scope_id)))
    })
}

fn iter_items_recursive(node: AstNodeRef) -> impl Iterator<Item = (AstNodeId, &Item)> {
    let mut i = get_ids_recursive(node).into_iter();
    iter::from_fn(move || {
        i.next()
            .and_then(|id| node.tree().get(id).map(|n| (n.id(), &n.value().item)))
    })
}
#[allow(dead_code)]
fn iter_values_recursive(node: AstNodeRef) -> impl Iterator<Item = (AstNodeId, &ItemWithPos)> {
    let mut i = get_ids_recursive(node).into_iter();
    iter::from_fn(move || {
        i.next()
            .and_then(|id| node.tree().get(id).map(|n| (n.id(), n.value())))
    })
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
        self.replace_node(old_node_id, new_node_id);
    }

    fn base(tree: AstTree, ctx: &'a mut Context) -> Self {
        Self {
            tree,
            ctx,
            macro_defs: vec![],
        }
    }

    fn process(&mut self) -> Result<(), UserError> {
        self.create_scopes()?;
        self.inline_includes()?;
        self.postfix_expressions()?;
        self.rename_locals();
        self.process_macros_definitions()?;
        self.generate_struct_symbols()?;
        self.scope_assignments()?;
        self.scope_labels()?;
        self.evaluate_assignments()?;
        Ok(())
    }

    pub fn new(tree: AstTree, ctx: &'a mut Context) -> Result<Self, UserError> {
        let mut ret = Self::base(tree, ctx);
        ret.process()?;
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
        info("Inlining include files", |_| {
            // Loop over the ast until we have replaced all of the includes
            // each include can have includes in it as well
            loop {
                // Get all of the include ids
                let include_ids: Vec<_> = iter_items_recursive(self.tree.root())
                    .filter_map(|(id, item)| {
                        item.unwrap_include()
                            .map(|p| (id, self.ctx.get_full_path(p).unwrap()))
                    })
                    .collect();

                if include_ids.is_empty() {
                    info_mess!("Finished inlining includes");
                    break;
                }

                // Go through the ids and get the tokens to insert into this AST
                for (id, actual_file) in include_ids {
                    if let Some(tokens) = self.ctx.get_tokens_from_full_path(&actual_file) {
                        info_mess!("Inlining {} - HAD TOKENS", actual_file.to_string_lossy());
                        let new_node_id = create_ast_node(&mut self.tree, &tokens.node);
                        self.replace_node(id, new_node_id);
                    } else {
                        let x: Vec<_> = self
                            .ctx
                            .token_store
                            .tokens
                            .keys()
                            .map(|k| k.to_string_lossy())
                            .collect();

                        panic!(
                            "Can't find include tokens for {}\n{:?}",
                            actual_file.to_string_lossy(),
                            x
                        )
                    }
                }
            }

            Ok(())
        })
    }

    // pub fn detach_nodes_by_id<I: Iterator<Item = AstNodeId>>(&mut self, i: I) {
    //     for id in i {
    //         self.tree.get_mut(id).unwrap().detach();
    //     }
    // }

    pub fn detach_nodes_filter<I, F>(&mut self, i: I, f: F) -> Vec<AstNodeId>
    where
        I: Iterator<Item = AstNodeId>,
        F: Fn(AstNodeRef) -> bool,
    {
        i.filter(|id| {
            let node = self.tree.get(*id).unwrap();
            if f(node) {
                self.tree.get_mut(*id).unwrap().detach();
                true
            } else {
                false
            }
        })
        .collect()
    }

    pub fn process_macros_definitions(&mut self) -> Result<(), UserError> {
        info("Processing macro definitions", |_| {
            // TODO: should be written in a way that can detect
            // redefinitions of a macro
            use std::collections::HashMap;
            use Item::{MacroCall, MacroCallProcessed, ScopeId};

            // Detach all of the MacroDef nodes
            let detached = self.detach_nodes_filter(iter_ids_recursive(self.tree.root()), |nref| {
                matches!(nref.value().item, Item::MacroDef(..))
            });

            let mdefs: HashMap<String, (AstNodeId, Vec<String>)> = detached
                .into_iter()
                .filter_map(|id| {
                    let item = &self.tree.get(id).unwrap().value().item;
                    item.unwrap_macro_def()
                        .map(|(nm, params)| (nm.into(), (id, params.to_vec())))
                })
                .collect();

            let mut mcalls = vec![];

            let root_scope_id = self.symbols().get_root_id();

            let mut scopes = ScopeTracker::new(root_scope_id);

            for (i, macro_call_node) in iter_refs_recursive(self.tree.root()).enumerate() {
                let syms = &mut self.ctx.asm_out.symbols;
                let val = &macro_call_node.value();
                match &val.item {
                    ScopeId(scope_id) => scopes.set_scope(*scope_id),

                    MacroCall(name) => {
                        // Create a unique name for this macro application scope
                        let caller_scope_name = format!("%MACRO%_{name}_{i}");

                        // Create the scope
                        let macro_caller_scope_id =
                            syms.create_or_get_scope_for_parent(&caller_scope_name, scopes.scope());

                        let (macro_id, params) = mdefs.get(name).ok_or_else(|| {
                            self.node_error("Can't find macro", macro_call_node.id(), false)
                        })?;

                        mcalls.push((
                            macro_id,
                            macro_caller_scope_id,
                            params,
                            macro_call_node.id(),
                            val.pos.clone(),
                        ))
                    }
                    _ => (),
                }
            }

            // Create new nodes to replace the current macro call nodes
            // let mut nodes_to_change: Vec<(AstNodeId, AstNodeId)> = vec![];
            for (macro_id, caller_scope_id, params, caller_node_id, pos) in mcalls.into_iter() {
                let syms = &mut self.ctx.asm_out.symbols;
                let params_vec = syms.add_symbols_to_scope(caller_scope_id, params).unwrap();

                // Create the node we'll replace
                let replacement_id = self
                    .tree
                    .orphan(ItemWithPos {
                        item: MacroCallProcessed {
                            macro_id: *macro_id,
                            scope_id: caller_scope_id,
                            params_vec_of_id: params_vec.into(),
                        },
                        pos,
                    })
                    .id();

                self.replace_node_take_children(caller_node_id, replacement_id);
            }

            Ok(())
        })
    }

    fn get_source_info_from_node_id(&self, id: AstNodeId) -> Result<SourceInfo, SourceErrorType> {
        let n = self.tree.get(id).unwrap();
        self.ctx.sources().get_source_info(&n.value().pos)
    }

    fn rename_locals(&mut self) {
        info("Scoping locals into globals", |x| {
            let mut scopes = ScopeBuilder::new();

            let rename = |fqn: &String, name: &String| {
                let ret = format!("{fqn}/{name}");
                x.debug(format!("{name} -> {ret}"));
                ret
            };

            // Expand all local labels to have a scoped name
            // and change all locals to globals
            for v in self.tree.values_mut() {
                use Item::*;
                match &v.item {
                    AssignmentFromPc(LabelDefinition::Text(name)) => {
                        scopes.pop();
                        scopes.push_new(name);
                    }

                    LocalAssignmentFromPc(LabelDefinition::Text(name)) => {
                        let new_name = rename(&scopes.get_current_fqn(), name);
                        v.item = AssignmentFromPc(new_name.into());
                    }

                    LocalAssignment(LabelDefinition::Text(name)) => {
                        let new_name = rename(&scopes.get_current_fqn(), name);
                        v.item = Assignment(new_name.into());
                    }

                    LocalLabel(LabelDefinition::Text(name)) => {
                        let new_name = rename(&scopes.get_current_fqn(), name);
                        v.item = Label(new_name.into());
                    }

                    TokenizedFile(_, _) => {
                        scopes.pop();
                    }

                    _ => (),
                };
            }
        });
    }

    // Convert this node to from infix to postfix
    fn node_to_postfix(&mut self, node_id: AstNodeId) -> Result<(), UserError> {
        let node = self.tree.get(node_id).expect("Can't fetch node id");
        assert!(node.value().item.is_expr());

        let args = node.children().map(Term::from).collect::<Vec<_>>();

        let ret = to_postfix(&args).map_err(|s| {
            let args: Vec<_> = args
                .iter()
                .map(|a| format!("{:?}", self.tree.get(a.node).unwrap().value().item))
                .collect();
            let msg = format!("\n{:?} {:?}\n {}", s, node.value(), args.join("\n"));
            let msg = format!("Can't convert to postfix: {msg}");
            self.node_error(msg, node_id, true)
        })?;

        // Remove and reappend each node
        let tree = &mut self.tree;

        for t in ret.iter().map(|t| t.node) {
            tree.get_mut(t).expect("Couldn't Fetch mut child").detach();
            tree.get_mut(node_id).unwrap().append_id(t);
        }

        tree.get_mut(node_id).unwrap().value().item = Item::PostFixExpr;
        Ok(())
    }

    fn postfix_expressions(&mut self) -> Result<(), UserError> {
        info("Converting expressions to poxtfix", |_| {
            for id in iter_ids_recursive(self.tree.root()) {
                if self.tree.get(id).unwrap().value().item.is_expr() {
                    self.node_to_postfix(id)?
                }
            }
            Ok(())
        })
    }

    fn convert_error(&self, e: AstError) -> UserError {
        let si = self.get_source_info_from_node_id(e.node_id).unwrap();
        UserError::from_ast_error(e, &si)
    }

    fn node_error<S>(&self, msg: S, id: AstNodeId, is_failure: bool) -> UserError
    where
        S: Into<String>,
    {
        let node = self.tree.get(id).unwrap();
        let si = &self.get_source_info_from_node_id(node.id()).unwrap();
        UserError::from_text(msg, si, is_failure)
    }

    fn eval_node(&self, id: AstNodeId, current_scope_id: u64) -> Result<i64, UserError> {
        use super::eval::eval;
        let node = self.tree.get(id).unwrap();
        let item = &node.value().item;
        let err = |m| self.node_error(m, id, true);

        if let Item::PostFixExpr = item {
            let reader = self.ctx.asm_out.symbols.get_symbol_reader(current_scope_id);
            eval(&reader, node).map_err(|e| self.convert_error(e.into()))
        } else {
            Err(err(&format!(
                "Incorrect item type for evalulation : {item:?}"
            )))
        }
    }

    fn eval_node_child(&self, id: AstNodeId, current_scope_id: u64) -> Result<i64, UserError> {
        let err = |m| self.node_error(m, id, true);
        let node = self.tree.get(id).unwrap();
        let first_child = node
            .first_child()
            .ok_or_else(|| err("Can't find a child node"))?;

        self.eval_node(first_child.id(), current_scope_id)
    }

    pub fn add_symbol<S>(
        &mut self,
        value: i64,
        name: S,
        id: AstNodeId,
        current_scope_id: u64,
    ) -> Result<SymbolScopeId, UserError>
    where
        S: Into<String>,
    {
        let mut writer = self.ctx.asm_out.symbols.get_symbol_nav(current_scope_id);
        writer
            .add_symbol_with_value(&name.into(), value)
            .map_err(|e| {
                let msg = format!("Symbol error {e:?}");
                self.node_error(msg, id, false)
            })
    }

    fn generate_struct_symbols(&mut self) -> Result<(), UserError> {
        let current_scope_id = self.ctx.asm_out.symbols.get_root_id();

        info("Generating symbols for struct definitions", |x| {
            use Item::*;

            // self.ctx.asm_out.symbols.set_root();

            // let mut symbols = self.symbols.clone();
            for id in iter_ids_recursive(self.tree.root()) {
                let item = &self.tree.get(id).unwrap().value().item.clone();

                if let StructDef(name) = item {
                    let mut current = 0;
                    x.debug(format!("Generating symbols for {name}"));

                    let kids_ids = get_kids_ids(&self.tree, id);

                    for c_id in kids_ids {
                        let i = &self.tree.get(c_id).unwrap().value().item;

                        if let StructEntry(entry_name) = i {
                            let value = self.eval_node_child(c_id, current_scope_id)?;
                            let scoped_name = format!("{name}.{entry_name}");
                            self.add_symbol(current, &scoped_name, c_id, current_scope_id)?;
                            x.debug(format!("Struct: Set {scoped_name} to {current}"));
                            current += value;
                        }
                    }

                    let scoped_name = format!("{name}.size");
                    self.add_symbol(current, &scoped_name, id, current_scope_id)?;
                }
            }

            Ok(())
        })
    }

    /// Traverse all nodes and create scopes from Scope(name)
    /// and change node from Scope(name) -> ScopeId(scope_id)
    fn create_scopes(&mut self) -> Result<(), UserError> {
        use Item::*;

        let root_id = self.ctx.asm_out.symbols.get_root_id();

        for node_id in iter_ids_recursive(self.tree.root()) {
            let item = self.tree.get(node_id).unwrap().value().item.clone();

            if let Scope(scope) = &item {
                let mut writer = self.ctx.asm_out.symbols.get_symbol_nav(root_id);
                let id = writer.set_current_scope(scope.as_str());
                let mut x = self.tree.get_mut(node_id).unwrap();
                x.value().item = ScopeId(id);
            }
        }

        Ok(())
    }

    fn symbols(&self) -> &SymbolTree {
        self.ctx.get_symbols()
    }

    fn symbols_mut(&mut self) -> &mut SymbolTree {
        self.ctx.get_symbols_mut()
    }

    fn scope_labels(&mut self) -> Result<(), UserError> {
        use Item::*;
        let root_node = self.tree.root();
        let root_scope_id = self.symbols().get_root_id();

        let mut current_scope_id = root_scope_id;

        for node_id in iter_ids_recursive(root_node) {
            let value = self.tree.get(node_id).unwrap().value();
            let item = value.item.clone();
            let pos = value.pos.clone();

            match &item {
                Scope(name) => {
                    panic!("Should not happen {name}")
                }

                ScopeId(scope_id) => current_scope_id = *scope_id,

                // Convert any label in tree to a lable reference
                Label(LabelDefinition::Text(name)) => {
                    let si = self.ctx.sources().get_source_info(&pos).unwrap();

                    let err = if is_problem(&pos) {
                        format!(
                            "Current scope {} name {name} {si:#?} cs_id:{current_scope_id}",
                            self.ctx.asm_out.symbols.get_fqn_from_id(current_scope_id)
                        )
                    } else {
                        "not a problen?".to_owned()
                    };

                    let symbols = &mut self.ctx.asm_out.symbols;
                    let reader = symbols.get_symbol_reader(current_scope_id);
                    let id = reader.get_symbol_info(name).expect(&err).symbol_id;
                    let mut x = self.tree.get_mut(node_id).unwrap();
                    x.value().item = Label(id.into());
                }

                _ => (),
            }
        }

        Ok(())
    }

    /// Traverse through all assignments and reserve a label for them at the correct scope
    fn scope_assignments(&mut self) -> Result<(), UserError> {
        info("Correctly scoping assignments", |_| {
            use Item::*;
            let root_id = self.ctx.asm_out.symbols.get_root_id();
            let mut current_scope = root_id;

            for node_id in iter_ids_recursive(self.tree.root()) {
                let value = self.tree.get(node_id).unwrap().value();
                let item = &value.item;

                match item {
                    Scope(name) => panic!("Whut? {name}"),

                    ScopeId(scope_id) => current_scope = *scope_id,

                    AssignmentFromPc(LabelDefinition::Text(name)) => {
                        let name = name.clone();

                        let mut writer = self.ctx.get_symbols_mut().get_symbol_nav(current_scope);
                        let sym_id = writer.add_symbol(&name).unwrap();
                        let mut this_node_mut = self.tree.get_mut(node_id).unwrap();
                        this_node_mut.value().item = AssignmentFromPc(sym_id.into());
                    }

                    Assignment(LabelDefinition::Text(name)) => {
                        let symbols = &mut self.ctx.asm_out.symbols;
                        let mut writer = symbols.get_symbol_nav(current_scope);
                        let sym_id = writer.add_symbol(name).unwrap();
                        let mut x = self.tree.get_mut(node_id).unwrap();
                        x.value().item = Assignment(sym_id.into());
                    }
                    _ => (),
                }
            }

            Ok(())
        })
    }

    fn evaluate_assignments(&mut self) -> Result<(), UserError> {
        info("Evaluating assignments", |_| {
            use super::eval::eval;
            use Item::*;
            let mut current_scope_id = self.ctx.get_symbols().get_root_id();

            let symbols = &mut self.ctx.asm_out.symbols;

            for id in iter_ids_recursive(self.tree.root()) {
                match &self.tree.get(id).unwrap().value().item {
                    ScopeId(scope_id) => {
                        current_scope_id = *scope_id;
                    }

                    Assignment(LabelDefinition::Scoped(label_id)) => {
                        let label_id = *label_id;
                        let tree = &mut self.tree;
                        let expr = tree.get(id).unwrap().first_child().unwrap();
                        let reader = symbols.get_symbol_reader(current_scope_id);

                        match eval(&reader, expr) {
                            Ok(value) => {
                                symbols
                                    .set_symbol_from_id(label_id, value)
                                    .expect("Can't set symbols");
                            }

                            Err(EvalError {
                                source: EvalErrorEnum::CotainsPcReference,
                                ..
                            }) => {
                                let assignment = Item::AssignmentFromPc(label_id.into());
                                tree.get_mut(id)
                                    .expect("Can't get assignment node")
                                    .value()
                                    .item = assignment;
                            }

                            Err(e) => {
                                return Err(self.convert_error(e.into()));
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
    #[allow(dead_code)]
    pub fn new(node: &AstNodeRef) -> Self {
        Self {
            node: node.id(),
            priority: to_priority(&node.value().item),
        }
    }
}

impl From<AstNodeRef<'_>> for Term {
    fn from(node: AstNodeRef) -> Self {
        Self {
            node: node.id(),
            priority: to_priority(&node.value().item),
        }
    }
}
