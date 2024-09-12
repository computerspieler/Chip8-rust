mod chip8;

use std::env;
use std::fs;
use raylib::prelude::*;

/* The keyboard's layout:
        1 2 3 C
        4 5 6 D
        7 8 9 E
        A 0 B F
*/
const KEYBOARD_KEYS: [KeyboardKey; 16] = [
	KeyboardKey::KEY_X, 	// 0
	KeyboardKey::KEY_ONE,	// 1
	KeyboardKey::KEY_TWO,	// 2
	KeyboardKey::KEY_THREE,	// 3
	KeyboardKey::KEY_A,     // 4
	KeyboardKey::KEY_Z,   	// 5
	KeyboardKey::KEY_E,   	// 6
	KeyboardKey::KEY_Q, 	// 7
	KeyboardKey::KEY_S, 	// 8
	KeyboardKey::KEY_D, 	// 9
	KeyboardKey::KEY_W, 	// A
	KeyboardKey::KEY_C, 	// B
	KeyboardKey::KEY_FOUR,	// C
	KeyboardKey::KEY_R,  	// D
	KeyboardKey::KEY_F,  	// E
	KeyboardKey::KEY_V	    // F
];
const ZOOM: usize = 10;

fn main()
{
	let mut state = chip8::CPUState::new();
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
		state.set_byte(i + 0x200, rom[i]);
	}
	
	let (mut rl, thread) = raylib::init()
        .size((chip8::SCREEN_WIDTH * ZOOM) as i32, (chip8::SCREEN_HEIGHT * ZOOM) as i32)
        .title("Chip 8 Emulator")
        .build();

	rl.set_target_fps(60);
	while !rl.window_should_close() {
		let keys = KEYBOARD_KEYS.map(|x| rl.is_key_down(x));
		for _ in 0 .. 5 {
			state.cycle(keys);
		}

		if state.should_beep() {
			/* TODO: Play sound */
			println!("BEEP !");
		}
		state.decrease_timers();
		
		let mut d = rl.begin_drawing(&thread);
		d.clear_background(Color::BLACK);
		for x in 0 .. chip8::SCREEN_WIDTH {
			for y in 0 .. chip8::SCREEN_HEIGHT {
				if state.get_pixel(x, y) {
					d.draw_rectangle((ZOOM*x) as i32, (ZOOM*y) as i32,
						ZOOM as i32, ZOOM as i32,
						Color::WHITE);
				}
			}
		}
	}
}
