use rand::Rng;

type U12 = u16;

const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;
const ZOOM: usize = 10;

struct CPUState
{
	i : U12,
    pc : U12,
    sp : u8,
	delay_timer : u8,
	sound_timer : u8,
    v : [u8; 16],
	memory : [u8; 4096],
    stack : [u16; 16],

    screen : [bool; SCREEN_WIDTH * SCREEN_HEIGHT],

	rng : rand::rngs::ThreadRng
}

impl CPUState
{
	const HEXA_SPRITE_SIZE: usize = 5;
	const HEXA_SPRITES: [u8; CPUState::HEXA_SPRITE_SIZE * 16] = [
		0xF0, 0x90, 0x90, 0x90, 0xF0,	// 0
		0x20, 0x60, 0x20, 0x20, 0x70,	// 1
		0xF0, 0x10, 0xF0, 0x80, 0xF0,	// 2
		0xF0, 0x10, 0xF0, 0x10, 0xF0,	// 3
		0x90, 0x90, 0xF0, 0x10, 0x10,   // 4
		0xF0, 0x80, 0xF0, 0x10, 0xF0,	// 5
		0xF0, 0x80, 0xF0, 0x90, 0xF0,	// 6
		0xF0, 0x10, 0x20, 0x40, 0x40,	// 7
		0xF0, 0x90, 0xF0, 0x90, 0xF0,	// 8
		0xF0, 0x90, 0xF0, 0x10, 0xF0,	// 9
		0xF0, 0x90, 0xF0, 0x90, 0x90,	// A
		0xE0, 0x90, 0xE0, 0x90, 0xE0,	// B
		0xF0, 0x80, 0x80, 0x80, 0xF0,	// C
		0xE0, 0x90, 0x90, 0x90, 0xE0,	// D
		0xF0, 0x80, 0xF0, 0x80, 0xF0,	// E
		0xF0, 0x80, 0xF0, 0x80, 0x80	// F
	];
	fn new() -> Self {
		let mut output = CPUState {
			i : 0,
			pc : 0x200,
			sp : 0,
			delay_timer : 0,
			sound_timer : 0,
			v : [0; 16],
			memory : [0; 4096],
			stack : [0; 16],
			screen : [false; SCREEN_WIDTH * SCREEN_HEIGHT],
	
			rng : rand::thread_rng()
		};
		
		let n = CPUState::HEXA_SPRITES.len();
		for i in 0 .. n {
			output.memory[i] = CPUState::HEXA_SPRITES[i];
		}

		output
	}

    fn push(&mut self, val : u16)
    {
		if (self.sp as usize) < self.stack.len() {
			self.stack[self.sp as usize] = val;
            self.sp += 1;
        }
    }
    
    fn pop(&mut self) -> u16
    {
        if self.sp > 0 {
            self.sp -= 1;
        }
        return self.stack[self.sp as usize];
    }

	fn decrease_timers(&mut self) {
		if self.sound_timer > 0 {
			self.sound_timer -= 1;
		}
		if self.delay_timer > 0 {
			self.delay_timer -= 1;
		}
	}

