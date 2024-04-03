use std::fmt::Display;

#[derive(Debug)]
pub struct Instruction {
    instruction: u16,
}

impl Instruction {
    pub fn new(instruction: u16) -> Self {
        Self { instruction }
    }

    /*
     * All bellow methods are used to index certain portions of the instruction.
     * Inlined as to not impose overhead of function call (methods are a single expression)
     */

    #[inline]
    pub fn nnn(&self) -> usize {
        (self.instruction & 0xFFF) as usize
    }

    #[inline]
    pub fn kk(&self) -> u8 {
        (self.instruction & 0xFF) as u8
    }

    #[inline]
    pub fn x(&self) -> usize {
        ((self.instruction >> 8) & 0xF) as usize
    }

    #[inline]
    pub fn y(&self) -> usize {
        ((self.instruction >> 4) & 0xF) as usize
    }

    #[inline]
    pub fn low_nibble(&self) -> usize {
        (self.instruction & 0xF) as usize
    }

    #[inline]
    pub fn high_nibble(&self) -> usize {
        ((self.instruction >> 12) & 0xF) as usize
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#04x}", self.instruction)
    }
}
