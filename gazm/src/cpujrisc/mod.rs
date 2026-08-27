//! The Jaguar RISC (GPU/DSP, "JRISC") backend.
//!
//! Assembles the Tempest 2000 GPU sources (`src/gpu/*.gas`): a fixed
//! 16-bit instruction word (`op<<10 | r1<<5 | r2`), 32-bit `movei`
//! immediates stored low word first, and rmac-compatible operand forms
//! (`r0..r31`, REGEQU/EQU register names, `#n` quick immediates,
//! `(r14+n)` index loads/stores, `jr`/`jump` condition codes).

pub mod assembler;
pub mod frontend;
pub mod isa;
pub mod preprocess;
