use super::*;
use bit_vec::*;
use rand::{thread_rng, Rng};

pub const DISPLAY_WIDTH: usize = 0x80;
pub const DISPLAY_HEIGHT: usize = 0x40;
const PG_START: usize = 0x200;
const MEM_SIZE: usize = 0x1000;
const FONT_DATA: [u8; 0xF0] = [
    //CHIP8 fonts
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
    //SCHIP fonts.
    0xFF, 0xFF, 0xC3, 0xC3, 0xC3, 0xC3, 0xC3, 0xC3, 0xFF, 0xFF, // 0
    0x0C, 0x0C, 0x3C, 0x3C, 0x0C, 0x0C, 0x0C, 0x0C, 0x3F, 0x3F, // 1
    0xFF, 0xFF, 0x03, 0x03, 0xFF, 0xFF, 0xC0, 0xC0, 0xFF, 0xFF, // 2
    0xFF, 0xFF, 0x07, 0x07, 0xFF, 0xFF, 0x07, 0x07, 0xFF, 0xFF, // 3
    0xC3, 0xC3, 0xC3, 0xC3, 0xFF, 0xFF, 0x03, 0x03, 0x03, 0x03, // 4
    0xFF, 0xFF, 0xC0, 0xC0, 0xFF, 0xFF, 0x03, 0x03, 0xFF, 0xFF, // 5
    0xFF, 0xFF, 0xC0, 0xC0, 0xFF, 0xFF, 0xC3, 0xC3, 0xFF, 0xFF, // 6
    0xFF, 0xFF, 0x03, 0x03, 0x0C, 0x0C, 0x30, 0x30, 0x30, 0x30, // 7
    0xFF, 0xFF, 0xC3, 0xC3, 0xFF, 0xFF, 0xC3, 0xC3, 0xFF, 0xFF, // 8
    0xFF, 0xFF, 0xC3, 0xC3, 0xFF, 0xFF, 0x03, 0x03, 0xFF, 0xFF, // 9
    0xFF, 0xFF, 0xC3, 0xC3, 0xFF, 0xFF, 0xC3, 0xC3, 0xC3, 0xC3, // A
    0xFC, 0xFC, 0xC3, 0xC3, 0xFC, 0xFC, 0xC3, 0xC3, 0xFC, 0xFC, // B
    0xFF, 0xFF, 0xC0, 0xC0, 0xC0, 0xC0, 0xC0, 0xC0, 0xFF, 0xFF, // C
    0xFC, 0xFC, 0xC3, 0xC3, 0xC3, 0xC3, 0xC3, 0xC3, 0xFC, 0xFC, // D
    0xFF, 0xFF, 0xC0, 0xC0, 0xFF, 0xFF, 0xC0, 0xC0, 0xFF, 0xFF, // E
    0xFF, 0xFF, 0xC0, 0xC0, 0xFF, 0xFF, 0xC0, 0xC0, 0xC0, 0xC0, // F
];

struct Quirks {
    vf_reset_quirk: bool,
    jumping_quirk: bool,
}

pub struct Chip8Builder<'a> {
    program: Option<&'a [u8]>,
    quirks: Quirks,
}

impl<'a> Chip8Builder<'a> {
    pub fn new() -> Self {
        Self {
            program: None,
            quirks: Quirks {
                vf_reset_quirk: false,
                jumping_quirk: false,
            },
        }
    }

    pub fn with_program(mut self, program: &'a [u8]) -> Self {
        self.program = Some(program);
        self
    }

    pub fn with_vf_reset_quirk(mut self) -> Self {
        self.quirks.vf_reset_quirk = true;
        self
    }

    pub fn with_jumping_quirk(mut self) -> Self {
        self.quirks.jumping_quirk = true;
        self
    }

    pub fn build(self) -> Result<Chip8, EmuErr> {
        let program = self
            .program
            .expect("Program must be loaded to build emulator");
        Chip8::new(self.quirks, program)
    }
}

