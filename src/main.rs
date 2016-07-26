extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

use std::path::Path;
use piston::event_loop::*;
use piston::input::*;
use piston::window::WindowSettings;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::*;

mod cpu;
mod gpu;
mod mmu;

fn main() {

	let opengl = OpenGL::V3_2;
	let mut window: Window = 
		WindowSettings::new(
			"RustBoy Emulator",
			[640, 480]
		)
		.opengl(opengl)
		.exit_on_esc(true)
		.build()
		.unwrap();


	let mut gl = GlGraphics::new(opengl);
	let mut events = window.events();

	while let Some(e) = events.next(&mut window) {
		use graphics::*;

	}

}
