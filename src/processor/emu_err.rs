use super::instruction::Instruction;
use std::fmt;

pub enum EmuErr {
    PgSize { pg_len: u32, max_len: u32 },
    BadAddr { pc: u16, max: u16 },
    BadInstruction { pc: u16, instruction: Instruction },
}

impl fmt::Display for EmuErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmuErr::PgSize { pg_len, max_len } => {
                write!(
                    f,
                    "Program of length {:#04x} too large for available mem {:#04x}",
                    pg_len, max_len
                )
            }
            EmuErr::BadAddr { pc, max } => {
                write!(
                    f,
                    "Attempted to index address {:#04x} (Max address: {:#04x})",
                    pc, max
                )
            }
            EmuErr::BadInstruction { pc, instruction } => {
                write!(f, "Bad instruction {instruction} at location {:#04x}", pc)
            }
        }
    }
}
