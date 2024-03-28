use super::*;
use bit_vec::*;
use rand::{thread_rng, Rng};

macro_rules! bad_instruction {
    ($instruction:expr) => {
        Err(format!(
            "Bad instruction: 0x{:4X}",
            $instruction.get_instruction()
        ))
    };
}

const MEM_SIZE: usize = 0xFFF;
const PG_START: usize = 0x200;
pub const DISPLAY_WIDTH: usize = 0x40;
pub const DISPLAY_HEIGHT: usize = 0x20;
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

type ExecRes = Result<(), String>;

pub struct Processor {
    pub mem: [u8; MEM_SIZE],
    stack: [u16; 0x10],
    stack_pointer: u8,
    v_reg: [u8; 0x10],
    i_reg: u16,
    delay_reg: u8,
    sound_reg: u8,
    pc: u16,
    pressed_keys: [bool; 0x10],
    display_buffer: Box<[BitVec]>,
}

impl Processor {
    //Uses slice of bytes as program data.
    pub fn new(program: &[u8]) -> Result<Self, String> {
        let pg_size = program.len();

        //Checks if program fits into memory.
        let mut mem = [0u8; MEM_SIZE];

        //Loads number sprites into memory, from 0x0 to 0x50.
        for (m_byte, f_byte) in mem.iter_mut().zip(FONT_DATA.iter()) {
            *m_byte = *f_byte;
        }

        //Slize of memory that will hold the program, typically starting at 0x200.
        let mem_pg_slice = &mut mem[PG_START..];

        //Inserts program into mem slice.
        for (m_byte, p_byte) in mem_pg_slice.iter_mut().zip(program.iter()) {
            *m_byte = *p_byte;
        }

        Ok(Self {
            mem,
            stack: [0; 0x10],
            stack_pointer: 0,
            v_reg: [0; 0x10],
            i_reg: 0,
            delay_reg: 0xFF,
            sound_reg: 0xFF,
            pc: PG_START as u16,
            pressed_keys: [false; 0x10],
            display_buffer: vec![BitVec::from_elem(DISPLAY_HEIGHT, false); DISPLAY_WIDTH]
                .into_boxed_slice(),
        })
    }

    fn mem_byte(&mut self, index: u16) -> &mut u8 {
        &mut self.mem[index as usize]
    }

    fn get_vreg(&mut self, index: u8) -> &mut u8 {
        &mut self.v_reg[index as usize]
    }

    pub fn execute_next(&mut self) -> ExecRes {
        //Merges 2 byte opcode into u16
        let high_byte = (*self.mem_byte(self.pc) as u16) << 8;
        let low_byte = *self.mem_byte(self.pc + 1) as u16;
        let raw_instruction = high_byte | low_byte;
        let instruction = Instruction::new(raw_instruction);

        //PC incremented before exxecution as jump instructions modify PC.
        self.pc += 2;
        self.decode_and_execute(instruction)?;
        Ok(())
    }

    pub fn decode_and_execute(&mut self, instruction: Instruction) -> ExecRes {
        let x_reg_ref = self.get_vreg(instruction.x());
        let kk = instruction.kk();
        match instruction.high_nibble() {
            0x0 => {
                if kk == 0xE0 {
                    self.clear_display();
                } else if kk == 0xEE {
                    self.stack_pointer -= 1;
                    self.pc = self.stack[self.stack_pointer as usize]
                } else {
                    return bad_instruction!(instruction);
                }
            }
            0x1 => self.pc = instruction.nnn(),
            0x2 => {
                self.stack[self.stack_pointer as usize] = self.pc;
                self.stack_pointer += 1;
                self.pc = instruction.nnn();
            }
            0x3 => {
                if *x_reg_ref == kk {
                    self.pc += 2;
                }
            }
            0x4 => {
                if *x_reg_ref != kk {
                    self.pc += 2;
                }
            }
            0x5 => {
                if *x_reg_ref == *self.get_vreg(instruction.y()) {
                    self.pc += 2;
                }
            }
            0x6 => *x_reg_ref = instruction.kk(),
            0x7 => *x_reg_ref = x_reg_ref.wrapping_add(instruction.kk()),
            0x8 => return self.instruction_0x8(instruction),
            0x9 => {
                if *x_reg_ref != *self.get_vreg(instruction.y()) {
                    self.pc += 2;
                }
            }
            0xA => self.i_reg = instruction.nnn(),
            0xB => self.pc = instruction.nnn() + *x_reg_ref as u16,
            0xC => *x_reg_ref = thread_rng().gen_range(0u8..=255u8) % instruction.kk(),
            0xD => self.draw(instruction),
            0xE => {
                let x_reg_val = *x_reg_ref;
                let key_pressed = self.pressed_keys[x_reg_val as usize];
                if kk == 0x9E {
                    if key_pressed {
                        self.pc += 2;
                    }
                } else if kk == 0xA1 {
                    if !key_pressed {
                        self.pc += 2;
                    }
                } else {
                    return bad_instruction!(instruction);
                }
            }
            0xF => return self.instruction_0xf(instruction),
            _ => return bad_instruction!(instruction),
        }
        Ok(())
    }

