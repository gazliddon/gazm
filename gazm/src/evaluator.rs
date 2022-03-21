use utils::sources::{SymbolQuery, SymbolTree, SymbolWriter};

use crate::ast::{AstNodeId, AstNodeRef };
use crate::error::UserError;
use crate::eval::eval;
use crate::gasm::GResult;

use utils::sources::{Position, SourceInfo, Sources, SymbolError, SymbolInfo};

/// Evaluates things
pub struct Evaluator<'a> {
    pub symbols : SymbolTree,
    pub sources : &'a Sources,
    ctx: &'a crate::ctx::Context,
    // errors (mut)
    // sources
    // opts
    // mut symbols
}
use std::path::PathBuf;

impl<'a> SymbolQuery for Evaluator<'a> {
    fn get_symbol_info(&self, name: &str) -> Result<&SymbolInfo, SymbolError> {
        self.symbols.get_symbol_info(name)
    }
}

impl<'a> Evaluator<'a> {

   pub fn get_binary_extents(
        &self,
        file_name: PathBuf,
        node: AstNodeRef,
    ) -> GResult<std::ops::Range<usize>> {
        let data_len = self
            .ctx
            .get_size(file_name)
            .map_err(|e| self.user_error(e.to_string(), node, true))?;

        let mut r = 0..data_len;

        let mut c = node.children();

        let offset_size = c
            .next()
            .and_then(|offset| c.next().map(|size| (offset, size)));

        if let Some((offset, size)) = offset_size {
            let offset = self.eval_node(offset)?;
            let size = self.eval_node(size)?;
            let offset = offset as usize;
            let size = size as usize;
            let last = (offset + size) - 1;

            if !(r.contains(&offset) && r.contains(&last)) {
                let msg =
                    format!("Trying to grab {offset:04X} {size:04X} from file size {data_len:X}");
                return Err(self.user_error(msg, node, true).into());
            };

            r.start = offset;
            r.end = offset + size;
        }

        Ok(r)
    }


    pub fn eval_macro_args(&mut self, scope: &String, node: AstNodeRef, macro_node: AstNodeRef) {

        use crate::item::Item::*;

        self.symbols.set_scope(scope);

        let mut assignments = vec![];

        if let MacroDef(.., params) = &macro_node.value().item {
            let args = node.children();
            args.zip(params).for_each(|(arg, param)| {
                if !self.symbols.symbol_exists_from_name(param) {
                    let evaled = self.eval_node(arg);
                    if let Ok(value) = evaled {
                        assignments.push((param.clone(), value));
                    }
                }
            });
        }

        for (param, value) in assignments {
            self
                .symbols
                .add_symbol_with_value(&param, value)
                .unwrap();
        }

        self.symbols.pop_scope();
    }

    pub fn get_symbols_mut(&mut self) -> &mut SymbolTree {
        &mut self.symbols
    }

    pub fn get_children(&self, node: AstNodeRef) -> Vec<AstNodeId> {
        node.children().map(|n| n.id()).collect()
    }

    pub fn eval_node(&self, node: AstNodeRef) -> GResult<i64> {
        eval(self, node).map_err(|err| {
            let info = self.get_source_info(&node.value().pos).unwrap();
            UserError::from_ast_error(err.into(), &info).into()
        })
    }

    pub fn get_source_info(&self, pos: &Position) -> Result<SourceInfo, String> {
        self.sources.get_source_info(pos)
    }

    pub fn eval_with_pc(&mut self, n: AstNodeRef, pc: u64) -> GResult<i64> {
        self
            .symbols
            .add_symbol_with_value("*", pc as i64)
            .unwrap();
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
