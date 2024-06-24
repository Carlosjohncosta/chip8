use super::emu_err::EmuErr;

const STACK_LENGTH: usize = 0x10;

pub struct Stack {
    stack: [u16; STACK_LENGTH],
    sp: usize,
}

impl Stack {
    pub fn new() -> Self {
        Stack {
            stack: [0; STACK_LENGTH],
            sp: 0,
        }
    }

    pub fn push(&mut self, val: u16) -> Result<(), EmuErr> {
        if self.sp >= STACK_LENGTH {
            return Err(EmuErr::StackOverflow { sp: self.sp });
        }
        self.stack[self.sp] = val;
        self.sp += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Result<u16, EmuErr> {
        if self.sp == 0 {
            return Err(EmuErr::StackUnderflow { sp: self.sp });
        }
        self.sp -= 1;
        let val = self.stack[self.sp];
        Ok(val)
    }
}
