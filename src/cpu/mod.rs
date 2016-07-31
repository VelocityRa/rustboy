//
//		Central Processing Unit
//

#![allow(dead_code)]

pub use self::opcodes::opcode_map;
pub mod opcodes;
pub mod instructions;

use mmu::Memory;
use emulator::Emulator;

// CPU Clock speed
pub const CLOCK_SPEED: f64 = 4.194304; // MHz

// Clock cycles between every screen refresh 
pub const SCREEN_REFRESH_INTERVAL: u32 = 70244; // clock cycles

// Tell the compiler to generate a default() function
// Which zero initializes everything
#[derive(Default)]
struct Register {
	high: u8,
	low: u8,
}

impl Register {
	pub fn get_both(&self) -> u16 {
		return (self.high as u16) << 8 | self.low as u16;
	}
	pub fn set_both(&mut self, val: u16) {
		self.high = ((val & 0xFF00) >> 8 ) as u8;
		self.low = (val & 0x00FF) as u8;
	}
}

// Z80 registers
#[derive(Default)]
pub struct Registers {
	a: u8,			// A: Accumulator
	flags: Flags,	// Flags
	bc: Register,		// BC: General purpose
	de: Register,		// DE: General purpose
	hl: Register,		// HL: General purpose
	sp: u16	,			// SP: Stack pointer
	pc: u16,			// PC: Program counter
}


#[derive(Default)]
pub struct Flag {
	value: bool,
}

impl Flag {
	pub fn get(&self) -> bool {
		self.value
	}
	pub fn set(&mut self) {
		self.value = true;
	}
	pub fn unset(&mut self) {
		self.value = false;
	}
	pub fn toggle(&mut self) {
		self.value = !self.value;
	}
}

// Set everything to zero/false
#[derive(Default)]
// Pack the bools like a bitfield
#[repr(C, packed)]
pub struct Flags {
	zf: Flag,		// Zero Flag
	n: 	Flag,		// Add/Sub-Flag (BCD)
	h: 	Flag,		// Half Carry Flag (BCD)
	cy: Flag,		// Carry Flag

	unused1: bool,	// Unused (always 0)
	unused2: bool,	// Unused (always 0)
	unused3: bool,	// Unused (always 0)
	unused4: bool,	// Unused (always 0)
}

#[derive(Default)]
pub struct Timers {
	// This register is incremented at rate of 16384Hz
	// Writing any value to this register resets it to 00h
	div_reg: u8,

	// This timer is incremented by a clock frequency specified by the TAC register ($FF07)
	// When the value overflows (gets bigger than FFh) then it will be reset to the 
	// value specified in TMA (FF06), and an interrupt will be requested
	counter: u8,
}

pub struct Cpu {
	regs: Registers,
	timers: Timers,
	total_cycles: u32,
}

impl Cpu {
	pub fn new() -> Cpu {
		let mut cpu: Cpu = Cpu {
			regs: Default::default(),
			timers: Default::default(),
			total_cycles: 0,
		};
		cpu.reset_state();
		cpu
	}

	// Power Up Sequence
	pub fn reset_state(&mut self) {
		self.regs.a = 0x01;
		self.regs.flags.zf.set();
		self.regs.flags.h.set();
		self.regs.flags.cy.set();
		self.regs.bc.set_both(0x0013);
		self.regs.de.set_both(0x00D8);
		self.regs.hl.set_both(0x014D);
		self.regs.sp = 0xFFFE;
		self.regs.pc = 0x0000;		
	}

	pub fn get_regs_mut(&mut self) -> &mut Registers {
		&mut self.regs
	}
	pub fn get_flags_mut(&mut self) -> &mut Flags {
		&mut self.regs.flags
	}

	pub fn update_timers(&mut self, dt: f64, mem: &mut Memory) {
		use std::i32;

		// This register is incremented at rate of 16384Hz
		self.timers.div_reg = 
			self.timers.div_reg.wrapping_add(
				((dt * 16384.0) as u64 % 256) as u8
			);

		// TODO: Load correct incrementation rate
		let (new_counter, counter_overflowed) = 
			self.timers.counter.overflowing_add(
				((dt * 16384.0) as u64 % 256) as u8
			);

		self.timers.counter = new_counter;
		if counter_overflowed {
			// Read value from TMA - Timer Modulo
			self.timers.counter = mem.read_byte(0xFF06); 
		}

		//println!("d:{:02X} \t c:{:02X}", self.timers.div_reg, self.timers.counter);
		mem.write_byte(0xFF04, self.timers.div_reg);
		mem.write_byte(0xFF05, self.timers.counter);
	}

	// Dispatcher
	pub fn run(&mut self, mem: &mut Memory) {
		
		while self.total_cycles < SCREEN_REFRESH_INTERVAL {
			let op: u8 = mem.read_byte(self.regs.pc);
			
			if op != 0 { println!("{:02X}: {:02X}", self.regs.pc, op)};
			
			opcode_map[op as usize](self);

			self.total_cycles += 4;
			self.regs.pc += 4;

		}

		self.total_cycles = SCREEN_REFRESH_INTERVAL;
	}
}

//	======================================
//	|               TESTS                |
//	======================================

#[cfg(test)]
mod cpu_tests {
	use super::*;

	#[test]
	fn reg_get_and_set() {
		let mut cpu = Cpu::new();

		// Get mutable reference
		let mut regs = cpu.get_regs_mut();

		assert_eq!(regs.pc.get_both(), 0);

		regs.sp.set_both(123);
		assert_eq!(regs.sp.get_both(), 123);

		regs.hl.high = 3;
		assert_eq!(regs.hl.high, 3);
		assert_eq!(regs.hl.get_both(), 0b00000011_00000000);
	}

	#[test]
	fn flag_get_and_set() {
		let mut cpu = Cpu::new();

		// Get mutable reference
		let mut flags = cpu.get_flags_mut();

		flags.zf.set();
		flags.h.set();
		
		assert_eq!(flags.zf.get(), 	true);
		assert_eq!(flags.n.get(), 	false);
		assert_eq!(flags.h.get(), 	true);
		assert_eq!(flags.cy.get(), 	false);
	}
}
