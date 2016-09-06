//
//		Memory Management Unit 
//

/*
  0000-3FFF   16KB ROM Bank 00     (in cartridge, fixed at bank 00)
  4000-7FFF   16KB ROM Bank 01..NN (in cartridge, switchable bank number)
  ________________________________________________________________________
              we map the above banks to the loaded rom
              and raw_mem stores the memory below
  ________________________________________________________________________
  8000-9FFF   8KB Video RAM (VRAM) (switchable bank 0-1 in CGB Mode)
  A000-BFFF   8KB External RAM     (in cartridge, switchable bank, if any)
  C000-CFFF   4KB Work RAM Bank 0 (WRAM)
  D000-DFFF   4KB Work RAM Bank 1 (WRAM)  (switchable bank 1-7 in CGB Mode)
  E000-FDFF   Same as C000-DDFF (ECHO)    (typically not used)
  FE00-FE9F   Sprite Attribute Table (OAM)
  FEA0-FEFF   Not Usable
  FF00-FF7F   I/O Ports
  FF80-FFFE   High RAM (HRAM)
  FFFF        Interrupt Enable Register
*/

#![allow(dead_code)]

use piston_window::PistonWindow;

use timer::Timer;
use gpu::Gpu;
use cartridge::CartridgeHeader;

const START_MAPPED_MEM: usize = 0x8000;
const MEM_SIZE: usize = 0xFFFF + 1 - START_MAPPED_MEM;

pub struct Memory {
	// Interrupt flags, http://problemkaputt.de/pandocs.htm#interrupts
	// The master enable flag will be on the cpu
	pub if_: u8,
	pub ie_: u8,

	raw_mem: [u8; MEM_SIZE],

	pub rom_loaded: Vec<u8>,

	//rom_header: Option<&CartridgeHeader>,

	pub timer: Box<Timer>,
	pub gpu: Box<Gpu>,
}

impl Memory {
	// Allocate a 64k byte array and zero initialize it
	// This is all the system's RAM
	pub fn new(window: &PistonWindow) -> Memory {
		let mut mem = Memory {
			if_: 0u8,
			ie_: 0u8,
			raw_mem: [0u8; MEM_SIZE],
			rom_loaded: Vec::new(),

			//rom_header: None,
			timer: Box::new(Timer::new()),
			gpu: Box::new(Gpu::new(window)),
		};
		mem.power_on();
		mem
	}

	pub fn power_on(&mut self) {
		// From http://problemkaputt.de/pandocs.htm#powerupsequence
		self.write_byte_raw(0xff05, 0x00); // TIMA
		self.write_byte_raw(0xff06, 0x00); // TMA
		self.write_byte_raw(0xff07, 0x00); // TAC
		self.write_byte_raw(0xff10, 0x80); // NR10
		self.write_byte_raw(0xff11, 0xbf); // NR11
		self.write_byte_raw(0xff12, 0xf3); // NR12
		self.write_byte_raw(0xff14, 0xbf); // NR14
		self.write_byte_raw(0xff16, 0x3f); // NR21
		self.write_byte_raw(0xff17, 0x00); // NR22
		self.write_byte_raw(0xff19, 0xbf); // NR24
		self.write_byte_raw(0xff1a, 0x7f); // NR30
		self.write_byte_raw(0xff1b, 0xff); // NR31
		self.write_byte_raw(0xff1c, 0x9F); // NR32
		self.write_byte_raw(0xff1e, 0xbf); // NR33
		self.write_byte_raw(0xff20, 0xff); // NR41
		self.write_byte_raw(0xff21, 0x00); // NR42
		self.write_byte_raw(0xff22, 0x00); // NR43
		self.write_byte_raw(0xff23, 0xbf); // NR30
		self.write_byte_raw(0xff24, 0x77); // NR50
		self.write_byte_raw(0xff25, 0xf3); // NR51
		self.write_byte_raw(0xff26, 0xf1); // NR52
		self.write_byte_raw(0xff40, 0xb1); // LCDC, tweaked to turn the window on
		self.write_byte_raw(0xff42, 0x00); // SCY
        self.write_byte_raw(0xff43, 0x00); // SCX
        self.write_byte_raw(0xff44, 0x00); // LY
		self.write_byte_raw(0xff45, 0x00); // LYC
		self.write_byte_raw(0xff47, 0xfc); // BGP
		self.write_byte_raw(0xff48, 0xff); // OBP0
		self.write_byte_raw(0xff49, 0xff); // OBP1
		self.write_byte_raw(0xff4a, 0x00); // WY
		self.write_byte_raw(0xff4b, 0x07); // WX, tweaked to position the window at (0, 0)
		self.write_byte_raw(0xffff, 0x00); // IE

	}
	pub fn set_rom(&mut self, rom: Vec<u8>) {
		self.rom_loaded = rom;
	}
	// Borrow
	// pub fn borrow_rom_header(&mut self, header: &CartridgeHeader) {
	// 	self.rom_header = Some(header);
	// }

