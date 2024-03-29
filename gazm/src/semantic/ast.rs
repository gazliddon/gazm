#![forbid(unused_imports)]
use grl_eval::{to_postfix, GetPriority};
use grl_sources::{Position, SourceErrorType, SourceInfo};
use std::{collections::HashMap, iter};
use thin_vec::{thin_vec, ThinVec};

use super::{EvalError, EvalErrorEnum};

use crate::{
    assembler::{Assembler, ScopeBuilder, ScopeTracker},
    astformat::as_string,
    debug_mess,
    error::{AstError, UserError},
    frontend::{AstNodeKind, LabelDefinition, Node},
    gazmsymbols::{ScopedName, SymbolError, SymbolScopeId, SymbolTreeReader, SymbolTreeWriter},
    interesting_mess,
    messages::*,
};

pub type AstTree = ego_tree::Tree<ItemWithPos>;
pub type AstNodeRef<'a> = ego_tree::NodeRef<'a, ItemWithPos>;
pub type AstNodeId = ego_tree::NodeId;
pub type AstNodeMut<'a> = ego_tree::NodeMut<'a, ItemWithPos>;

#[derive(Debug, PartialEq, Clone)]

pub struct ItemWithPos
{
    pub item: AstNodeKind,
    pub pos: Position,
}

impl ItemWithPos
{
    pub fn new(n: &Node) -> Self {
        Self {
            item: n.item.clone(),
            pos: n.ctx,
        }
    }
}

/// Ast
#[derive(Debug, Clone)]
pub struct Ast
{
    pub tree: AstTree,
}

impl std::fmt::Display for Ast
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let txt = as_string(self.tree.root());
        write!(f, "{txt}")
    }
}

impl Ast
{
    pub fn new(tree: AstTree) -> Self {
        Self { tree }
    }

    pub fn from_node(node: &Node) -> Self {
        let mut ret = AstTree::new(ItemWithPos::new(node));

        for c in &node.children {
            Self::add_node(&mut ret.root_mut(), c);
        }
        Ast { tree: ret }
    }

    fn add_node(parent: &mut AstNodeMut, node: &Node) {
        let ipos = ItemWithPos::new(node);
        let mut this_node = parent.append(ipos);

        for n in &node.children {
            Self::add_node(&mut this_node, n);
        }
    }

    pub fn get_kids_ids(&self, id: AstNodeId) -> ThinVec<AstNodeId> {
        self.as_ref()
            .get(id)
            .unwrap()
            .children()
            .map(|c| c.id())
            .collect()
    }

    pub fn create_ast_node(&mut self, node: &Node) -> AstNodeId {
        let ipos = ItemWithPos::new(node);

        let mut this_node = self.as_mut().orphan(ipos);

        for n in &node.children {
            Self::add_node(&mut this_node, n);
        }
        this_node.id()
    }

    pub fn replace_node(&mut self, old_node_id: AstNodeId, new_node_id: AstNodeId) {
        let mut old_node = self
            .as_mut()
            .get_mut(old_node_id)
            .expect("can't retrieve old node");
        old_node.insert_id_after(new_node_id);
        old_node.detach();
    }
    pub fn replace_node_take_children(&mut self, old_node_id: AstNodeId, new_node_id: AstNodeId) {
        let mut replacement_node = self
            .as_mut()
            .get_mut(new_node_id)
            .expect("Can't fetch replacement node");
        replacement_node.reparent_from_id_append(old_node_id);
        self.replace_node(old_node_id, new_node_id);
    }

    pub fn detach_nodes_filter<I, F>(&mut self, i: I, f: F) -> ThinVec<AstNodeId>
    where
        I: Iterator<Item = AstNodeId>,
        F: Fn(AstNodeRef) -> bool,
    {
        i.filter(|id| {
            let node = self.as_ref().get(*id).unwrap();
            if f(node) {
                self.as_mut().get_mut(*id).unwrap().detach();
                true
            } else {
                false
            }
        })
        .collect()
    }

    pub fn create_orphan(&mut self, item: AstNodeKind, pos: Position) -> AstNodeId
    {
        self.as_mut().orphan(ItemWithPos { item, pos }).id()
    }

