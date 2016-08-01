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

const START_MAPPED_MEM: usize = 0x8000;
const MEM_SIZE: usize = 0xFFFF + 1 - START_MAPPED_MEM;

pub struct Memory {
	raw_mem: [u8; MEM_SIZE],
	pub rom_loaded: Vec<u8>,
}

impl Memory {
	// Allocate a 64k byte array and zero initialize it
	// This is all the system's RAM
	pub fn new() -> Memory {
		Memory {
			raw_mem: [0u8; MEM_SIZE],
			rom_loaded: Vec::new(),
		}
	}

	pub fn set_rom(&mut self, rom: Vec<u8>) {
		self.rom_loaded = rom;
	}

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

	pub fn read_byte(&self, addr: u16) -> u8 {
		self.debug_print_addr(addr, true);

		match addr {
			0x0000 ... 0x7FFF => self.rom_loaded[addr as u16 as usize],
			0xFEA0 ... 0xFEFF => panic!("Unusable memory accessed"),
			_ => self.read_byte_raw(addr)
		}
	}

	pub fn read_word(&self, addr: u16) -> u16 {
		assert!(addr <= 0xFFFF - 1,
		 "Invalid memory read: {:04X}", addr);

		(self.read_byte(addr) as u16) << 8 |
		(self.read_byte(addr + 1) as u16)
	}

	pub fn write_byte(&mut self, addr: u16, data: u8) {
		self.debug_print_addr(addr, false);

		// TODO match addr {}
		self.write_byte_raw(addr, data);
	}

	pub fn write_word(&mut self, addr: u16, data: u16) {
		assert!(addr <= 0xFFFF - 1,
		 "Invalid memory write: {:04X}", addr);

		self.write_byte(addr, (data >> 8) as u8);
		self.write_byte(addr + 1, (data & 0x00FF) as u8);
	}

	fn debug_print_addr(&self, addr: u16, is_reading: bool) {
		println!("{} {:04X} in {}", if is_reading {"Read from"} else {"Write to"}, addr,
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
	fn mem_read_and_write() {
		let mut mem: Memory = Memory::new();

		mem.write_byte(0x0004, 0x12);
		mem.write_byte(0x0005, 0x34);
		assert_eq!(mem.read_word(0x0004), 0x1234);

		mem.write_word(0x0006, 0x5678);
		assert_eq!(mem.read_byte(0x0006), 0x56);
		assert_eq!(mem.read_byte(0x0007), 0x78);
	}
}