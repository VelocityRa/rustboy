#![allow(dead_code)]
#![allow(unused_variables)]

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate colored;
extern crate image;
extern crate texture;
extern crate rand;
extern crate piston;
extern crate piston_window;
extern crate graphics;
extern crate glfw_window;
extern crate gfx;
extern crate gfx_core;
extern crate gfx_graphics;
extern crate gfx_device_gl;
extern crate gfx_text;
extern crate fps_counter;

use std::env;
use env_logger::LogBuilder;
use log::LogRecord;
use colored::*;

use piston_window::{OpenGL, PistonWindow, WindowSettings, Texture};
use glfw_window::GlfwWindow;
use piston::window::AdvancedWindow;
use piston::event_loop::EventLoop;
use piston::input::*;
use fps_counter::FPSCounter;

use graphics::clear;
use texture::*;

mod cpu;
mod gpu;
mod mmu;
mod cartridge;
mod emulator;
mod timer;

const OPENGL: OpenGL = OpenGL::V3_2;
static DEFAULT_LOG_LEVEL: &'static str = "debug";
static DEFAULT_LOG_LEVELS: &'static str = "gfx_device_gl=warn,cargo=error";
static WINDOW_TITLE: &'static str = "Rust Boy Emulator";

const SCREEN_MULT: u32 = 4;
const BG_COLOR: [f32; 4] = [5./255., 36./255., 5./255., 1.0];
const TEXT_COLOR: [f32; 4] = [255./255., 255./255., 255./255., 1.0];
const TEXT_TITLE_COLOR: [f32; 4] = [210./255., 210./255., 210./255., 1.0];

const NATIVE_DIMS: [u32; 2] = [160, 144];
const SCREEN_DIMS: [u32; 2] = [NATIVE_DIMS[0] * SCREEN_MULT,
                               NATIVE_DIMS[1] * SCREEN_MULT];
const FONT_SIZE: u8 = (1. + SCREEN_MULT as f32 * 4.5) as u8;


fn main() {

    // Logging stuff
    let format = |record: &LogRecord| {
        use log::LogLevel;

        let level = record.level();
        let level_str = level.to_string();
        let coloured_level = match level {
            LogLevel::Trace => level_str.white(),
            LogLevel::Debug => level_str.green(),
            LogLevel::Info => level_str.blue(),
            LogLevel::Warn => level_str.yellow().bold(),
            LogLevel::Error => level_str.red().bold(),
        };
        let record_loc = &record.location().file()[4..];    // Trim "src/"
        format!("[{}][{}]: {}", coloured_level, record_loc.cyan(), record.args())
    };
    let mut builder = LogBuilder::new();
    builder.format(format);
    builder.parse(&format!("{},{}", DEFAULT_LOG_LEVEL, DEFAULT_LOG_LEVELS));
    builder.init().unwrap();

    // Argument parsing
    let args: Vec<_> = env::args().collect();
    let rom_path: &String;

    match args.len() {
        2 => rom_path = &args[1],
        _ => {
            error!("No arguments provided.\nUSAGE: rustboy-emu <path/to/rom>");
            return;
        },
    }

    // Window creation
    let mut window: PistonWindow<GlfwWindow> =
        WindowSettings::new(
            WINDOW_TITLE,
            SCREEN_DIMS,
        )
        .opengl(OPENGL)
        .resizable(false)
        .build()
        .unwrap();
    window.set_max_fps(60);
    window.set_ups(60);

    // Initialize emulator
    let mut emu = emulator::Emulator::new(&window, rom_path);

    // Append game name to title
    window.set_title(
       String::from(format!("{} - {}", WINDOW_TITLE, emu.rom_header.get_game_title()))
    );

    let output_color = window.output_color.clone();

    // Initialize text renderers
    macro_rules! make_font(
    () => (gfx_text::new(window.factory.clone())
            .with_size(FONT_SIZE)
            .with_font("resources/fonts/joystix monospace.ttf")
            .build().unwrap();
    ));
    let mut text = make_font!();
    let mut text_shadow = make_font!();

    // Set up framebuffer
    let ts = TextureSettings::new().filter(texture::Filter::Nearest).compress(false).generate_mipmap(false);
    let mut framebuffer =
        Texture::create(&mut window.factory, Format::Rgba8, &*emu.mem.gpu.image_data, NATIVE_DIMS, &ts)
        .expect("Couldn't create framebuffer texture");

    // Set up framerate counter
    let mut fps = FPSCounter::new();

    // Main Event Loop
    while let Some(evt) = window.next() {
        //debug!("EVENT: {:?}", evt);

        // Space to pause/unpause emulation
        if let Some(Button::Keyboard(Key::Space)) = evt.press_args() {
            emu.toggle_running();
        }

        // D to enable/disable debugging text
        if let Some(Button::Keyboard(Key::D)) = evt.press_args() {
            emu.toggle_debugging();
        }

        if let Event::Render(_) = evt {
            //println!("RENDER: {}", emu.frame_count);
            // Draw BG
            window.draw_2d(&evt, |c, g| {
                clear(BG_COLOR, g);
            });

            // Emulator rendering (does nothing for now, look below)
            //emu.render(&r, &mut window, &mut framebuffer, &evt);

            // TODO: Move these to the above call
            // Update the framebuffer
            UpdateTexture::update(&mut framebuffer, &mut window.encoder, Format::Rgba8,
                &*emu.mem.gpu.image_data, [0,0], NATIVE_DIMS).unwrap();
            // Draw the screen
            window.draw_2d(&evt, |c, g| {
                use graphics::Transformed;

                emu.mem.gpu.img.draw(&framebuffer, &c.draw_state,
                    c.transform.scale(SCREEN_MULT as f64, SCREEN_MULT as f64), g);
            });

            // TODO: Move to seperate module (debugger.rs)
            // Debugger rendering
            if emu.is_debugging() {
                let mut dbg_string = format!("\tEmulator\n{:?}\n FPS: {}\n\n", emu, fps.tick());
                dbg_string.push_str(&format!("\tRegisters\n{:?}\n\n", emu.cpu.get_regs()));
                dbg_string.push_str(&format!("\tFlags\n{:?}\n\n", emu.cpu.get_flags()));
                dbg_string.push_str(&format!("\tTimers\n{:?}\n\n", emu.mem.get_timers()));

                // Split lines and place them appropriately
                let dbg_lines = dbg_string.split('\n');
                for (line_n, line) in dbg_lines.enumerate() {
                    text.add(
                        line,
                        [10, 10 + line_n as i32 * FONT_SIZE as i32 + 1],
                        if line.len() !=0 && line.as_bytes()[0] == '\t' as u8 {  // sorry
                            TEXT_TITLE_COLOR
                        } else {
                            TEXT_COLOR
                        },
                    );
                    text_shadow.add(
                        line,
                        [10 + 2, 10 + 2 + line_n as i32 * FONT_SIZE as i32 + 1],
                        [0., 0., 0., 1.]
                    );
                }
                window.draw_2d(&evt, |c, g| {
                    text_shadow.draw(&mut g.encoder, &output_color).unwrap();
                    text.draw(&mut g.encoder, &output_color).unwrap();
                });
            }
        }

        if let Some(u) = evt.update_args() {
            //println!("UPDATE: {}", emu.frame_count);
            if emu.is_running() {
                debug!("FRAME START: {}", emu.frame_count);
                emu.update(&u);
            }
        }
    }
}
