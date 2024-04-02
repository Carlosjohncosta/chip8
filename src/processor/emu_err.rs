use super::instruction::Instruction;
use std::{fmt, ops::Range};

#[derive(Debug)]
pub enum EmuErr {
    PgSize { pg_len: usize, max_len: usize },
    BadAddr { addr: u16 },
    BadMemSlice { range: Range<usize> },
    BadInstruction { pc: u16, instruction: Instruction },
    BadPc { pc: u16 },
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
            BadAddr { addr } => {
                write!(f, "Attempted to acess address: {:#04x}", addr)
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
        }
    }
}
