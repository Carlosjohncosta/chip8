use super::emu_err::EmuErr;
use std::ops::{Range, RangeBounds};

pub trait Sliceable {
    type Output;
    fn len(&self) -> usize;
    fn inner_slice(&self) -> &[Self::Output];
    fn inner_slice_mut(&mut self) -> &mut [Self::Output];
    fn bad_index_err(addr: usize) -> EmuErr;
    fn bad_slice_err(range: Range<usize>) -> EmuErr;

    fn get_range(&self, range: impl RangeBounds<usize>) -> Range<usize> {
        let len = self.len();
        let start = match range.start_bound() {
            std::ops::Bound::Included(&start) => start as usize,
            std::ops::Bound::Excluded(&start) => start as usize + 1,
            std::ops::Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            std::ops::Bound::Included(&end) => end as usize + 1,
            std::ops::Bound::Excluded(&end) => end as usize,
            std::ops::Bound::Unbounded => len,
        };
        start..end
    }
    fn get(&self, addr: usize) -> Result<&Self::Output, EmuErr> {
        self.inner_slice()
            .get(addr)
            .ok_or(Self::bad_index_err(addr))
    }

    fn get_mut(&mut self, addr: usize) -> Result<&mut Self::Output, EmuErr> {
        self.inner_slice_mut()
            .get_mut(addr)
            .ok_or(Self::bad_index_err(addr))
    }

    fn slice(&self, range: impl RangeBounds<usize>) -> Result<&[Self::Output], EmuErr> {
        let range = self.get_range(range);
        self.inner_slice()
            .get(range.clone())
            .ok_or(Self::bad_slice_err(range))
    }

    fn slice_mut(&mut self, range: impl RangeBounds<usize>) -> Result<&mut [Self::Output], EmuErr> {
        let range = self.get_range(range);
        self.inner_slice_mut()
            .get_mut(range.clone())
            .ok_or(Self::bad_slice_err(range))
    }
}

const MEM_SIZE: usize = 0x1000;
const PG_START: usize = 0x200;
const FONT_DATA: [u8; 0x50] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Memory {
    memory: [u8; MEM_SIZE],
}

impl Memory {
    pub fn init_with(program: &[u8]) -> Result<Self, EmuErr> {
        let pg_len = program.len();
        let max_len = MEM_SIZE - PG_START;
        if pg_len > max_len {
            return Err(EmuErr::PgSize { pg_len, max_len });
        }

        //Checks if program fits into memory.
        let mut memory = [0u8; MEM_SIZE];

        //Loads number sprites into memory, from 0x0 to 0x50.
        for (m_byte, f_byte) in memory.iter_mut().zip(FONT_DATA.iter()) {
            *m_byte = *f_byte;
        }

        //Slize of memory that will hold the program, typically starting at 0x200.
        let mem_pg_slice = &mut memory[PG_START..];

        //Inserts program into mem slice.
        for (m_byte, p_byte) in mem_pg_slice.iter_mut().zip(program.iter()) {
            *m_byte = *p_byte;
        }

        Ok(Self { memory })
    }
}

impl Sliceable for Memory {
    type Output = u8;

    fn bad_index_err(addr: usize) -> EmuErr {
        EmuErr::BadMemIndex { index: addr }
    }

    fn bad_slice_err(range: Range<usize>) -> EmuErr {
        EmuErr::BadMemSlice { range }
    }

    fn len(&self) -> usize {
        self.memory.len()
    }

    fn inner_slice(&self) -> &[Self::Output] {
        &self.memory
    }

    fn inner_slice_mut(&mut self) -> &mut [Self::Output] {
        &mut self.memory
    }
}

pub struct VReg {
    v_reg: [u8; 0x10],
}

impl VReg {
    pub fn new() -> Self {
        Self { v_reg: [0; 0x10] }
    }
}

impl Sliceable for VReg {
    type Output = u8;

    fn bad_index_err(addr: usize) -> EmuErr {
        EmuErr::BadMemIndex { index: addr }
    }

    fn bad_slice_err(range: Range<usize>) -> EmuErr {
        EmuErr::BadMemSlice { range }
    }

    fn len(&self) -> usize {
        self.v_reg.len()
    }

    fn inner_slice(&self) -> &[Self::Output] {
        &self.v_reg
    }

    fn inner_slice_mut(&mut self) -> &mut [Self::Output] {
        &mut self.v_reg
    }
}

const STACK_LENGTH: usize = 0x10;

pub struct Stack {
    stack: [usize; STACK_LENGTH],
    sp: usize,
}

impl Stack {
    pub fn new() -> Self {
        Stack {
            stack: [0; STACK_LENGTH],
            sp: 0,
        }
    }

    pub fn push(&mut self, val: usize) -> Result<(), EmuErr> {
        if self.sp >= STACK_LENGTH {
            return Err(EmuErr::BadPush { sp: self.sp });
        }
        self.stack[self.sp] = val;
        self.sp += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Result<usize, EmuErr> {
        if self.sp == 0 {
            return Err(EmuErr::BadPop { sp: self.sp });
        }
        self.sp -= 1;
        let val = self.stack[self.sp];
        Ok(val)
    }
}