    pub fn get_timers(&self) -> &Timer {
        &self.timer.as_ref()
    }

	// Private members

	fn read_byte_raw(&self, addr: u16) -> u8 {
		let addr: usize = (addr as usize) - START_MAPPED_MEM;
		assert!(addr <= MEM_SIZE,
		 "Invalid memory read: {:04X}", addr);

		self.raw_mem[addr]
	}
	
	fn read_word_raw(&self, addr: u16) -> u16 {
		let addr: usize = (addr as usize) - START_MAPPED_MEM;
		assert!(addr <= MEM_SIZE - 1,
		 "Invalid memory read: {:04X}", addr);

		(self.raw_mem[addr] as u16) << 8 |
		(self.raw_mem[addr + 1] as u16)
	}

	fn write_byte_raw(&mut self, addr: u16, data: u8) {
		let addr: usize = (addr as usize) - START_MAPPED_MEM;
		assert!(addr <= MEM_SIZE,
		 "Invalid memory read: {:04X}", addr);

		self.raw_mem[addr] = data
	}

	fn write_word_raw(&mut self, addr: u16, data: u16) {
		let addr: usize = (addr as usize) - START_MAPPED_MEM;
		assert!(addr <= MEM_SIZE - 1,
		 "Invalid memory write: {:04X}", addr);

		self.raw_mem[addr] = (data >> 8) as u8;
		self.raw_mem[addr + 1] = (data & 0x00FF) as u8;
	}

	// Public members

	// Read Byte
	pub fn rb(&self, addr: u16) -> u8 {
		//self.debug_print_addr(addr, true);

		match addr {
			0x0000 ... 0x3FFF => self.rom_loaded[addr as u16 as usize],
			// TODO: Memory bank switching
			0x4000 ... 0x7FFF => {
					self.rom_loaded[addr as u16 as usize]
					//panic!("Bank switching unimplemented"); // self.rom_loaded[addr as u16 as usize],
				},
			0xE000 ... 0xFDFF => self.read_byte_raw(addr - 0x2000),	// Mirrored memory
			0xFEA0 ... 0xFEFF => panic!("Unusable memory accessed"),
			0xFF00 ... 0xFF79 => self.ioreg_rb(addr),

			// Timer Registers
			//0xFF04 => self.
			_ => self.read_byte_raw(addr),
		}
	}

	// Read word
	pub fn rw(&self, addr: u16) -> u16 {
		assert!(addr <= 0xFFFF - 1,
		 "Invalid memory read: {:04X}", addr);

		(self.rb(addr) as u16) |
		(self.rb(addr + 1) as u16) << 8
	}

	// Write byte
	pub fn wb(&mut self, addr: u16, data: u8) {
		//self.debug_print_addr(addr, false);

		match addr {
			0xE000 ... 0xFDFF => self.write_byte_raw(addr - 0x2000, data),	// Mirrored memory
			0xFEA0 ... 0xFEFF => panic!("Unusable memory written to"),
			0xFF00 ... 0xFF79 => self.ioreg_wb(addr, data),
			_ => self.write_byte_raw(addr, data),
		}
	}

	// Write word
	pub fn ww(&mut self, addr: u16, data: u16) {
		assert!(addr <= 0xFFFF - 1,
		 "Invalid memory write: {:04X}", addr);

		self.wb(addr, data as u8);
		self.wb(addr + 1, (data >> 8) as u8);
	}