    pub fn alter_node<F>(&mut self, node_id: AstNodeId, f: F)
    where
        F: Fn(&mut ItemWithPos),
    {
        let mut this_node_mut = self.as_mut().get_mut(node_id).unwrap();
        f(this_node_mut.value())
    }
}

impl AsRef<AstTree> for Ast
{
    fn as_ref(&self) -> &AstTree {
        &self.tree
    }
}
impl AsMut<AstTree> for Ast
{
    fn as_mut(&mut self) -> &mut AstTree {
        &mut self.tree
    }
}

/// AstCtx
/// Does semantic analysis and ast lowering
pub struct AstCtx<'a>
{
    pub ast_tree: Ast,
    pub macro_defs: ThinVec<AstNodeId>,
    pub ctx: &'a mut Assembler,
    pub docs: HashMap<AstNodeId, String>,
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

/// Return a vec of depth first node ids
fn get_ids_recursive(node: AstNodeRef) -> ThinVec<AstNodeId> 
{
    let mut ret = thin_vec![];
    get_recursive(node, &mut |x: AstNodeRef| ret.push(x.id()));
    ret
}

/// Depth first iteration of all node ids
pub fn iter_ids_recursive(node: AstNodeRef) -> impl Iterator<Item = AstNodeId>

{
    let mut i = get_ids_recursive(node).into_iter();
    iter::from_fn(move || i.next())
}

pub fn iter_refs_recursive(node: AstNodeRef) -> impl Iterator<Item = AstNodeRef> 
{
    let mut i = get_ids_recursive(node).into_iter();
    iter::from_fn(move || i.next().and_then(|id| node.tree().get(id)))
}

fn iter_items_recursive(node: AstNodeRef) -> impl Iterator<Item = (AstNodeId, &AstNodeKind)>

{
    let mut i = get_ids_recursive(node).into_iter();
    iter::from_fn(move || {
        i.next()
            .and_then(|id| node.tree().get(id).map(|n| (n.id(), &n.value().item)))
    })
}

#[allow(dead_code)]
fn iter_values_recursive(node: AstNodeRef
) -> impl Iterator<Item = (AstNodeId, &ItemWithPos)> 

{
    let mut i = get_ids_recursive(node).into_iter();
    iter::from_fn(move || {
        i.next()
            .and_then(|id| node.tree().get(id).map(|n| (n.id(), n.value())))
    })
}

impl<'a> AstCtx<'a>
{
    pub fn new(tree: Ast, ctx: &'a mut Assembler) -> Result<Self, UserError> {
        let mut ret = Self::base(tree, ctx);
        ret.process()?;
        Ok(ret)
    }

    pub fn from_nodes(ctx: &'a mut Assembler, node: &Node) -> Result<Self, UserError> {
        let tree = Ast::from_node(node);
        let r = Self::new(tree, ctx)?;
        Ok(r)
    }

    fn process(&mut self) -> Result<(), UserError> {
        status("Semantic analysis", |_| {
            self.inline_includes()?;
            self.gather_docs()?;
            self.create_scopes()?;
            self.postfix_expressions()?;
            self.rename_locals();
            self.process_macros_definitions()?;
            self.generate_struct_symbols()?;
            self.scope_assignments()?;
            self.process_imports()?;
            self.scope_labels()?;
            self.evaluate_assignments()?;
            Ok(())
        })
    }

    pub fn get_tree(&self) -> &AstTree {
        self.ast_tree.as_ref()
    }

    pub fn get_tree_mut(&mut self) -> &mut AstTree {
        self.ast_tree.as_mut()
    }