	fn interpret(&mut self, inst_lo: u8, inst_hi: u8, increment_pc: &mut bool, keys: [bool; 16])
	{
		let inst : u16 =
			(u16::from(inst_hi) << 8) | u16::from(inst_lo);

        match inst & 0xF000 {
			0x0000 => {
				match inst & 0x0FFF {
				0x0E0 => {
					self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
				}
				0x0EE => {
					self.pc = self.pop();
				}
				_ => { /* Ignored */ return; }
				}
			}
			0x1000 => {
				self.pc = inst & 0x0FFF;
				*increment_pc = false;
			}
			0x2000 => {
				self.push(self.pc);
				self.pc = inst & 0x0FFF;
				*increment_pc = false;
			}
			0x3000 => {
				let x = usize::from(inst_hi & 0x0F);
				if self.v[x] == inst_lo {
					self.pc += 4;
					*increment_pc = false;
				}
			}
			0x4000 => {
				let x = usize::from(inst_hi & 0x0F);
				if self.v[x] != inst_lo {
					self.pc += 4;
					*increment_pc = false;
				}
			}
			0x5000 => {
				let x = usize::from(inst_hi & 0x0F);
				let y = usize::from((inst_lo >> 4) & 0x0F);
	
				if (inst_lo & 0x0F) != 0 {
					return;
					/* TODO: Invalid instruction */
				}
	
				if self.v[x] == self.v[y] {
					self.pc += 4;
					*increment_pc = false;
				}
			}
			0x6000 => {
				let x = usize::from(inst_hi & 0x0F);
				self.v[x] = inst_lo;
			}
			0x7000 => {
				let x = usize::from(inst_hi & 0x0F);
				self.v[x] = ((self.v[x] as u16 + inst_lo as u16) & 0xFF) as u8;
			}
			0x8000 => {
				let x = usize::from(inst_hi & 0x0F);
				let y = usize::from((inst >> 4) & 0x0F);
				
				let vy = self.v[y];
				match inst & 0x0F {
				0x00 => { self.v[x] = vy; }
				0x01 => { self.v[x] |= vy; }
				0x02 => { self.v[x] &= vy; }
				0x03 => { self.v[x] ^= vy; }
				0x04 => {
					let res = self.v[x] as u16 + vy as u16;
					self.v[0xF] = if res >= 256 { 1 } else { 0 };
					self.v[x] = (res & 0xFF) as u8;
				}
				0x05 => {
					let res = self.v[x] as i16 - vy as i16;
					self.v[0xF] = if res > 0 { 1 } else { 0 };
					self.v[x] = res as u8;
				}
				0x06 => {
					self.v[0xF] = if self.v[x] & 0x1 != 0 { 1 } else { 0 };
					self.v[x] >>= 1;
				}
				0x07 => {
					self.v[0xF] = if vy > self.v[x] { 1 } else { 0 };
					self.v[x] = vy - self.v[x];
				}
				0x0E => {
					self.v[0xF] = if self.v[x] & 0x80 != 0 { 1 } else { 0 };
					self.v[x] <<= 1;
				}
	
				_ => { return; /* TODO: Invalid instruction */ }
				}
			}
			0x9000 => {
				let x = usize::from(inst_hi & 0x0F);
				let y = usize::from((inst_lo >> 4) & 0x0F);
	
				if (inst_lo & 0x0F) != 0 {
					return;
					/* TODO: Invalid instruction */
				}
	
				if self.v[x] != self.v[y] {
					self.pc += 4;
					*increment_pc = false;
				}
			}
			0xA000 => { self.i = inst & 0x0FFF; }
			0xB000 => {
				self.pc = u16::from(self.v[0]) + inst & 0x0FFF;
				*increment_pc = false;
			}
			0xC000 => {
				let x = usize::from((inst >> 8) & 0x0F);
				let mask = u8::from(self.rng.gen_range(0 .. 255));
				
				self.v[x] = inst_lo & mask;
			}
			0xD000 => {
				self.v[0xF] = 0;
	
				let vx = self.v[usize::from(inst_hi & 0x0F)] as usize;
				let vy = self.v[usize::from((inst_lo >> 4) & 0x0F)] as usize;
				let n = usize::from(inst_lo & 0x0F);
	
				if n == 0 {
					/* TODO: Implement Hi-Res mode */
					return;
				// Display N bytes sprites
				} else {
					let start = self.i as usize;
					for i in 0 .. n {
						let byte = self.memory[(i + start) % self.memory.len()];
						for j in 0 .. 8 {
							let flip_pixel =
								if byte & (1 << (7 - j)) == 0 { false }
								else { true };
							
							let screen_pos = SCREEN_WIDTH * ((vy + i) % SCREEN_HEIGHT)
								+ (vx + j) % SCREEN_WIDTH;
	
							// If we're about to turn off a pixel
							// Set v[0xF] to 1
							if self.screen[screen_pos] && flip_pixel {
								self.v[0xF] = 1;
							}
							self.screen[screen_pos] ^= flip_pixel;
						}
					}
				}
			}
			0xE000 => {
				let vx = usize::from(self.v[usize::from(inst_hi & 0x0F)]);
	
				match inst_lo {
				0x9E => {
					if vx < 16 {
						if keys[vx] {
							self.pc += 4;
							*increment_pc = false;
						}
					}
				}
				0xA1 => {
					if vx < 16 {
						if !keys[vx] {
							self.pc += 4;
							*increment_pc = false;
						}
					}
				}
	
				_ => { return; /* TODO: Invalid instruction */ }
				}
			}
			0xF000 => {
				let x = usize::from(inst_hi & 0x0F);
	
				match inst_lo {
				0x07 => { self.v[x] = self.delay_timer; }
				0x0A => {
					for i in 0 .. keys.len() {
						if keys[i] {
							self.v[x] = i as u8;
							return;
						}
					}
					// If no keys are pressed
					// We will re-execute the same instruction
					// On the next cycle
					*increment_pc = false;
				}
				0x15 => { self.delay_timer = self.v[x]; }
				0x18 => { self.sound_timer = self.v[x]; }
				0x1E => {
					self.i += u16::from(self.v[x]);
					self.i &= 0xFFF;
				}
				0x33 => {
					let vx = self.v[x];
					let mem_len = self.memory.len();
					self.memory[self.i as usize] =
						((vx / 100) % 10) as u8;
					self.memory[(self.i + 1) as usize % mem_len] =
						((vx / 10) % 10) as u8;
					self.memory[(self.i + 2) as usize % mem_len] =
						(vx % 10) as u8;
				}
				0x29 => {
					let x = (inst_hi & 0x0F) as usize;
					/* NOTE: The hexadecimal sprites will be
					   stored between 0 and HEXA_SPRITE_SIZE*16 in memory */
					self.i = (self.v[x] as u16) * (CPUState::HEXA_SPRITE_SIZE as u16);
				}
				0x55 => {
					let k = (inst_hi & 0x0F) as usize;
					let mem_len = self.memory.len();
					for i in 0 ..= k {
						self.memory[(self.i as usize + i) % mem_len] =
							self.v[i];
					}
				}
				0x65 => {
					let k = (inst_hi & 0x0F) as usize;
					let mem_len = self.memory.len();
					for i in 0 ..= k {
						self.v[i] =
							self.memory[(self.i as usize + i) % mem_len];
					}
				}
				_ => { return; /* TODO: Invalid instruction */ }
				}
			}
	
			_ => { return; /* Unreachable */ }
		}	
	}

