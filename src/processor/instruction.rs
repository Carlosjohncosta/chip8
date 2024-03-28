pub struct Instruction {
    instruction: u16,
}

impl Instruction {
    pub fn new(instruction: u16) -> Self {
        Self { instruction }
    }

    pub fn get_instruction(&self) -> u16 {
        self.instruction
    }

    /*
     * All bellow methods are used to index certain portions of the instruction.
     * Inlined as to not impose overhead of function call (methods are a single expression)
     */

    #[inline]
    pub fn nnn(&self) -> u16 {
        self.instruction & 0xFFF
    }

    #[inline]
    pub fn kk(&self) -> u8 {
        (self.instruction & 0xFF) as u8
    }

    #[inline]
    pub fn x(&self) -> u8 {
        ((self.instruction >> 8) & 0xF) as u8
    }

    #[inline]
    pub fn y(&self) -> u8 {
        ((self.instruction >> 4) & 0xF) as u8
    }

    #[inline]
    pub fn low_nibble(&self) -> u8 {
        (self.instruction & 0xF) as u8
    }

    #[inline]
    pub fn high_nibble(&self) -> u8 {
        ((self.instruction >> 12) & 0xF) as u8
    }
}