    /// Bring all of the imports needed into the correct namespaces
    fn process_imports(&mut self) -> Result<(), UserError> {
        use AstNodeKind::*;

        info("Processing imports", |_| {
            interesting_mess!("Import labels");

            let mut scopes = self.get_root_scope_tracker();

            for node_id in iter_ids_recursive(self.get_tree().root()) {
                let node = self.get_tree().get(node_id).unwrap();

                match &node.value().item {
                    ScopeId(scope_id) => scopes.set_scope(*scope_id),

                    Import => {
                        let ids: Vec<_> = node.children().map(|n| n.id()).collect();

                        self.scope_labels_node(node_id, scopes.clone())?;

                        for kid_id in ids {
                            let item = self
                                .get_tree()
                                .get(kid_id)
                                .expect("Internal error")
                                .value()
                                .item
                                .clone();

                            if let Label(LabelDefinition::Scoped(symbol_id)) = item {
                                let name = self
                                    .ctx
                                    .get_symbols()
                                    .get_symbol_info_from_id(symbol_id)
                                    .unwrap()
                                    .name()
                                    .to_owned();
                                self.ctx
                                    .get_symbols_mut()
                                    .add_reference_symbol(&name, scopes.scope(), symbol_id)
                                    .unwrap();
                            } else {
                                // TODO trying to import non existant labels should be an error
                                eprintln!("{:?}", item);
                                panic!()
                            }
                        }
                    }

                    _ => (),
                }
            }

            interesting_mess!("Importing symbols");

            let mut scopes = self.get_root_scope_tracker();

            for node_id in iter_ids_recursive(self.get_tree().root()) {
                let node = self.get_tree().get(node_id).unwrap();

                if let ScopeId(scope_id) = &node.value().item {
                    scopes.set_scope(*scope_id)
                }
            }

            Ok(())
        })
    }

    /// Remove all doc nodes
    /// and put into a doc databse
    fn gather_docs(&mut self) -> Result<(), UserError> {
        let mut doc_map: HashMap<AstNodeId, String> = HashMap::new();

        for id in iter_ids_recursive(self.get_tree().root()) {
            let node = self.get_tree().get(id).unwrap();

            if let AstNodeKind::Doc(text, ..) = &node.value().item {
                let parent_id = node.parent().unwrap().id();
                doc_map.insert(parent_id, text.to_string());
                self.get_tree_mut().get_mut(id).unwrap().detach();
            }
        }

        self.docs = doc_map;

        Ok(())
    }

    fn base(tree: Ast, ctx: &'a mut Assembler) -> Self {
        Self {
            ast_tree: tree,
            ctx,
            macro_defs: thin_vec![],
            docs: Default::default(),
        }
    }

    /// Find all of the includes in this AST and replace with the
    /// with inlines included files tokens
    fn inline_includes(&mut self) -> Result<(), UserError> {
        info("Inlining include files", |_| {
            // Loop over the ast until we have replaced all of the includes
            // each include can have includes in it as well
            let mut num_of_included_files = 0;
            loop {
                let tree = self.get_tree();
                // Get all of the include ids
                let include_ids: Vec<_> = iter_items_recursive(tree.root())
                    .filter_map(|(id, item)| {
                        item.unwrap_include()
                            .map(|p| (id, self.ctx.get_full_path(p).unwrap()))
                    })
                    .collect();

                num_of_included_files += include_ids.len();

                if include_ids.is_empty() {
                    if num_of_included_files == 0 {
                        debug_mess!("No includes files to inline");
                    } else {
                        debug_mess!("Finished inlining {num_of_included_files} include(s)");
                    }
                    break;
                }

                // Go through the ids and get the tokens to insert into this AST
                for (id, actual_file) in include_ids {
                    if let Some(tokens) = self.ctx.get_tokens_from_full_path(&actual_file) {
                        debug_mess!("Inlining {} - HAD TOKENS", actual_file.to_string_lossy());
                        let new_node_id = self.ast_tree.create_ast_node(&tokens.node);
                        self.ast_tree.replace_node(id, new_node_id);
                    } else {
                        let files = self.ctx.token_store.get_files();

                        panic!(
                            "Can't find include tokens for {}\nFiles Searched:\n{:?}",
                            actual_file.to_string_lossy(),
                            files
                        )
                    }
                }
            }

            Ok(())
        })
    }

