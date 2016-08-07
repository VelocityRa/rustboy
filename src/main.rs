extern crate piston;
extern crate graphics;
extern crate sdl2_window;
extern crate opengl_graphics;
extern crate gfx_debug_draw;

use std::env;

use piston::event_loop::*;
use piston::input::*;
use piston::window::WindowSettings;
use piston::window::AdvancedWindow;
use sdl2_window::Sdl2Window as Window;
use opengl_graphics::*;

#[macro_use]
mod logger;

mod cpu;
mod gpu;
mod mmu;
mod emulator;
mod timer;

const SCREEN_NATIVE_DIMS: [u32; 2] = [160, 144];
const SCREEN_MULT: u32 = 3;

const OPENGL: OpenGL = OpenGL::V3_2;
static WINDOW_TITLE: &'static str = "Rust Boy Emulator";

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

	let mut window: Window = 
		WindowSettings::new(
			WINDOW_TITLE,
			SCREEN_DIMS,
		)
		.opengl(OPENGL)
		.exit_on_esc(true)
		.build()
		.unwrap();

	let mut emu = emulator::Emulator::new(rom_path);

	emu.read_header();

	// Append game name to title
	window.set_title(
		format!("{} - {}", WINDOW_TITLE, emu.rom_header.get_game_title())
		);

	// 

	// Main Event Loop
	let mut events = window.events().max_fps(60).ups(60);
	'main_loop: while let Some(evt) = events.next(&mut window) {
		if let Some(r) = evt.render_args() {
			emu.render(&r);
		}

		if let Some(u) = evt.update_args() {
			if emu.is_running() {
				emu.update(&u);
			}
		}
	}
}