	/// Reads a value from a known IO type register
	fn ioreg_rb(&self, addr: u16) -> u8 {
		debug!("ioreg_rb {:x}", addr);
		match (addr >> 4) & 0xF {
			// I/O Ports (0xFF0x)
			0x0 => {
				match addr & 0xF {
					// TODO: Input
					//0x0 => self.input.rb(addr),
					0x0 => {warn!("Input requested (unimplemented) in address {:04X}", addr); 0},
					0x4 => self.timer.div,
					0x5 => self.timer.tima,
					0x6 => self.timer.tma,
					0x7 => self.timer.tac,
					0xf => self.if_,

					_ => self.read_byte_raw(addr),
				}
			}
            // Video I/O Registers (0xFF4x)
            0x4 => {
                match addr & 0xF {
                    0...5 | 7...0xB | 0xF => {
        				debug!("gpu_rb {:x}", addr);
                    	self.gpu.rb(addr)
                    },
                    _ => self.read_byte_raw(addr),
                }
            }
			_ => self.read_byte_raw(addr),
		}
	}

	fn ioreg_wb(&mut self, addr: u16, data: u8) {
        debug!("ioreg_wb {:x} {:x}", addr, data);
        match (addr >> 4) & 0xF {
        	// I/O Ports (0xFF0x)
			0x0 => {
				match addr & 0xF {
					0x0 => { debug!("Serial data transfer (unimplemented) in address {:04X}, data {:02X}", addr, data)}
                    0x4 => { self.timer.div = 0; }
                    0x5 => { self.timer.tima = data; }
                    0x6 => { self.timer.tma = data; }
                    0x7 => {
                        self.timer.tac = data;
                        self.timer.update();
                    }
                    0xf => { self.if_ = data; }
                    _ => {
                        warn!("Unhandled ioreg_wb address");
                        self.write_byte_raw(addr, data);
                    }
                }
            }
            // Video I/O Registers (0xFF4x)
            0x4 => {
                match addr & 0xF {
                    0...3 | 5 | 7...0xB => {
                    	let dt = self.gpu.wb(addr, data);
        				debug!("gpu_wb {:x} {:x}", addr, data);
                    	dt
                    },
                    4 => warn!("LY read request, but it is read-only"),
                    6 => warn!("DMA transfer requested (unimplemented)"),
                    _ => self.write_byte_raw(addr, data)
                }
            }
            _ => {
                warn!("Unhandled ioreg_wb address: {:04X}", addr);
                self.write_byte_raw(addr, data);
            }
        }
	}

	fn debug_print_addr(&self, addr: u16, read: bool) {
		debug!("{} {:04X} in {}", if read {"Read from"} else {"Write to"}, addr,
		
		match addr {
			0x0000 ... 0x3FFF => "16KB ROM Bank 00",	// (in cartridge, fixed at bank 00)
			0x4000 ... 0x7FFF => "a 16KB ROM Bank", // (in cartridge, switchable bank number)
			0x8000 ... 0x9FFF => "8KB Video RAM (VRAM)", // (switchable bank 0-1 in CGB Mode)
			0xA000 ... 0xBFFF => "8KB External RAM", // (in cartridge, switchable bank, if any)
			0xC000 ... 0xCFFF => "4KB Work RAM Bank 0 (WRAM)",
			0xD000 ... 0xDFFF => "4KB Work RAM Bank 1 (WRAM)", //   (switchable bank 1-7 in CGB Mode)
			0xE000 ... 0xFDFF => "Same as C000-DDFF (ECHO)", //     (typically not used)
			0xFE00 ... 0xFE9F => "Sprite Attribute Table (OAM)",
			0xFEA0 ... 0xFEFF => "Not Usable",
			0xFF00 ... 0xFF7F => "I/O Ports",
			0xFF80 ... 0xFFFE => "High RAM (HRAM)",
			0xFFFF => "Interrupt Enable Register",
			_ => unreachable!(),
		})
	}
}

//	======================================
//	|               TESTS                |
//	======================================

#[cfg(test)]
mod mem_tests {
	use super::*;

	#[test]
	fn mem_read_and_write_raw() {
		let mut mem: Memory = Memory::new();

		mem.write_byte_raw(0x8004, 0x12);
		mem.write_byte_raw(0x8005, 0x34);
		assert_eq!(mem.read_word_raw(0x8004), 0x1234);

		mem.write_word_raw(0x8006, 0x5678);
		assert_eq!(mem.read_byte_raw(0x8006), 0x56);
		assert_eq!(mem.read_byte_raw(0x8007), 0x78);
	}
}