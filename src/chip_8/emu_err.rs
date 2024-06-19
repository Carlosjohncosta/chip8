use super::instruction::Instruction;
use std::fmt;

#[derive(Debug)]
pub enum EmuErr {
    ProgramLength { pg_len: usize, max_len: usize },
    BadInstruction { pc: u16, instruction: Instruction },
    PcOutOfBounds { pc: u16 },
    StackUnderflow { sp: usize },
    StackOverflow { sp: usize },
    IregOverflow { ireg: u16, offset: u16 },
}

impl fmt::Display for EmuErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use EmuErr::*;
        match self {
            ProgramLength { pg_len, max_len } => {
                write!(
                    f,
                    "Program of length {:#04x} too large for available mem {:#04x}",
                    pg_len, max_len
                )
            }
            BadInstruction { pc, instruction } => {
                write!(f, "Bad instruction {instruction} at location {:#04x}", pc)
            }
            PcOutOfBounds { pc } => {
                write!(f, "pc out of bounds at {:#04x}", pc)
            }
            StackUnderflow { sp } => {
                write!(f, "Attempted to push with sp: {:#04x}", sp)
            }
            StackOverflow { sp } => {
                write!(f, "Attempted to pop with sp: {:#04x}", sp)
            }
            IregOverflow { ireg, offset } => {
                write!(f, "Attempted to add {:#04x} to ireg {:#04x}", offset, ireg)
            }
        }
    }
}
