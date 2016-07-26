extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

use std::path::Path;
use std::env;

use piston::event_loop::*;
use piston::input::*;
use piston::window::WindowSettings;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::*;

mod cpu;
mod gpu;
mod mmu;
mod emulator;

const OPENGL: OpenGL = OpenGL::V3_2;

fn main() {

	let args: Vec<_> = env::args().collect();
	let rom_path: &String;
    if args.len() < 2 {
        panic!("No arguments provided.
        		USAGE: rustboy <path/to/rom>");
    } else {
    	rom_path = &args[1];
    }

    let mut window: Window = 
		WindowSettings::new(
			"Rust Boy Emulator",
			[640, 480]
		)
		.opengl(OPENGL)
		.exit_on_esc(true)
		.build()
		.unwrap();


	let mut emu = emulator::Emulator {
		gl: GlGraphics::new(OPENGL),
		rom_loaded: emulator::try_open_rom(&rom_path),
	};

	// Main Event Loop
	let mut events = window.events();
	while let Some(evt) = events.next(&mut window) {
		
		if let Some(r) = evt.render_args() {
			emu.render(&r);
		}

		if let Some(u) = evt.update_args() {
			emu.update(&u);
		}

	}

}
