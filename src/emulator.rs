use opengl_graphics::*;
use piston::input::*;

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::path::Path;

pub struct Emulator {
	pub gl: GlGraphics,			// OpenGL drawing backend
	pub rom_loaded: Vec<u8>,	// Rom in heap
}

impl Emulator {

	// Render screen
	pub fn render(&mut self, args: &RenderArgs) {
		use graphics::*;
	}

	// Update state
	pub fn update(&mut self, args: &UpdateArgs) {

	}

}

fn open_rom<P: AsRef<Path>>(rom_path: P) -> io::Result< Vec<u8> > {

	// try! to open the file
	let mut rom_file = try!(File::open(rom_path));

	// Create the buffer
	let mut rom_buffer: Vec<u8> = Vec::new();

	// Read the data
	let bytes_read = try!(rom_file.read_to_end(&mut rom_buffer));

	// no panic! issued so we're good
	return Ok(rom_buffer);
}

// Wrapper for open_rom
pub fn try_open_rom<P: AsRef<Path>>(rom_path: P) -> Vec<u8> {

	// Create a Path and a Display to the desired file
	let rom_display = rom_path.as_ref().display();

	// Call open_rom and handle Result
	match open_rom(&rom_path) {
        Err(why) => 
        	panic!("Couldn't open rom {}: {}", rom_display,
                                                   why.description()),
		Ok(data) => {
			println!("Read {} bytes from ROM: {}.", data.len(), rom_display);
			return data
		},
	};
}