pub struct Chip8 {
    quirks: Quirks,
    memory: [u8; 0x1000],
    stack: Stack,
    v_reg: [u8; 0x10],
    i_reg: u16,
    delay_reg: u8,
    _sound_reg: u8,
    pc: u16,
    pressed_keys: [bool; 0x10],
    display_buffer: Box<[BitVec]>,
    high_res: bool,
}

impl Chip8 {
    //Uses slice of bytes as program data.
    fn new(quirks: Quirks, program: &[u8]) -> Result<Self, EmuErr> {
        let pg_len = program.len();
        let max_len = MEM_SIZE - PG_START;
        if pg_len > max_len {
            return Err(EmuErr::ProgramLength { pg_len, max_len });
        }

        //Program memory.
        let mut memory = [0u8; MEM_SIZE];

        //Loads fonts into memory.
        for (m_byte, &f_byte) in memory.iter_mut().zip(FONT_DATA.iter()) {
            *m_byte = f_byte;
        }

        //Slice of memory that will hold the program, typically starting at 0x200.
        let mem_pg_slice = &mut memory[PG_START..];

        //Loads program into memory.
        for (m_byte, &p_byte) in mem_pg_slice.iter_mut().zip(program.iter()) {
            *m_byte = p_byte;
        }

        Ok(Self {
            quirks,
            memory,
            stack: Stack::new(),
            v_reg: [0; 0x10],
            i_reg: 0,
            delay_reg: 0x0,
            _sound_reg: 0xFF,
            pc: PG_START as u16,
            pressed_keys: [false; 0x10],
            display_buffer: vec![BitVec::from_elem(DISPLAY_HEIGHT, false); DISPLAY_WIDTH]
                .into_boxed_slice(),
            high_res: false,
        })
    }

    pub fn execute_next(&mut self) -> Result<(), EmuErr> {
        if self.pc as usize >= self.memory.len() - 1 {
            return Err(EmuErr::PcOutOfBounds { pc: self.pc });
        }

        //Merges 2 byte opcode into instruction.
        let high_byte = (self.memory[self.pc as usize] as u16) << 8;
        let low_byte = self.memory[self.pc as usize + 1] as u16;
        let raw_instruction = high_byte | low_byte;
        let instruction = Instruction::new(raw_instruction);

        //PC incremented before execution as jump instructions modify PC.
        self.pc += 2;
        self.decode_and_execute(instruction)?;
        Ok(())
    }

    fn check_ireg_offset(&self, offset: u16) -> Result<(), EmuErr> {
        if (self.i_reg + offset) as usize > MEM_SIZE {
            return Err(EmuErr::IregOverflow {
                ireg: self.i_reg,
                offset,
            });
        }
        Ok(())
    }

    fn decode_and_execute(&mut self, instruction: Instruction) -> Result<(), EmuErr> {
        let x_reg_ref = &mut self.v_reg[instruction.x()];
        let kk = instruction.kk();
        match instruction.high_nibble() {
            0x0 => match kk {
                0xE0 => self.clear_display(),
                0xEE => self.pc = self.stack.pop()?,
                0xFE => self.high_res = false,
                0xFF => self.high_res = true,
                _ => {
                    return Err(EmuErr::BadInstruction {
                        pc: self.pc,
                        instruction,
                    });
                }
            },
            0x1 => self.pc = instruction.nnn(),
            0x2 => {
                self.stack.push(self.pc)?;
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
                if *x_reg_ref == self.v_reg[instruction.y()] {
                    self.pc += 2;
                }
            }
            0x6 => *x_reg_ref = instruction.kk(),
            0x7 => *x_reg_ref = x_reg_ref.wrapping_add(instruction.kk()),
            0x8 => return self.instruction_0x8(instruction),
            0x9 => {
                if *x_reg_ref != self.v_reg[instruction.y()] {
                    self.pc += 2;
                }
            }
            0xA => self.i_reg = instruction.nnn(),
            0xB => self.pc = instruction.nnn() + *x_reg_ref as u16,
            0xC => *x_reg_ref = thread_rng().gen_range(0u8..=255u8) % instruction.kk(),
            0xD => self.draw(instruction)?,
            0xE => {
                let key_pressed = self.pressed_keys[*x_reg_ref as usize];
                match kk {
                    0x9E => {
                        if key_pressed {
                            self.pc += 2;
                        }
                    }
                    0xA1 => {
                        if !key_pressed {
                            self.pc += 2;
                        }
                    }
                    _ => {
                        return Err(EmuErr::BadInstruction {
                            pc: self.pc,
                            instruction,
                        });
                    }
                }
            }
            0xF => return self.instruction_0xf(instruction),
            _ => {}
        }
        Ok(())
    }

