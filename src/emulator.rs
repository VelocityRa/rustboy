use piston::input::*;
use piston_window::{PistonWindow, Texture};
use gfx_device_gl::Resources as R;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::path::Path;

use cpu::Cpu;
use mmu::Memory;
use gpu::Gpu;
use cartridge::*;

// Clock cycles between every screen refresh 
pub const SCREEN_REFRESH_INTERVAL: u32 = 70224; // clock cycles

pub struct Emulator {
    pub cpu: Cpu,
    pub mem: Memory,
    pub rom_header: CartridgeHeader,
}

impl Emulator {
    pub fn new(window: &PistonWindow, rom_path: &String) -> Emulator {
        let mut emu = Emulator {
            cpu: Cpu::new(),
            mem: Memory::new(window),
            rom_header: Default::default(),
        };

        // Read rom and move ownership to memory component
        emu.mem.set_rom(try_open_rom(&rom_path));

        // Give immutable reference of rom header to memory component
        //emu.mem.borrow_rom_header(&emu.rom_header);

        emu
    }

    // Render screen
    pub fn render(&mut self, args: &RenderArgs, window: &mut PistonWindow, framebuffer: &mut Texture<R>, evt: &Event) {

        self.mem.gpu.display(window, evt);
    }

    // Update state
    // Gets called once a frame
    pub fn update(&mut self, args: &UpdateArgs) {
        // If is_stepping is false, runs for a frame (~70k clock cycles)
        // If it's true runs for just 1 instruction
        let mut temp_total_cycles = 0;
        while temp_total_cycles <= SCREEN_REFRESH_INTERVAL {
            let cycles = self.cpu.exec(&mut self.mem);
            self.mem.timer.step(cycles, &mut self.mem.if_);
            self.mem.gpu.step(cycles, &mut self.mem.if_);
            
            temp_total_cycles += cycles;

            if self.cpu.get_regs().stop {self.cpu.stop(); return; }
            if self.cpu.is_stepping { return; }
        }
    }

    pub fn read_header(&mut self) {
        self.rom_header = read_header_impl(&self);
    }
    pub fn get_header(&self) -> &CartridgeHeader {
        &self.rom_header
    }
/*
    pub fn update_cpu_timers(&mut self, dt: f64) {
        self.cpu.update_timers(&mut self.mem);
    }
*/
    #[inline]
    pub fn is_running(&self) -> bool {
        self.cpu.is_running
    }
    pub fn set_running(&mut self, state: bool) {
        self.cpu.is_running = state;
    }
    pub fn toggle_running(&mut self) {
        self.cpu.is_running = !self.cpu.is_running;
    }
    pub fn toggle_debugging(&mut self) {
        self.cpu.is_debugging = !self.cpu.is_debugging;
    }
    
}

fn open_rom<P: AsRef<Path>>(rom_path: P) -> io::Result< Vec<u8> > {

    // try! to open the file
    let mut rom_file = try!(File::open(rom_path));

    // Create the buffer
    let mut rom_buffer: Vec<u8> = Vec::new();

    // Read the data
    try!(rom_file.read_to_end(&mut rom_buffer));

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
            data
        },
    }
}


//  ======================================
//  |               TESTS                |
//  ======================================

#[cfg(test)]
mod emu_tests {
    use super::*;

}