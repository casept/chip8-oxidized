pub const SCHIP8_SCREEN_WIDTH: usize = 64;
pub const SCHIP8_SCREEN_HEIGHT: usize = 32;
pub const NUM_PIXELS: usize = SCHIP8_SCREEN_WIDTH * SCHIP8_SCREEN_HEIGHT;

const FONT: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10, 0xF0, 0x80, 0xF0, 0xF0,
    0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0, 0x10, 0xF0, 0xF0, 0x80,
    0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90, 0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0,
    0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0, 0x90, 0xE0, 0x90, 0xE0, 0xF0, 0x80, 0x80, 0x80,
    0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0, 0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80,
];

pub struct SChip8 {
    pub key_pad: [bool; 16],
    pub dt: u8, // Delay timer
    pub st: u8, // Sound timer
    pub screen: [u8; NUM_PIXELS],
    ram: [u8; 0x1000],
    pc: usize,
    ar: u16, // Address register
    v: [u16; 16],
    stack: [usize; 48], // Stack implemented as empty ascending
    sp: usize,
}

impl SChip8 {
    pub fn new(program: Vec<u8>) -> SChip8 {
        let mut schip8 = SChip8 {
            ram: [0; 0x1000],
            pc: 512,
            ar: 0,
            v: [0; 16],
            dt: 0,
            st: 0,
            stack: [0; 48],
            sp: 0,
            key_pad: [false; 16],
            screen: [0; NUM_PIXELS],
        };

        let (reserved, ram) = schip8.ram.split_at_mut(512);
        assert!(reserved.len() == 512);
        let (ram_l, _ram_r) = ram.split_at_mut(program.len());
        ram_l.copy_from_slice(program.as_slice());

        // Insert font data
        let (font_area, _ram) = schip8.ram.split_at_mut(80);
        assert!(font_area.len() == 80);
        font_area.copy_from_slice(&FONT);

        return schip8;
    }

