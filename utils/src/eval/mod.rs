mod error;
mod eval_postfix;
mod postfix;
pub use error::*;
pub use eval_postfix::*;
pub use postfix::*;

use std::fmt::Debug;

pub fn infix_expr_to_value<I, E, ERR>(i: &[I], evaluator: &E) -> Result<I::ExprValue, ERR>
where
    I: ItemTraits + Clone + From<I::ExprValue> + Debug + GetPriority,
    <I as ItemTraits>::ExprValue: OperatorTraits,
    E: Eval<I, ERR>,
    ERR: From<GenericEvalErrorKind>,
{
    let post_fix_expr = to_postfix(i).map_err(GenericEvalErrorKind::PostFixError)?;
    let res = evaluate_postfix_expr(post_fix_expr.into_iter(), evaluator).map_err(|(_, e)| e)?;
    Ok(res)
}


