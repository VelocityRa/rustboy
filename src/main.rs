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
use std::borrow::BorrowMut;

use glutin_window::GlutinWindow;

use piston_window::*;

use gfx::traits::*;
use gfx_core::factory::Typed;
use gfx::format::{DepthStencil, Formatted, Srgba8};

use gfx_core::Resources;
use gfx_graphics::Gfx2d;
use gfx_debug_draw::DebugRenderer;

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

	let mut window: PistonWindow<GlutinWindow> = 
		WindowSettings::new(
			WINDOW_TITLE,
			SCREEN_DIMS,
		)
		.opengl(OPENGL)
		.build()
		.unwrap();
	window.set_max_fps(60);
	window.set_ups(60);

	let mut emu = emulator::Emulator::new(rom_path);

	emu.read_header();

	// Append game name to title
	window.set_title(
		format!("{} - {}", WINDOW_TITLE, emu.rom_header.get_game_title())
		);

    let mut debug_renderer = {
        let text_renderer = {
            gfx_text::new(window.factory.clone()).unwrap()
        };
        DebugRenderer::new(window.factory.clone(), text_renderer, 64).ok().unwrap()
    };

	// Main Event Loop
	while let Some(evt) = window.next() {
		if let Some(r) = evt.render_args() {
			use graphics::*;

			const BG: [f32; 4] = [0.15, 0.15, 0.15, 1.0];

	        window.draw_2d(&evt, |c, g| {
            	clear(BG, g);

	            debug_renderer.draw_text_at_position(
                "Test",
                [6.0, 0.0, 0.0],
                [1.0, 0.0, 0.0, 1.0],

            );

            debug_renderer.draw_line([0.2, 0.2, 0.0], [0.0, 0.0, 5.0], [0.3, 0.3, 1.0, 1.0]);
        	});

			//emu.render(&r);
		}

		if let Some(u) = evt.update_args() {
			if emu.is_running() {
				//emu.update(&u);
			}
		}
	}
}