    pub fn run(&mut self, key: usize, redraw: &mut bool) -> bool {
        #[cfg(debug_assertions)]
        {
            println!("----- CHIP8 Oxidized Interactive Debugger -----");
        }

        let first_half: u8 = self.ram[self.pc];
        let second_half: u8 = self.ram[self.pc + 1];
        let instruction: [u8; 4] = [
            (first_half & 0xF0) >> 4,
            first_half & 0xF,
            (second_half & 0xF0) >> 4,
            second_half & 0xF,
        ];
        #[cfg(debug_assertions)]
        {
            println!(
                "Opcode: {:01X}{:01X}{:01X}{:01X}",
                instruction[0], instruction[1], instruction[2], instruction[3]
            );
        }

        match instruction {
            // 00CN - Scroll display N lines down
            // 00E0 - Clears the screen.
            [0x0, 0x0, 0xE, 0x0] => {
                self.screen = [0; NUM_PIXELS];
                *redraw = true;
            }
            // 00EE - Returns from a subroutine.
            [0x0, 0x0, 0xE, 0xE] => {
                self.sp -= 1;
                self.pc = self.stack[self.sp];
            }
            // 00FB - Scroll display 4 pixels right
            [0x0, 0x0, 0xF, 0xB] => {
                unimplemented!();
            }
            // 00FC - Scroll display 4 pixels left
            [0x0, 0x0, 0xF, 0xC] => {
                unimplemented!();
            }
            // 00FD - Exit CHIP interpreter
            [0x0, 0x0, 0xF, 0xD] => {
                return false;
            }
            // 00FE - Disable extended screen mode
            [0x0, 0x0, 0xF, 0xE] => {
                unimplemented!();
            }
            // 00FF - Enable extended screen mode for full-screen graphics
            [0x0, 0x0, 0xF, 0xF] => {
                unimplemented!();
            }
            // 0NNN - Calls RCA 1802 program at address NNN. Not necessary for most ROMs.
            // See issue.
            [0x0, _, _, _] => {
                unimplemented!();
            }
            // 1NNN - Jumps to address NNN.
            [0x1, a, b, c] => {
                let addr = (((a as u16) << 8) | ((b as u16) << 4) | (c as u16)) as usize;
                self.pc = addr - 2;
            }
            // 2NNN - Calls subroutine at NNN.
            [0x2, a, b, c] => {
                let addr = (((a as u16) << 8) | ((b as u16) << 4) | (c as u16)) as usize;
                self.stack[self.sp] = self.pc;
                self.sp += 1;
                self.pc = addr - 2;
            }
            // 3XNN - Skips the next instruction if VX equals NN. (Usually the next instruction is a jump to skip a code block)
            [0x3, x, b, c] => {
                let nn = ((b << 4) | c) as u16;
                if self.v[x as usize] == nn {
                    self.pc += 2;
                }
            }
            // 4XNN - Skips the next instruction if VX doesn't equal NN. (Usually the next instruction is a jump to skip a code block)
            [0x4, x, b, c] => {
                let nn = ((b << 4) | c) as u16;
                if self.v[x as usize] != nn {
                    self.pc += 2;
                }
            }
            // 5XNN - Skips the next instruction if VX equals VY. (Usually the next instruction is a jump to skip a code block)
            [0x5, x, y, 0] => {
                if self.v[x as usize] == self.v[y as usize] {
                    self.pc += 2;
                }
            }
            // 6XNN - Sets VX to NN.
            [0x6, x, b, c] => {
                let nn = ((b << 4) | c) as u16;
                self.v[x as usize] = nn;
            }
            // 7XNN - Adds NN to VX. (Carry flag is not changed)
            [0x7, x, b, c] => {
                let nn = ((b << 4) | c) as u16;
                let sum = self.v[x as usize] + nn;
                self.v[x as usize] = (sum & 0xFF) as u16;
            }
            // 8XY0 - Sets VX to the value of VY.
            [0x8, x, y, 0x0] => {
                self.v[x as usize] = self.v[y as usize];
            }
            // 8XY1 - Sets VX to VX or VY. (Bitwise OR operation)
            [0x8, x, y, 0x1] => {
                self.v[x as usize] |= self.v[y as usize];
            }
            // 8XY2 - Sets VX to VX and VY. (Bitwise AND operation)
            [0x8, x, y, 0x2] => {
                self.v[x as usize] &= self.v[y as usize];
            }
            // 8XY3 - Sets VX to VX xor VY.
            [0x8, x, y, 0x3] => {
                self.v[x as usize] ^= self.v[y as usize];
            }
            // 8XY4 - Adds VY to VX. VF is set to 1 when there's a carry, and to 0 when there isn't.
            [0x8, x, y, 0x4] => {
                let sum = self.v[x as usize] + self.v[y as usize];
                if sum < 0x100 {
                    self.v[0xF] = 0;
                } else {
                    self.v[0xF] = 1;
                }
                self.v[x as usize] = sum & 0xFF;
            }
            // 8XY5 - VY is subtracted from VX. VF is set to 0 when there's a borrow, and 1 when there isn't.
            [0x8, x, y, 0x5] => {
                let mut diff = (self.v[x as usize] as i32) - (self.v[y as usize] as i32);
                if diff >= 0 {
                    self.v[0xF] = 1;
                } else {
                    self.v[0xF] = 0;
                    diff = -diff;
                }
                self.v[x as usize] = diff as u16;
            }
            // 8XY6 - Stores the least significant bit of VX in VF and then shifts VX to the right by 1.
            [0x8, x, _y, 0x6] => {
                self.v[0xF] = self.v[x as usize] & 0x1;
                self.v[x as usize] >>= 1;
            }
            // 8XY7 - Sets VX to VY minus VX. VF is set to 0 when there's a borrow, and 1 when there isn't.
            [0x8, x, y, 0x7] => {
                let mut diff = (self.v[y as usize] as i32) - (self.v[x as usize] as i32);
                if diff >= 0 {
                    self.v[0xF] = 1;
                } else {
                    self.v[0xF] = 0;
                    diff = -diff;
                }
                self.v[x as usize] = diff as u16;
            }
            // 8XYE - Stores the most significant bit of VX in VF and then shifts VX to the left by 1.
            [0x8, x, _y, 0xE] => {
                self.v[0xF] = (self.v[x as usize] & 0x8000) >> 15;
                self.v[x as usize] <<= 1;
            }
            // 9XY0 - Skips the next instruction if VX doesn't equal VY.
            [0x9, x, y, 0x0] => {
                if self.v[x as usize] != self.v[y as usize] {
                    self.pc += 2;
                }
            }
            // ANNN - Sets I to the address NNN.
            [0xA, a, b, c] => {
                let addr = ((a as u16) << 8) | ((b as u16) << 4) | (c as u16);
                self.ar = addr;
            }
            // BNNN - Jumps to the address NNN plus V0.
            [0xB, a, b, c] => {
                let mut addr = ((a as usize) << 8) | ((b as usize) << 4) | (c as usize);
                addr += self.v[0] as usize;
                self.pc = addr - 2;
            }
            // CXNN - Sets VX to the result of a bitwise and operation on a random number (Typically: 0 to 255) and NN.
            [0xC, x, b, c] => {
                let nn = ((b << 4) | c) as u16;
                let rand = rand::random::<u8>() as u16;
                self.v[x as usize] = rand & nn;
            }
            // DXYN - Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels and a height of N pixels. Each row of 8 pixels is read as bit-coded starting from memory location I; I value doesn’t change after the execution of this instruction. As described above, VF is set to 1 if any screen pixels are flipped from set to unset when the sprite is drawn, and to 0 if that doesn’t happen
            // - Coordinate (VX, VY)                            - Check
            // - 8x{1-F} sprite, starts at I                    - Check
            // - Each row bit coded                             - Check
            // - I does not change                              - Check
            // - Flip from set to unset => VF=1, otherwise VF=0 - Check
            // For SCHIP8: Show N-byte sprite from M(I) at coords (VX,VY), VF := collision. If N=0 and extended mode, show 16x16 sprite.
            [0xD, x, y, c] => {
                let pixels = &mut self.screen;
                self.v[0xF] = 0;
                let mut ar = self.ar as usize;
                let x = (self.v[x as usize] as usize) % SCHIP8_SCREEN_WIDTH;
                let mut yi = (self.v[y as usize] as usize) % SCHIP8_SCREEN_HEIGHT;
                let ye = yi + (c as usize);
                // Iterate over the rows
                while yi < ye {
                    if yi >= SCHIP8_SCREEN_HEIGHT {
                        break;
                    }
                    // Extract each bit from sprite
                    let sprite_data = self.ram[ar];
                    let sprite_row = [
                        (sprite_data & 0x80) >> 7,
                        (sprite_data & 0x40) >> 6,
                        (sprite_data & 0x20) >> 5,
                        (sprite_data & 0x10) >> 4,
                        (sprite_data & 0x08) >> 3,
                        (sprite_data & 0x04) >> 2,
                        (sprite_data & 0x02) >> 1,
                        sprite_data & 0x01,
                    ];

                    // Get the current row in the screen buffer
                    let pixel_row_coord = yi * SCHIP8_SCREEN_WIDTH + x;
                    let pixel_row_left = pixel_row_coord;
                    let mut pixel_row_right = pixel_row_coord + 8;
                    if pixel_row_left >= NUM_PIXELS {
                        break;
                    }
                    if pixel_row_right > NUM_PIXELS {
                        pixel_row_right = NUM_PIXELS;
                    }
                    let pixel_row = &mut pixels[pixel_row_left..pixel_row_right];

                    // Iterate over sprite pixels
                    let mut xi = 0;
                    for sprite_pixel in sprite_row.iter() {
                        let pixel = pixel_row[xi];
                        // Collision detection
                        if pixel == 1 && *sprite_pixel == 1 {
                            self.v[0xF] = 1;
                        }
                        // XOR the pixels from the screen buffer and the sprite
                        let result = pixel ^ *sprite_pixel;
                        pixel_row[xi] = result;
                        xi += 1;
                        if xi >= pixel_row.len() {
                            break;
                        }
                    }
                    ar += 1;
                    yi += 1;
                }
                *redraw = true;
            }
            // EX9E - Skips the next instruction if the key stored in VX is pressed. (Usually the next instruction is a jump to skip a code block)
            [0xE, x, 0x9, 0xE] => {
                let vx = self.v[x as usize];
                let keyp = self.key_pad[vx as usize];
                if keyp {
                    self.pc += 2;
                }
            }
            // EXA1 - Skips the next instruction if the key stored in VX isn't pressed. (Usually the next instruction is a jump to skip a code block)
            [0xE, x, 0xA, 0x1] => {
                let vx = self.v[x as usize];
                let keyp = self.key_pad[vx as usize];
                if !keyp {
                    self.pc += 2;
                }
            }
            // FX07 - Sets VX to the value of the delay timer.
            [0xF, x, 0x0, 0x7] => {
                self.v[x as usize] = self.dt as u16;
            }
            // FX0A - A key press is awaited, and then stored in VX. (Blocking Operation. All instruction halted until next key event)
            [0xF, x, 0x0, 0xA] => {
                // TODO:
                if key < 16 {
                    self.v[x as usize] = key as u16;
                } else {
                    self.pc -= 2;
                }
            }
            // FX15 - Sets the delay timer to VX.
            [0xF, x, 0x1, 0x5] => {
                self.dt = self.v[x as usize] as u8;
            }
            // FX18 - Sets the sound timer to VX.
            [0xF, x, 0x1, 0x8] => {
                // TODO:
                self.st = self.v[x as usize] as u8;
            }
            // FX1E - Adds VX to I. VF is set to 1 when there is a range overflow (I+VX>0xFFF), and to 0 when there isn't.
            [0xF, x, 0x1, 0xE] => {
                self.ar += self.v[x as usize];
                if self.ar > 0xFFF {
                    self.v[0xF] = 1;
                    self.ar &= 0xFFF;
                } else {
                    self.v[0xF] = 0;
                }
            }
            // FX29 - Sets I to the location of the sprite for the character in VX. Characters 0-F (in hexadecimal) are represented by a 4x5 font.
            [0xF, x, 0x2, 0x9] => {
                self.ar = self.v[x as usize] * 5;
            }
            // FX33 - Stores the binary-coded decimal representation of VX, with the most significant of three digits at the address in I, the middle digit at I plus 1, and the least significant digit at I plus 2. (In other words, take the decimal representation of VX, place the hundreds digit in memory at location in I, the tens digit at location I+1, and the ones digit at location I+2.)
            [0xF, x, 0x3, 0x3] => {
                let ar = self.ar as usize;
                let vx = self.v[x as usize];
                self.ram[ar] = ((vx - (vx % 100)) / 100) as u8;
                self.ram[(ar + 1)] = ((vx - vx % 10) / 10) as u8;
                self.ram[(ar + 2)] = (vx % 10) as u8;
            }
            // FX55 - Stores V0 to VX (including VX) in memory starting at address I. The offset from I is increased by 1 for each value written, but I itself is left unmodified.
            [0xF, x, 0x5, 0x5] => {
                let ar = self.ar as usize;
                let mut xi = 0;
                while xi <= (x as usize) {
                    self.ram[ar + xi] = self.v[xi] as u8;
                    xi += 1;
                }
            }
            // FX65 - Fills V0 to VX (including VX) with values from memory starting at address I. The offset from I is increased by 1 for each value written, but I itself is left unmodified.
            [0xF, x, 0x6, 0x5] => {
                let ar = self.ar as usize;
                let mut xi = 0;
                while xi <= (x as usize) {
                    self.v[xi] = self.ram[ar + xi] as u16;
                    xi += 1;
                }
            }
            // FX75 - Store V0..VX in RPL user flags (X <= 7)
            [0xF, x, 0x7, 0x5] => {
                unimplemented!();
            }
            // FX85 - Read V0..VX from RPL user flags (X <= 7)
            [0xF, x, 0x8, 0x5] => {
                unimplemented!();
            }
            [_, _, _, _] => {
                panic!("Unknown instruction!");
            }
        }

        self.pc += 2;

        #[cfg(debug_assertions)]
        {
            loop {
                use std::io::Write;
                print!("> ");
                std::io::stdout().flush().unwrap();
                let mut line = String::new();
                std::io::stdin().read_line(&mut line).unwrap();
                match line.trim() {
                    "reg" => {
                        println!("pc: {:X}", self.pc);
                        println!("ar: {:X}", self.ar);
                        println!("sp: {:X}", self.sp);
                        println!("dt: {:X}", self.dt);
                        println!("st: {:X}", self.st);
                        for (i, reg) in self.v.iter().enumerate() {
                            println!("V{:X}: {:X}", i, reg);
                        }
                    }
                    "stack" => {
                        for (i, elem) in self.stack.iter().rev().enumerate() {
                            println!("{:02X}: {:03X}", (0x2F - i), elem);
                        }
                    }
                    "ram" => {
                        for (i, elem) in self.ram.iter().rev().enumerate() {
                            println!("{:03X}: {:02X}", (0xFFF - i), elem);
                        }
                    }
                    "disp" => {
                        for (i, pixel) in self.screen.iter().enumerate() {
                            if (i > 0) && (i % SCHIP8_SCREEN_WIDTH == 0) {
                                println!("");
                            }
                            print!("{}", pixel);
                        }
                        println!("");
                    }
                    "h" => {
                        println!("Available commands: reg, stack, ram, disp, h, c, q");
                    }
                    "c" | "" => {
                        break;
                    }
                    "q" => {
                        return false;
                    }
                    _ => {
                        println!("Unknown command");
                    }
                }
            }
        }

        return true;
    }
}