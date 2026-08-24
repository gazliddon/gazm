#![allow(unused_imports)]
mod asm;
mod binary;
mod bytesizes;
mod compile;
mod edit;
mod evaluator;
mod plan;
mod scopes;
mod scopetracker;
mod sizer;
mod traits;
mod writers;

pub use asm::*;
pub use binary::*;
pub use bytesizes::*;
pub use compile::*;
pub use edit::*;
pub use evaluator::*;
pub use plan::*;
pub use scopes::*;
pub use scopetracker::*;
pub use sizer::*;
pub use traits::*;
pub use writers::*;

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use crate::frontend::AstNodeKindDiscriminants;

    use super::{compile::compiler_handles, sizer::sizer_emits};

    /// The kinds the sizer can record in the walk plan must be exactly the
    /// kinds the compiler can replay. Anything else either errors at size
    /// time or panics at compile time — and a kind added to one pass but not
    /// the other would silently corrupt the output.
    #[test]
    fn sizer_and_compiler_agree_on_plan_kinds() {
        let mut got: Vec<_> = AstNodeKindDiscriminants::iter()
            .filter(|k| sizer_emits(k) != compiler_handles(k))
            .collect();
        got.sort_by_key(|k| format!("{k:?}"));

        assert!(
            got.is_empty(),
            "sizer and compiler disagree on plan kinds: {got:?}; \
             a kind emitted by one pass but not handled by the other \
             silently corrupts the output"
        );
    }
}
