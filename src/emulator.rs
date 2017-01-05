use piston::input::*;
use piston_window::{PistonWindow, Texture};
use gfx_device_gl::Resources as R;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::{io, fmt};
use std::path::Path;
use piston::window::Window;

use cpu::Cpu;
use mmu::Memory;
use cartridge::*;

// Clock cycles between every screen refresh
pub const SCREEN_REFRESH_INTERVAL: u32 = 70224; // clock cycles

pub struct Emulator {
    pub cpu: Cpu,
    pub mem: Memory,
    pub rom_header: CartridgeHeader,

    is_frame_stepping: bool,
    is_instr_stepping: bool,
    is_debugging: bool,
    frame_cycles: u32, // cycles left until the frame ends
    pub frame_count: u32,
}

impl Emulator {
    pub fn new<W: Window>(window: &PistonWindow<W>, rom_path: &String) -> Emulator {
        let mut emu = Emulator {
            cpu: Cpu::new(),
            mem: Memory::new(window),
            rom_header: Default::default(),
            is_frame_stepping: false,
            is_instr_stepping: false,
            is_debugging: true,
            frame_cycles: 0,
            frame_count: 0,
        };

        // Read rom and move ownership to memory component
        emu.mem.set_rom(try_open_rom(&rom_path));
        emu.read_header();

        // If the rom is more than 32KB, it has VRAM so we need to copy it
        if emu.rom_header.rom_size > 0 {
            emu.mem.copy_vram();
        }
        emu.mem.copy_rom();

        emu.mem.find_mbc(emu.rom_header.cartridge_type);

        // Give immutable reference of rom header to memory component
        //emu.mem.borrow_rom_header(&emu.rom_header);

        emu
    }

    // Render screen
    pub fn render<W: Window>(&mut self, args: &RenderArgs, window: &mut PistonWindow<W>, framebuffer: &mut Texture<R>, evt: &Event) {
        self.mem.gpu.display(window, evt);
    }

    // Update state
    // Gets called once a frame
    pub fn update(&mut self, args: &UpdateArgs) {

        // If is_stepping is false, runs for a frame (~70k clock cycles)
        // If it's true runs for just 1 instruction

        while self.frame_cycles < SCREEN_REFRESH_INTERVAL {
            let cycles = self.cpu.exec(&mut self.mem);
            self.mem.timer.step(cycles, &mut self.mem.if_);
            self.mem.gpu.step(cycles, &mut self.mem.if_);

            self.frame_cycles += cycles;

            if self.cpu.get_regs().stop {self.cpu.stop(); return; }
            if self.is_instr_stepping { self.set_running(false) }; // kinda broken
        }
        if self.frame_cycles >= SCREEN_REFRESH_INTERVAL {
            self.frame_cycles -= SCREEN_REFRESH_INTERVAL;
        }

        self.frame_count += 1;
        if self.is_frame_stepping { self.set_running(false) };
        // Update gpu image data
        self.mem.gpu.update();
    }

    fn read_header(&mut self) {
        self.rom_header = read_header_impl(&self);
    }
    pub fn get_header(&self) -> &CartridgeHeader {
        &self.rom_header
    }
    pub fn is_debugging(&self) -> bool {
        self.is_debugging
    }
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
        self.is_debugging = !self.is_debugging;
    }

}

impl fmt::Debug for Emulator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
" State: {}
 Frame: {}   Cycles: {}",
            if self.cpu.is_running {"Running"} else {"Paused"},
            self.frame_count,
            self.cpu.total_cycles,
        )
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