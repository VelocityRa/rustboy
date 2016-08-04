//
//		Central Processing Unit
//

#![allow(dead_code)]

pub mod instructions;

use mmu::Memory;
use emulator::Emulator;
use timer::Timer;

// CPU Clock speed
pub const CLOCK_SPEED: f64 = 4.194304; // MHz

// Clock cycles between every screen refresh 
pub const SCREEN_REFRESH_INTERVAL: u32 = 70224; // clock cycles

#[allow(dead_code)]
pub enum Interrupt {
    Vblank  = 0x01,
    LCDStat = 0x02,
    Timer   = 0x04,
    Serial  = 0x08,
    Joypad  = 0x10,
}

// Tell the compiler to generate a default() function
// Which zero initializes everything

// Z80 registers
#[derive(Default)]
pub struct Registers  {
	a: u8,		// A: Accumulator
	flags: Flags,		// Flags
	b: u8,
	c: u8,		// BC: General purpose
	d: u8,
	e: u8,		// DE: General purpose
	h: u8,
	l: u8,		// HL: General purpose
	sp: u16,			// SP: Stack pointer
	pc: u16,			// PC: Program counter
}


impl Registers {
	pub fn bc(&self) -> u16 { (self.b as u16) << 8 | self.c as u16 }
	pub fn de(&self) -> u16 { (self.d as u16) << 8 | self.e as u16 }
	pub fn hl(&self) -> u16 { (self.h as u16) << 8 | self.l as u16 }

	pub fn bc_set(&mut self, new: u16){ self.b = (new >> 8) as u8; self.c = new as u8; }
	pub fn de_set(&mut self, new: u16){ self.d = (new >> 8) as u8; self.e = new as u8; }
	pub fn hl_set(&mut self, new: u16){ self.h = (new >> 8) as u8; self.l = new as u8; }

	#[inline]
	pub fn bump(&mut self) -> u16 {
		let ret = self.pc;
		self.pc += 1;
		// Could just a return ret++ work here?
		return ret;
	}
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


pub struct Cpu {
	regs: Registers,
	timer: Timer,
	total_cycles: u32,
}

impl Cpu {
	pub fn new() -> Cpu {
		let mut cpu: Cpu = Cpu {
			regs: Default::default(),
			timer: Timer::new(),
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
		self.regs.bc_set(0x0013);
		self.regs.de_set(0x00D8);
		self.regs.hl_set(0x014D);
		self.regs.sp = 0xFFFE;
		self.regs.pc = 0x0150;
	}

	pub fn get_regs(&self) -> &Registers {
		&self.regs
	}
	pub fn get_flags(&self) -> &Flags {
		&self.regs.flags
	}
	pub fn get_regs_mut(&mut self) -> &mut Registers {
		&mut self.regs
	}
	pub fn get_flags_mut(&mut self) -> &mut Flags {
		&mut self.regs.flags
	}


	pub fn update_timers(&mut self, mem: &mut Memory) {
		/*
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
*/
		//println!("d:{:02X} \t c:{:02X}",  self.timers.div_reg, self.timers.counter);  Memory-map the timers


		// TODO: Handle in memory mapping instead
		//mem.write_byte(0xFF04, self.timers.div_reg);
		//mem.write_byte(0xFF05, self.timers.counter);
	}

	// Dispatcher
	pub fn run(&mut self, mem: &mut Memory) {
		while self.total_cycles < SCREEN_REFRESH_INTERVAL {
			let op: u8 = mem.rb(self.regs.pc);
			
			println!("pc:{:04X}, op:{:02X}", self.regs.pc, op);
			
			let time = instructions::exec(op, &mut self.regs, mem);

			self.total_cycles += time;

			self.regs.pc += 1;

		}

		// Should this get reset or should we modulo above
		// Not sure yet...
		self.total_cycles = 0;
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

		assert_eq!(regs.pc, 0x0150);

		regs.sp = 123;
		assert_eq!(regs.sp, 123);

		regs.h = 3;
		regs.l = 0;
		assert_eq!(regs.h, 3);
		assert_eq!(regs.hl(), 0b00000011_00000000);
	}

	#[test]
	fn flag_get_and_set() {
		let mut cpu = Cpu::new();

		// Get mutable reference
		let mut flags = cpu.get_flags_mut();

		flags.zf.set();
		flags.n.unset();
		flags.h.set();
		flags.cy.unset();
		
		assert_eq!(flags.zf.get(), 	true);
		assert_eq!(flags.n.get(), 	false);
		assert_eq!(flags.h.get(), 	true);
		assert_eq!(flags.cy.get(), 	false);
	}
}