    pub fn process_macros_definitions(&mut self) -> Result<(), UserError> {
        info("Processing macro definitions", |_| {
            // TODO: should be written in a way that can detect
            // redefinitions of a macro
            use AstNodeKind::{MacroCall, MacroCallProcessed, ScopeId};

            // Detach all of the MacroDef nodes
            let detached = self
                .ast_tree
                .detach_nodes_filter(iter_ids_recursive(self.get_tree().root()), |nref| {
                    matches!(nref.value().item, AstNodeKind::MacroDef(..))
                });

            let mdefs: HashMap<String, (AstNodeId, Vec<String>)> = detached
                .into_iter()
                .filter_map(|id| {
                    let item = &self.get_tree().get(id).unwrap().value().item;
                    item.unwrap_macro_def()
                        .map(|(nm, params)| (nm.into(), (id, params.to_vec())))
                })
                .collect();

            let mut mcalls = vec![];
            let mut scopes = self.get_root_scope_tracker();

            let root = self.ast_tree.tree.root();

            for (i, macro_call_node) in iter_refs_recursive(root).enumerate() {
                let val = &macro_call_node.value();

                match &val.item {
                    ScopeId(scope_id) => scopes.set_scope(*scope_id),

                    MacroCall(name) => {
                        let syms = &mut self.ctx.asm_out.symbols;
                        // Create a unique name for this macro application scope
                        let caller_scope_name = format!("%MACRO%_{name}_{i}");

                        // Create the scope
                        let macro_caller_scope_id =
                            syms.create_or_get_scope_for_parent(&caller_scope_name, scopes.scope());

                        let (macro_id, params) = mdefs.get(name).ok_or_else(|| {
                            self.node_error("Can't find macro", macro_call_node.id(), false)
                        })?;

                        mcalls.push((
                            *macro_id,
                            macro_caller_scope_id,
                            params,
                            macro_call_node.id(),
                            val.pos,
                        ))
                    }
                    _ => (),
                }
            }

            // Create new nodes to replace the current macro call nodes
            // let mut nodes_to_change: Vec<(AstNodeId, AstNodeId)> = vec![];
            for (macro_id, scope_id, params, caller_node_id, pos) in mcalls.into_iter() {
                let params_vec_of_id: Result<ThinVec<_>, _> = params
                    .iter()
                    .map(|p| self.create_symbol(p, caller_node_id, &ScopeTracker::new(scope_id)))
                    .collect();

                let item = MacroCallProcessed {
                    macro_id,
                    scope_id,
                    params_vec_of_id: params_vec_of_id?,
                };

                let replacement_id = self.ast_tree.create_orphan(item, pos);

                self.ast_tree
                    .replace_node_take_children(caller_node_id, replacement_id);
            }

            self.macro_defs = mdefs.values().map(|(a, _)| *a).collect();

            Ok(())
        })
    }

    fn get_source_info_from_node_id(&self, id: AstNodeId) -> Result<SourceInfo, SourceErrorType> {
        let n = self.get_tree().get(id).unwrap();
        self.ctx.sources().get_source_info(&n.value().pos)
    }

    fn rename_locals(&mut self) {
        info("Renaming locals into globals", |_x| {
            use AstNodeKind::*;
            use LabelDefinition::*;

            let mut label_scopes = ScopeBuilder::new();

            let map_name = |last_global: &String, l: &LabelDefinition| -> LabelDefinition {
                l.map_string(|name| format!("{last_global}/{name}"))
            };

            // Expand all local labels to have a scoped name
            // and change all locals to globals
            for v in self.get_tree_mut().values_mut() {
                let fqn = label_scopes.get_current_fqn();

                match &v.item {
                    AssignmentFromPc(Text(name)) => {
                        label_scopes.pop();
                        label_scopes.push_new(name);
                    }

                    LocalAssignmentFromPc(label) => {
                        v.item = AssignmentFromPc(map_name(&fqn, label))
                    }

                    LocalAssignment(label) => v.item = Assignment(map_name(&fqn, label)),

                    LocalLabel(label) => v.item = Label(map_name(&fqn, label)),

                    TokenizedFile(..) => {
                        label_scopes.pop();
                    }

                    _ => (),
                };
            }
        });
    }

    // Convert this node to from infix to postfix
    fn node_to_postfix(&mut self, node_id: AstNodeId) -> Result<(), UserError> {
        let tree = self.get_tree();

        let node = tree.get(node_id).expect("Can't fetch node id");
        assert!(node.value().item.is_expr());

        let args = node.children().map(Term::from).collect::<Vec<_>>();

        let ret = to_postfix(&args).map_err(|s| {
            let args: Vec<_> = args
                .iter()
                .map(|a| format!("{:?}", tree.get(a.node).unwrap().value().item))
                .collect();
            let msg = format!("\n{:?} {:?}\n {}", s, node.value(), args.join("\n"));
            let msg = format!("Can't convert to postfix: {msg}");
            self.node_error(msg, node_id, true)
        })?;

        // Remove and reappend each node

        for t in ret.iter().map(|t| t.node) {
            self.get_tree_mut()
                .get_mut(t)
                .expect("Couldn't Fetch mut child")
                .detach();
            self.get_tree_mut().get_mut(node_id).unwrap().append_id(t);
        }

        self.get_tree_mut().get_mut(node_id).unwrap().value().item = AstNodeKind::PostFixExpr;

        Ok(())
    }