    #[inline]
    fn instruction_0x8(&mut self, instruction: Instruction) -> ExecRes {
        let y_reg_val = *self.get_vreg(instruction.y());
        let x_reg_ref = self.get_vreg(instruction.x());
        match instruction.low_nibble() {
            0x0 => {
                *x_reg_ref = y_reg_val;
            }
            0x1 => {
                *x_reg_ref |= y_reg_val;
                self.v_reg[0xF] = 0;
            }
            0x2 => {
                *x_reg_ref &= y_reg_val;
                self.v_reg[0xF] = 0;
            }
            0x3 => {
                *x_reg_ref ^= y_reg_val;
                self.v_reg[0xF] = 0;
            }
            0x4 => {
                let did_wrap = x_reg_ref.checked_add(y_reg_val).is_none();
                *x_reg_ref = x_reg_ref.wrapping_add(y_reg_val);
                self.v_reg[0xF] = did_wrap.to_u8();
            }
            0x5 => {
                let didnt_wrap = x_reg_ref.checked_sub(y_reg_val).is_some();
                *x_reg_ref = x_reg_ref.wrapping_sub(y_reg_val);
                self.v_reg[0xF] = didnt_wrap.to_u8();
            }
            0x6 => {
                let lsb = *x_reg_ref & 0x1;
                *x_reg_ref >>= 1;
                self.v_reg[0xF] = lsb;
            }
            0x7 => {
                let didnt_wrap = y_reg_val.checked_sub(*x_reg_ref).is_some();
                *x_reg_ref = y_reg_val.wrapping_sub(*x_reg_ref);
                self.v_reg[0xF] = didnt_wrap.to_u8();
            }
            0xE => {
                let hsb = *x_reg_ref >> 0x7;
                *x_reg_ref <<= 1;
                self.v_reg[0xF] = hsb;
            }
            _ => return bad_instruction!(instruction),
        }
        Ok(())
    }

    #[inline]
    fn instruction_0xf(&mut self, instruction: Instruction) -> ExecRes {
        let x_reg_val = *self.get_vreg(instruction.x());
        match instruction.kk() {
            0x7 => {
                *self.get_vreg(instruction.x()) = self.delay_reg;
            }
            0x0A => {
                if !self.pressed_keys.contains(&true) {
                    self.pc -= 2;
                }
            }
            0x15 => self.delay_reg = x_reg_val,
            0x1E => {
                self.v_reg[0xF] = self.i_reg.checked_add(x_reg_val as u16).is_none().to_u8();
                self.i_reg = self.i_reg.wrapping_add(x_reg_val as u16);
            }
            0x29 => self.i_reg = (x_reg_val * 5) as u16,
            0x55 => {
                let v_reg_slice = &self.v_reg[..=instruction.x() as usize];
                let mem_slice = &mut self.mem[self.i_reg as usize..];
                for (m, v) in mem_slice.iter_mut().zip(v_reg_slice.iter()) {
                    *m = *v;
                }
            }
            0x65 => {
                let v_reg_slice = &mut self.v_reg[..=instruction.x() as usize];
                let mem_slice = &self.mem[self.i_reg as usize..];
                for (v, m) in v_reg_slice.iter_mut().zip(mem_slice.iter()) {
                    *v = *m;
                }
            }
            _ => return bad_instruction!(instruction),
        }
        Ok(())
    }

    fn draw(&mut self, instruction: Instruction) {
        let v_reg_x = *self.get_vreg(instruction.x()) as usize;
        let v_reg_y = *self.get_vreg(instruction.y()) as usize;
        let i_reg = self.i_reg as usize;
        let mut vf_new = 0x0;
        //Slice of memory that will be used to draw from.
        let mem_slice = &self.mem[i_reg..][..instruction.low_nibble() as usize];
        for (i, byte) in mem_slice.iter().enumerate() {
            //Loops through each bit in byte and XORs bit onto display buffer
            for j in 0usize..8usize {
                //Coordinates mod size of axis as to not go out of bounds of the buffer.
                let x_coord = (v_reg_x + j) % DISPLAY_WIDTH;
                let y_coord = (v_reg_y + i) % DISPLAY_HEIGHT;
                let curr_bit = (byte >> (7 - j)) & 0x1 == 0x1;

                //Do nothing if the current bit is 0
                if !curr_bit {
                    continue;
                }

                let pixel = self.display_buffer[x_coord][y_coord];

                //VF flag set true if there is a pixel collision (1 XOR 1)
                if pixel {
                    vf_new = 0x1;
                }

                //Negate pixel as pixel is XORed onto display
                self.display_buffer[x_coord].set(y_coord, !pixel);
            }
        }
        self.v_reg[0xF] = vf_new;
    }

    fn clear_display(&mut self) {
        for col in self.display_buffer.iter_mut() {
            col.clear();
        }
    }

    pub fn get_display_buffer(&self) -> &[BitVec] {
        &*self.display_buffer
    }

    pub fn set_key(&mut self, key: usize, val: bool) {
        self.pressed_keys[key] = val;
    }

    pub fn dec_delay_reg(&mut self) {
        if self.delay_reg > 0 {
            self.delay_reg -= 1;
        }
    }
}

trait ToU8 {
    fn to_u8(&self) -> u8;
}

impl ToU8 for bool {
    fn to_u8(&self) -> u8 {
        if *self {
            1u8
        } else {
            0u8
        }
    }
}
