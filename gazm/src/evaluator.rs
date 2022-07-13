use emu::utils::sources::{
    Position, SourceFileLoader, SourceInfo, Sources, SymbolError, SymbolInfo, SymbolQuery,
    SymbolTree, SymbolWriter,
};

use crate::ast::{AstNodeId, AstNodeRef, AstTree};
use crate::error::UserError;
use crate::eval::eval;
use crate::error::{GazmError, GResult };
use crate::item::Item::*;

use crate::ctx::Context;

/// Evaluates things
pub struct Evaluator<'a> {
    pub symbols: &'a mut SymbolTree,
    pub source_file_loader: &'a mut SourceFileLoader,
}

use std::path::PathBuf;

impl<'a> SymbolQuery for Evaluator<'a> {
    fn get_symbol_info(&self, name: &str) -> Result<&SymbolInfo, SymbolError> {
        self.symbols.get_symbol_info(name)
    }
}

impl<'a> Evaluator<'a> {
    pub fn new(symbols: &'a mut SymbolTree, source_file_loader: &'a mut SourceFileLoader) -> Self {
        Self {
            symbols,
            source_file_loader,
        }
    }

    /// Evaluate all macro args
    /// if all arguments were evaluated returns true

    pub fn eval_macro_args(
        &mut self,
        scope: &str,
        node: AstNodeRef,
        macro_node: AstNodeRef,
    ) -> bool {
        if let MacroDef(_name, params) = &macro_node.value().item {
            self.symbols.set_scope(scope);
            let mut assignments = vec![];

            let mut args_evaled = 0;

            let args = node.children();

            args.zip(params).for_each(|(arg, param)| {
                if !self
                    .symbols
                    .get_current_scope_symbols()
                    .symbol_exists_from_name(param)
                {
                    let evaled = self.eval_node(arg);
                    if let Ok(value) = evaled {
                        assignments.push((param.clone(), value));
                        args_evaled += 1;
                    }
                } else {
                    args_evaled += 1;
                }
            });


            for (param, value) in assignments {
                let res = self.symbols.add_symbol_with_value(&param, value);
                if res.is_err() {
                    println!("{}", self.symbols.get_current_scope_symbols());
                    panic!();
                }
            }
            self.symbols.pop_scope();

            args_evaled == params.len()
        } else {
            panic!()
        }
    }

    pub fn get_symbols_mut(&mut self) -> &mut SymbolTree {
        self.symbols
    }

    pub fn get_symbols(&self) -> &SymbolTree {
        self.symbols
    }

    pub fn get_children(&self, node: AstNodeRef) -> Vec<AstNodeId> {
        node.children().map(|n| n.id()).collect()
    }

    pub fn eval_node(&self, node: AstNodeRef) -> GResult<i64> {
        let info = self.get_source_info(&node.value().pos).unwrap();
        eval(self, node).map_err(|err| {
            let e = match &err.source {
                crate::eval::EvalErrorEnum::SymbolNotFoud(name) => {
                    let scope = self.get_symbols().get_current_scope_fqn();
                    let mut err = err.clone();
                    err.source =
                        crate::eval::EvalErrorEnum::SymbolNotFoud(format!("{scope}::{name}"));
                    UserError::from_ast_error(err.into(), &info)
                }
                _ => UserError::from_ast_error(err.into(), &info),
            };
            e.into()
        })
    }

    pub fn get_source_info(&self, pos: &Position) -> Result<SourceInfo, String> {
        self.source_file_loader.sources.get_source_info(pos)
    }

    pub fn eval_with_pc(&mut self, n: AstNodeRef, pc: u64) -> GResult<i64> {
        self.symbols.add_symbol_with_value("*", pc as i64).unwrap();
        let ret = self.eval_node(n)?;
        self.symbols.remove_symbol_name("*");
        Ok(ret)
    }
    pub fn user_error<S: Into<String>>(
        &self,
        err: S,
        node: AstNodeRef,
        is_failure: bool,
    ) -> UserError {
        let info = self.get_source_info(&node.value().pos).unwrap();
        UserError::from_text(err, &info, is_failure)
    }

    pub fn eval_first_arg(&self, node: AstNodeRef) -> GResult<(i64, AstNodeId)> {
        let c = node
            .first_child()
            .ok_or_else(|| self.user_error("Missing argument", node, true))?;
        let v = self.eval_node(c)?;
        Ok((v, c.id()))
    }

    pub fn eval_two_args(&self, node: AstNodeRef) -> GResult<(i64, i64)> {
        let args = self.eval_n_args(node, 2)?;
        Ok((args[0], args[1]))
    }

    pub fn try_eval_n_args(&self, node: AstNodeRef, n: usize) -> Vec<Option<i64>> {
        let mut ret = vec![];

        for (i, node) in node.children().enumerate() {
            if i == n {
                break;
            }
            let to_push = if let Ok(v) = self.eval_node(node) {
                Some(v)
            } else {
                None
            };
            ret.push(to_push)
        }

        ret
    }

    pub fn eval_n_args(&self, node: AstNodeRef, n: usize) -> GResult<Vec<i64>> {
        let mut ret = vec![];

        for (i, node) in node.children().enumerate() {
            if i == n {
                break;
            }
            let v = self.eval_node(node)?;
            ret.push(v)
        }

        Ok(ret)
    }
}