    #[inline]
    fn instruction_0x8(&mut self, instruction: Instruction) -> Result<(), EmuErr> {
        let y_reg_val = self.v_reg[instruction.y()];
        let x_reg_ref = &mut self.v_reg[instruction.x()];
        match instruction.low_nibble() {
            0x0 => {
                *x_reg_ref = y_reg_val;
            }
            0x1 => {
                *x_reg_ref |= y_reg_val;
                if self.quirks.vf_reset_quirk {
                    self.v_reg[0xF] = 0;
                }
            }
            0x2 => {
                *x_reg_ref &= y_reg_val;
                if self.quirks.vf_reset_quirk {
                    self.v_reg[0xF] = 0;
                }
            }
            0x3 => {
                *x_reg_ref ^= y_reg_val;
                if self.quirks.vf_reset_quirk {
                    self.v_reg[0xF] = 0;
                }
            }
            0x4 => {
                let did_wrap = x_reg_ref.checked_add(y_reg_val).is_none();
                *x_reg_ref = x_reg_ref.wrapping_add(y_reg_val);
                self.v_reg[0xF] = did_wrap as u8;
            }
            0x5 => {
                let didnt_wrap = x_reg_ref.checked_sub(y_reg_val).is_some();
                *x_reg_ref = x_reg_ref.wrapping_sub(y_reg_val);
                self.v_reg[0xF] = didnt_wrap as u8;
            }
            0x6 => {
                let lsb = *x_reg_ref & 0x1;
                *x_reg_ref >>= 1;
                self.v_reg[0xF] = lsb;
            }
            0x7 => {
                let didnt_wrap = y_reg_val.checked_sub(*x_reg_ref).is_some();
                *x_reg_ref = y_reg_val.wrapping_sub(*x_reg_ref);
                self.v_reg[0xF] = didnt_wrap as u8;
            }
            0xE => {
                let hsb = *x_reg_ref >> 0x7;
                *x_reg_ref <<= 1;
                self.v_reg[0xF] = hsb;
            }
            _ => {
                return Err(EmuErr::BadInstruction {
                    pc: self.pc,
                    instruction,
                })
            }
        }
        Ok(())
    }

