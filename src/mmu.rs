//
//		Memory Management Unit 
//

/*

  0000-3FFF   16KB ROM Bank 00     (in cartridge, fixed at bank 00)
  4000-7FFF   16KB ROM Bank 01..NN (in cartridge, switchable bank number)
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

const MEM_SIZE: usize = 0xFFFF;

pub struct Memory {
	raw_mem: [u8; MEM_SIZE],
}

impl Memory {
	// Allocate a 64k byte array and zero initialize it
	// This is all the system's RAM
	pub fn new() -> Memory {
		Memory {
			raw_mem: [0u8; MEM_SIZE],
		}
	}

	pub fn read_byte(&self, addr: u16) -> u8 {
		assert!(addr <= 0xFFFF,
		 "Invalid memory read: {:04X}", addr);
		let addr: usize = addr as usize;

		self.raw_mem[addr]
	}
	pub fn read_word(&self, addr: u16) -> u16 {
		assert!(addr <= 0xFFFF - 1,
		 "Invalid memory read: {:04X}", addr);
		let addr: usize = addr as usize;

		(self.raw_mem[addr] as u16) << 8 |
		(self.raw_mem[addr + 1] as u16)
	}

	pub fn write_byte(&mut self, addr: u16, data: u8) {
		assert!(addr <= 0xFFFF,
		 "Invalid memory write: {:04X}", addr);
		let addr: usize = addr as usize;

		self.raw_mem[addr] = data
	}

	pub fn write_word(&mut self, addr: u16, data: u16) {
		assert!(addr <= 0xFFFF - 1,
		 "Invalid memory write: {:04X}", addr);
		let addr: usize = addr as usize;

		self.raw_mem[addr] = (data >> 8) as u8;
		self.raw_mem[addr + 1] = (data & 0x00FF) as u8;
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