    fn cycle(&mut self, keys : [bool; 16])
	{
		let mut increment_pc = true;
		let pc : usize = self.pc.into();

		let inst_hi = self.memory[pc];
		let inst_lo = self.memory[(pc + 1) % self.memory.len()];

		self.interpret(inst_lo, inst_hi, &mut increment_pc, keys);

		if increment_pc {
			self.pc += 2;
		}
		self.pc &= 0xFFF;
	}
}

use std::env;
use std::fs;
use raylib::prelude::*;

fn main()
{
	let mut state = CPUState::new();
	let args: Vec<String> = env::args().collect();
	if args.len() != 2 {
		println!("./chip8 [PATH TO THE ROM]");
		return;
	}

	let rom = fs::read(args[1].as_str());
	if let Err(e) = rom {
		dbg!("Error: {}", e);
		return;
	}

	let rom = rom.unwrap();
	let rom_size = rom.len();
	if rom_size > (0xFFF - 0x200) {
		print!("Invalid size: {}", rom.len());
		return;
	}

	for i in 0 .. rom_size {
		state.memory[i + 0x200] = rom[i];
	}
	
	let (mut rl, thread) = raylib::init()
        .size((SCREEN_WIDTH * ZOOM) as i32, (SCREEN_HEIGHT * ZOOM) as i32)
        .title("Chip 8 Emulator")
        .build();

	rl.set_target_fps(60);
	while !rl.window_should_close() {
		for _ in 0 .. 5 {
			state.cycle([false; 16]);
		}

		if state.sound_timer > 0 {
			/* TODO: Play sound */
			println!("BEEP !");
		}
		state.decrease_timers();
		
		let mut d = rl.begin_drawing(&thread);
		d.clear_background(Color::BLACK);
		for x in 0 .. SCREEN_WIDTH {
			for y in 0 .. SCREEN_HEIGHT {
				if state.screen[y * SCREEN_WIDTH + x] {
					d.draw_rectangle((ZOOM*x) as i32, (ZOOM*y) as i32,
						ZOOM as i32, ZOOM as i32,
						Color::WHITE);
				}
			}
		}
	}
}