    #[inline]
    fn instruction_0xf(&mut self, instruction: Instruction) -> Result<(), EmuErr> {
        let x_reg_val = self.v_reg[instruction.x()];
        match instruction.kk() {
            0x7 => {
                self.v_reg[instruction.x()] = self.delay_reg;
            }
            0xA => {
                if !self.pressed_keys.contains(&true) {
                    self.pc -= 2;
                }
            }
            0x15 => self.delay_reg = x_reg_val,
            0x18 => {
                //Instruction for sound delay.
            }
            0x1E => {
                let x_reg_val = x_reg_val as u16;
                self.check_ireg_offset(x_reg_val)?;
                self.i_reg += x_reg_val;
            }
            0x29 => self.i_reg = (x_reg_val * 5) as u16,
            0x30 => self.i_reg = (0x50 + x_reg_val * 10) as u16,
            0x33 => {
                self.check_ireg_offset(2)?;
                let bcd = u8_to_bcd_array(x_reg_val);
                for (m, d) in self.memory[self.i_reg as usize..].iter_mut().zip(bcd) {
                    *m = d;
                }
            }
            0x55 => {
                let v_reg_slice = &self.v_reg[..=instruction.x()];
                let mem_slice = &mut self.memory[self.i_reg as usize..];
                for (m, &v) in mem_slice.iter_mut().zip(v_reg_slice.iter()) {
                    *m = v;
                }
                if self.quirks.jumping_quirk {
                    self.i_reg += 1;
                }
            }
            0x65 => {
                let v_reg_slice = &mut self.v_reg[..=instruction.x()];
                let mem_slice = &self.memory[self.i_reg as usize..];
                for (v, &m) in v_reg_slice.iter_mut().zip(mem_slice.iter()) {
                    *v = m;
                }
                if self.quirks.jumping_quirk {
                    self.i_reg += 1;
                }
            }
            _ => {
                return Err(EmuErr::BadInstruction {
                    pc: self.pc,
                    instruction,
                })
            }
        }
        Ok(())
    }

    fn draw(&mut self, instruction: Instruction) -> Result<(), EmuErr> {
        let v_reg_x = self.v_reg[instruction.x()] as usize;
        let v_reg_y = self.v_reg[instruction.y()] as usize;
        let i_reg = self.i_reg as usize;
        let mut vf_new = 0x0;
        //Slice of memory that will be used to draw from.
        let high_res_sprite = self.high_res && instruction.low_nibble() == 0;
        let mem_slice = if high_res_sprite {
            self.check_ireg_offset(0x20)?;
            &self.memory[i_reg..][..0x20]
        } else {
            self.check_ireg_offset(instruction.low_nibble() as u16)?;
            &self.memory[i_reg..][..instruction.low_nibble() as usize]
        };
        let (mut x_wrap, mut y_wrap) = (DISPLAY_WIDTH, DISPLAY_HEIGHT);
        if !self.high_res {
            x_wrap /= 2;
            y_wrap /= 2;
        }
        for (i, byte) in mem_slice.iter().enumerate() {
            let y_offset = if high_res_sprite { i / 2 } else { i };
            let y_coord = (v_reg_y + y_offset) % y_wrap;
            for j in 0usize..8usize {
                let x_offset = if high_res_sprite { j + 8 * (i % 2) } else { j };
                let x_coord = (v_reg_x + x_offset) % x_wrap;
                let curr_bit = (byte >> (7 - j)) & 0x1 == 0x1;

                //continue if the current sprite bit is 0.
                if !curr_bit {
                    continue;
                }

                let pixel = self.display_buffer[x_coord][y_coord];

                //VF flag set true if there is a pixel collision (1 XOR 1).
                if pixel {
                    vf_new = 0x1;
                }

                //Negate pixel as pixel is XORed onto display
                self.display_buffer[x_coord].set(y_coord, !pixel);
            }
        }
        self.v_reg[0xF] = vf_new;
        Ok(())
    }

    fn clear_display(&mut self) {
        for col in self.display_buffer.iter_mut() {
            col.clear();
        }
    }

    pub fn get_display_buffer(&self) -> &[BitVec] {
        &self.display_buffer
    }

    pub fn set_key(&mut self, key: usize) {
        self.pressed_keys[key] = true;
    }

    pub fn unset_key(&mut self, key: usize) {
        self.pressed_keys[key] = false;
    }

    pub fn dec_delay_reg(&mut self) {
        if self.delay_reg > 0 {
            self.delay_reg -= 1;
        }
    }

    pub fn is_high_res(&self) -> bool {
        self.high_res
    }
}

fn u8_to_bcd_array(num: u8) -> [u8; 3] {
    let mut remainder = num;
    let mut output = [0u8; 3];
    for i in (0..3).rev() {
        let dec_pow = 10u8.pow(i);
        let digit = remainder / dec_pow;
        output[2 - i as usize] = digit;
        remainder -= digit * dec_pow;
    }
    output
}
