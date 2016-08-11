extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate gfx_graphics;
extern crate gfx;
extern crate gfx_core;
extern crate gfx_debug_draw;
extern crate gfx_text;
extern crate gfx_device_gl;
extern crate piston_window;

use std::env;

use glutin_window::GlutinWindow;

use piston_window::*;

#[macro_use]
mod logger;

mod cpu;
mod gpu;
mod mmu;
mod cartridge;
mod emulator;
mod timer;

const SCREEN_NATIVE_DIMS: [u32; 2] = [160, 144];
const SCREEN_MULT: u32 = 3;

const OPENGL: OpenGL = OpenGL::V3_2;
static WINDOW_TITLE: &'static str = "Rust Boy Emulator";
const BG_COLOR: [f32; 4] = [0.12, 0.12, 0.12, 1.0];
const TEXT_COLOR: [f32; 4] = [1., 1., 1., 0.4];

fn main() {
	
	let args: Vec<_> = env::args().collect();
	let rom_path: &String;

	match args.len() {
		2 => rom_path = &args[1],
		_ => panic!("No arguments provided.
				USAGE: rustboy <path/to/rom>"),
	}

	const SCREEN_DIMS: [u32; 2] = [SCREEN_NATIVE_DIMS[0] * SCREEN_MULT, 
		SCREEN_NATIVE_DIMS[1] * SCREEN_MULT];

	let mut window: PistonWindow<GlutinWindow> = 
		WindowSettings::new(
			WINDOW_TITLE,
			SCREEN_DIMS,
		)
		.opengl(OPENGL)
		.build()
		.unwrap();
	window.set_max_fps(60);
	window.set_ups(5);

	let mut emu = emulator::Emulator::new(rom_path);

	emu.read_header();

	// Append game name to title
	window.set_title(
		format!("{} - {}", WINDOW_TITLE, emu.rom_header.get_game_title())
		);

	let output_color = window.output_color.clone();

	// Initialize text renderer.
	let mut text = gfx_text::new(window.factory.clone())
		.with_size(16)
		.with_font("resources/fonts/joystix monospace.ttf")
		.build().unwrap();

	// Main Event Loop
	while let Some(evt) = window.next() {

		// Space to pause/unpause emulation
		if let Some(Button::Keyboard(Key::Space)) = evt.press_args() {
            emu.toggle_running();
        }

		if let Some(r) = evt.render_args() {

			// Debug stuff

			let mut dbg_string = format!("{:?}\n\n", emu.cpu);
			dbg_string.push_str(&format!("Registers:\n{:?}\n", emu.cpu.get_regs()));
			dbg_string.push_str(&format!("Flags:\n{:?}\n", emu.cpu.get_flags()));
			dbg_string.push_str(&format!("Timers:\n{:?}\n", emu.mem.get_timers()));
			
			// Split lines and place them appropriately
			// TODO: Possibly use anchors for text placement (or a mono font)
			let dbg_lines = dbg_string.split('\n');
			for (line_n, line) in dbg_lines.enumerate() {
				text.add(
					line,
					[10, 10 + line_n as i32 * 16 + 1],
					TEXT_COLOR,
				);
			}

			// End of debug stuff

			// Drawing
			window.draw_2d(&evt, |c, g| {
				clear(BG_COLOR, g);

				text.draw(&mut g.encoder, &output_color);
			});
			
			// Emulator rendering
			//emu.render(&r);
		}

		if let Some(u) = evt.update_args() {
			if emu.is_running() {
				emu.update(&u);
			}
		}
	}
}
