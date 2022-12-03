mod error;
mod eval;
/// Generics for evaluating infix expressions
mod postfix;
pub use error::*;
pub use eval::*;
pub use postfix::*;

use std::fmt::Debug;

pub fn infix_expr_to_value<I, E, ERR>(i: &[I], evaluator: &E) -> Result<I::ExprValue, ERR>
where
    I: ExprItemTraits + Clone + From<I::ExprValue> + Debug + GetPriority,
    <I as ExprItemTraits>::ExprValue: OperatorTraits,
    E: EvalExpr<I, ERR>,
    ERR: From<GenericEvalErrorKind>,
{
    let post_fix_expr = to_postfix(i).map_err(|e| GenericEvalErrorKind::PostFixError(e))?;
    let res = evaluate_postfix_expr(post_fix_expr.into_iter(), evaluator).map_err(|(_, e)| e)?;
    Ok(res)
}


