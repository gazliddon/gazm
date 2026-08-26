use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Copy)]
pub enum Z80AssemblyErrorKind {
    #[error("Unknown Z80 opcode")]
    UnknownOpcode,
    #[error("This opcode needs an operand")]
    MissingOperand,
    #[error("Operands do not match any form of this opcode")]
    OperandsDontMatch,
}
