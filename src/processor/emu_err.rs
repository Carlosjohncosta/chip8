use super::instruction::Instruction;
use std::{fmt, ops::Range};

#[derive(Debug)]
pub enum EmuErr {
    PgSize { pg_len: usize, max_len: usize },
    BadMemIndex { index: usize },
    BadMemSlice { range: Range<usize> },
    BadInstruction { pc: usize, instruction: Instruction },
    BadPc { pc: usize },
    BadPush { sp: usize },
    BadPop { sp: usize },
}

impl fmt::Display for EmuErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use EmuErr::*;
        match self {
            PgSize { pg_len, max_len } => {
                write!(
                    f,
                    "Program of length {:#04x} too large for available mem {:#04x}",
                    pg_len, max_len
                )
            }
            BadMemIndex { index } => {
                write!(f, "Attempted to acess address: {:#04x}", index)
            }
            BadMemSlice { range } => {
                write!(
                    f,
                    "Attempted to slice memory with bounds: {:#04x}..={:#04x}",
                    range.start, range.end
                )
            }
            BadInstruction { pc, instruction } => {
                write!(f, "Bad instruction {instruction} at location {:#04x}", pc)
            }
            BadPc { pc } => {
                write!(f, "pc out of bounds at {:#04x}", pc)
            }
            BadPush { sp } => {
                write!(f, "Attempted to push with sp: {:#04x}", sp)
            }
            BadPop { sp } => {
                write!(f, "Attempted to pop with sp: {:#04x}", sp)
            }
        }
    }
}