    fn postfix_expressions(&mut self) -> Result<(), UserError> {
        info("Converting expressions to poxtfix", |_| {
            for id in iter_ids_recursive(self.get_tree().root()) {
                if self.get_tree().get(id).unwrap().value().item.is_expr() {
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
        let node = self.get_tree().get(id).unwrap();
        let si = &self.get_source_info_from_node_id(node.id()).unwrap();
        UserError::from_text(msg, si, is_failure)
    }

    fn eval_node(&self, id: AstNodeId, current_scope_id: u64) -> Result<i64, UserError> {
        use super::gazmeval::eval;
        let node = self.get_tree().get(id).unwrap();
        let item = &node.value().item;
        let err = |m| self.node_error(m, id, true);

        if let AstNodeKind::PostFixExpr = item {
            let reader = self.ctx.asm_out.symbols.get_reader(current_scope_id);
            eval(&reader, node).map_err(|e| self.convert_error(e.into()))
        } else {
            Err(err(&format!(
                "Incorrect item type for evalulation : {item:?}"
            )))
        }
    }

    fn eval_node_child(&self, id: AstNodeId, current_scope_id: u64) -> Result<i64, UserError> {
        let err = |m| self.node_error(m, id, true);
        let node = self.get_tree().get(id).unwrap();
        let first_child = node
            .first_child()
            .ok_or_else(|| err("Can't find a child node"))?;

        self.eval_node(first_child.id(), current_scope_id)
    }

    fn get_root_scope_tracker(&self) -> ScopeTracker {
        ScopeTracker::new(self.ctx.asm_out.symbols.get_root_scope_id())
    }

    fn generate_struct_symbols(&mut self) -> Result<(), UserError> {
        let scopes = self.get_root_scope_tracker();

        info("Generating symbols for struct definitions", |_| {
            use AstNodeKind::*;

            // self.ctx.asm_out.symbols.set_root();

            // let mut symbols = self.symbols.clone();
            let tree = self.get_tree();

            for id in iter_ids_recursive(tree.root()) {
                let item = &self.get_tree().get(id).unwrap().value().item.clone();

                if let StructDef(name) = item {
                    let mut current = 0;
                    interesting_mess!("Generating symbols for {name}");

                    let kids_ids = self.ast_tree.get_kids_ids(id);

                    for c_id in kids_ids {
                        let i = &self.get_tree().get(c_id).unwrap().value().item;

                        if let StructEntry(entry_name) = i {
                            debug_mess!("Generating struct entry: {name} {entry_name}");
                            let value = self.eval_node_child(c_id, scopes.scope())?;
                            let scoped_name = format!("{name}.{entry_name}");
                            debug_mess!("About to create sym: {name} {entry_name}");
                            self.create_and_set_symbol(current, &scoped_name, c_id, &scopes)?;
                            debug_mess!("Struct: Set {scoped_name} to {current}");
                            current += value;
                        }
                    }

                    let scoped_name = format!("{name}.size");
                    self.create_and_set_symbol(current, &scoped_name, id, &scopes)?;
                }
            }

            Ok(())
        })
    }

    /// Traverse all nodes and create scopes from Scope(name)
    /// and change node from Scope(name) -> ScopeId(scope_id)
    fn create_scopes(&mut self) -> Result<(), UserError> {
        use AstNodeKind::*;

        let scopes = self.get_root_scope_tracker();

        for node_id in iter_ids_recursive(self.get_tree().root()) {
            let item = &self.get_tree().get(node_id).unwrap().value().item.clone();

            if let Scope(scope) = item {
                let id = self.get_writer(&scopes).create_or_set_scope(scope.as_str());
                self.ast_tree
                    .alter_node(node_id, |ipos| ipos.item = ScopeId(id));
            }
        }

        Ok(())
    }

    fn get_writer(&mut self, scopes: &ScopeTracker) -> SymbolTreeWriter {
        self.ctx.asm_out.symbols.get_writer(scopes.scope())
    }

    fn get_reader(&self, scopes: &ScopeTracker) -> SymbolTreeReader {
        self.ctx.asm_out.symbols.get_reader(scopes.scope())
    }

    fn get_scoped_symbol_id(
        &self,
        scoped_name: &ScopedName,
        node_id: AstNodeId,
    ) -> Result<SymbolScopeId, UserError> {
        let symbol_id = self
            .ctx
            .get_symbols()
            .get_symbol_info_from_scoped_name(scoped_name)
            .map(|si| si.symbol_id)
            .map_err(|e| self.sym_to_user_error(e, node_id))?;
        Ok(symbol_id)
    }

    fn get_unscoped_symbol_id(
        &self,
        name: &str,
        scopes: &ScopeTracker,
        node_id: AstNodeId,
    ) -> Result<SymbolScopeId, UserError> {
        let id = self
            .get_reader(scopes)
            .get_symbol_info(name)
            .map_err(|e| self.sym_to_user_error(e, node_id))?
            .symbol_id;
        Ok(id)
    }

    fn scope_labels_node(
        &mut self,
        id: AstNodeId,
        mut scopes: ScopeTracker,
    ) -> Result<(), UserError> {
        use AstNodeKind::*;

        let root_node = self.get_tree().get(id).unwrap();

        let nodes = get_ids_recursive(root_node);

        for node_id in &nodes {
            let value = self.get_tree().get(*node_id).unwrap().value().clone();

            match &value.item {
                ScopeId(scope_id) => scopes.set_scope(*scope_id),

                // Convert any label in tree to a lable reference
                Label(LabelDefinition::Text(name)) => {
                    let symbol_id = self.get_unscoped_symbol_id(name, &scopes, *node_id)?;
                    self.ast_tree
                        .alter_node(*node_id, |ipos| ipos.item = Label(symbol_id.into()));
                }

                Label(LabelDefinition::TextScoped(name)) => {
                    let scoped_name = ScopedName::new(name);
                    let symbol_id = self.get_scoped_symbol_id(&scoped_name, *node_id)?;
                    self.ast_tree
                        .alter_node(*node_id, |ipos| ipos.item = Label(symbol_id.into()));
                }

                _ => (),
            }
        }
        Ok(())
    }

    fn scope_labels(&mut self) -> Result<(), UserError> {
        info("Scoping labels", |_| {
            let root_node_id = self.get_tree().root().id();

            let scopes = self.get_root_scope_tracker();

            interesting_mess!("Scoping AST labels");
            self.scope_labels_node(root_node_id, scopes)?;

            use AstNodeKind::*;

            interesting_mess!("Scoping macro labels");

            for mac_nodes in self.macro_defs.clone().into_iter() {
                let node = self.get_tree().get(mac_nodes).unwrap();

                for node_id in get_ids_recursive(node).into_iter() {
                    let value = self.get_tree().get(node_id).unwrap().value().clone();

                    if let Label(LabelDefinition::TextScoped(name)) = &value.item {
                        let scoped_name = ScopedName::new(name);
                        let symbol_id = self.get_scoped_symbol_id(&scoped_name, node_id)?;
                        self.ast_tree
                            .alter_node(node_id, |ipos| ipos.item = Label(symbol_id.into()));
                    }
                }
            }
            Ok(())
        })
    }

    /// Traverse through all assignments and reserve a label for them at the correct scope
    fn scope_assignments(&mut self) -> Result<(), UserError> {
        info("Scoping assignments", |_| {
            use AstNodeKind::*;
            let mut scopes = self.get_root_scope_tracker();

            for node_id in iter_ids_recursive(self.get_tree().root()) {
                let value = self.get_tree().get(node_id).unwrap().value();
                let item = &value.item.clone();

                match item {
                    ScopeId(scope_id) => scopes.set_scope(*scope_id),

                    AssignmentFromPc(LabelDefinition::Text(name)) => {
                        debug_mess!("Assignment from PC: {name}");
                        let sym_id = self.create_symbol(name, node_id, &scopes)?;
                        self.ast_tree.alter_node(node_id, |ipos| {
                            ipos.item = AssignmentFromPc(sym_id.into());
                        });
                    }

                    Assignment(LabelDefinition::Text(name)) => {
                        debug_mess!("Assignment: {name}");
                        let sym_id = self.create_symbol(name, node_id, &scopes)?;
                        self.ast_tree.alter_node(node_id, |ipos| {
                            ipos.item = Assignment(sym_id.into());
                        });
                    }
                    _ => (),
                }
            }

            Ok(())
        })
    }

    fn evaluate_assignments(&mut self) -> Result<(), UserError> {
        info("Evaluating assignments", |_| {
            use super::gazmeval::eval;
            use AstNodeKind::*;

            let mut scopes = self.get_root_scope_tracker();

            for node_id in iter_ids_recursive(self.get_tree().root()) {
                match &self.get_tree().get(node_id).unwrap().value().item {
                    ScopeId(scope_id) => scopes.set_scope(*scope_id),

                    Assignment(LabelDefinition::Scoped(label_id)) => {
                        let label_id = *label_id;
                        let expr = self.get_tree().get(node_id).unwrap().first_child().unwrap();
                        let reader = self.ctx.get_symbols().get_reader(scopes.scope());
                        let res = eval(&reader, expr);

                        match res {
                            Ok(value) => self
                                .set_symbol(label_id, node_id, value)
                                .expect("Can't set symbol"),

                            Err(EvalError {
                                source: EvalErrorEnum::CotainsPcReference,
                                ..
                            }) => {
                                self.ast_tree.alter_node(node_id, |ipos| {
                                    ipos.item = AstNodeKind::AssignmentFromPc(label_id.into())
                                });
                            }

                            Err(e) => {
                                let _reader = self.ctx.get_symbols().get_reader(scopes.scope());
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
// Node management stuff
impl<'a> AstCtx<'a>
{
    fn set_symbol(
        &mut self,
        symbol_id: SymbolScopeId,
        node_id: AstNodeId,
        val: i64,
    ) -> Result<(), UserError> {
        self.ctx
            .get_symbols_mut()
            .set_symbol_for_id(symbol_id, val)
            .map_err(|e| self.sym_to_user_error(e, node_id))
    }

    pub fn create_symbol(
        &mut self,
        name: &str,
        id: AstNodeId,
        scopes: &ScopeTracker,
    ) -> Result<SymbolScopeId, UserError> {
        let mut writer = self.ctx.get_symbols_mut().get_writer(scopes.scope());
        writer
            .create_symbol(name)
            .map_err(|e| self.sym_to_user_error(e, id))

        // let syms = self.ctx.get_symbols_mut();
        // let sym_id = syms.create_symbol_in_scope(scopes.scope(), name)
        //     .map_err(|e| self.sym_to_user_error(e, id))?;

        // Ok(sym_id)
    }

    pub fn create_and_set_symbol(
        &mut self,
        value: i64,
        name: &str,
        id: AstNodeId,
        scopes: &ScopeTracker,
    ) -> Result<SymbolScopeId, UserError> {
        let symbol_id = self.create_symbol(name, id, scopes)?;
        self.ctx
            .get_symbols_mut()
            .set_symbol_for_id(symbol_id, value)
            .map_err(|e| self.sym_to_user_error(e, id))?;
        Ok(symbol_id)
    }

    fn sym_to_user_error(&self, e: SymbolError, id: AstNodeId) -> UserError {
        let msg = format!("Symbol error {e:?}");
        self.node_error(msg, id, false)
    }
}

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

pub fn to_priority(i: &AstNodeKind) -> Option<usize>
{
    use AstNodeKind::*;
    match i {
        Mul | Div => Some(12),
        Add | Sub => Some(11),
        ShiftL | ShiftR => Some(10),
        BitAnd => Some(9),
        BitXor => Some(8),
        BitOr => Some(7),
        _ => None,
    }
}

impl From<AstNodeRef<'_>> for Term
{
    fn from(node: AstNodeRef) -> Self {
        Self {
            node: node.id(),
            priority: to_priority(&node.value().item),
        }
    }
}
