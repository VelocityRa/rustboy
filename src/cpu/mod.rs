//
//		Central Processing Unit
//

#![allow(dead_code)]

pub mod instructions;

use std::fmt;

use mmu::Memory;
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
    pub ime: u32,
	halt: bool,
	stop: bool,

	a: u8,		// A: Accumulator
	b: u8,
	c: u8,		// BC: General purpose
	d: u8,
	e: u8,		// DE: General purpose
	h: u8,
	l: u8,		// HL: General purpose

	f: Flags,		// Flags

	sp: u16,			// SP: Stack pointer
	pc: u16,			// PC: Program counter

	delay: u32
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

	// Interrupts
	pub fn int_step(&mut self) {
        match self.delay {
            1 => { self.delay = 0; self.ime = 1; }
            2 => { self.delay = 1; }
            _ => return
        }
		debug!("Interrupt delay: {}", self.delay);
    }

    // Schedule enabling of interrupts
    pub fn ei(&mut self, m: &mut Memory) {
        if self.delay == 2 || m.rb(self.pc) == 0x76 {
            self.delay = 1;
        } else {
            self.delay = 2;
        }
		info!("Enable interrupts, delay: {}", self.delay);
    }

    pub fn di(&mut self) {
    	info!("Disable interrupts");
        self.ime = 0;
        self.delay = 0;
    }

	// Instructions

	fn hlpp(&mut self) {
		self.l += 1;
		if self.l == 0 {
			self.h += 1;
		}
	}

	fn hlmm(&mut self) {
		self.l -= 1;
		if self.l == 0xff {
			self.h -= 1;
		}
	}

	fn ret(&mut self, m: &Memory) {
		self.pc = m.rw(self.sp);
		self.sp += 2;
		debug!("RET to {:04X}", self.pc);
	}

	fn inc_hlm(&mut self, m: &mut Memory) {
		self.f.n.unset();
		let hl = self.hl();
		let v = m.rb(hl) + 1;
		m.wb(hl, v);
		if v == 0 {self.f.z.set()} else {self.f.z.unset()};
		if v & 0xF == 0 {self.f.h.set()} else {self.f.h.unset()};
	}

	fn dec_hlm(&mut self, m: &mut Memory) {
		self.f.n.set();
		let hl = self.hl();
		let v = m.rb(hl) - 1;
		m.wb(hl, v);
		if v == 0 {self.f.z.set()} else {self.f.z.unset()};
		if v & 0xF == 0xF {self.f.h.set()} else {self.f.h.unset()};
	}
}

impl fmt::Debug for Registers {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "PC: {:04X}  SP: {:04X}
			A: {:02X}
			B: {:02X}  AB: {:04X}
			C: {:02X}
			D: {:02X}  DE: {:04X}
			E: {:02X}
			H: {:02X}  HL: {:04X}
			L: {:02X}
			",
			self.pc, self.sp,
			self.a,
			self.b, self.bc(),
			self.d,
			self.e, self.de(),
			self.h,
			self.l, self.hl(),
			self.l,
		)
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
	#[inline]
	pub fn set_if(&mut self, cond: bool) {
		if cond {self.set()} else {self.unset()};
	}
}

impl fmt::Display for Flag {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.value) }
}

// Set everything to zero/false
#[derive(Default)]
// Pack the bools like a bitfield
#[repr(C, packed)]
pub struct Flags {
	z: Flag,		// Zero Flag
	n: Flag,		// Add/Sub-Flag (BCD)
	h: Flag,		// Half Carry Flag (BCD)
	c: Flag,		// Carry Flag

	unused1: bool,	// Unused (always 0)
	unused2: bool,	// Unused (always 0)
	unused3: bool,	// Unused (always 0)
	unused4: bool,	// Unused (always 0)
}

impl Flags {
	pub fn reset(&mut self) {
		self.z.unset();
		self.n.unset();
		self.h.unset();
		self.c.unset();
	}
}


impl fmt::Debug for Flags {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Z: {}
			N: {}
			H: {}
			C: {}
			",
			self.z,
			self.n,
			self.h,
			self.c,
		)
	}
}


pub struct Cpu {
	regs: Registers,
	timer: Timer,
	total_cycles: u32,

	pub is_running: bool,
	pub is_stepping: bool
}

impl Cpu {
	pub fn new() -> Cpu {
		let mut cpu: Cpu = Cpu {
			regs: Default::default(),
			timer: Timer::new(),
			total_cycles: 0,
			is_running: true,
			is_stepping: false,
		};
		cpu.reset_state();
		// Runs for just 1 instruction every run() call (for debugging)
		cpu.is_stepping = true;

		cpu
	}

	// Power Up Sequence
	pub fn reset_state(&mut self) {
		self.regs.a = 0x01;
		self.regs.f.z.set();
		self.regs.f.h.set();
		self.regs.f.c.set();
		self.regs.bc_set(0x0013);
		self.regs.de_set(0x00D8);
		self.regs.hl_set(0x014D);
		self.regs.sp = 0xFFFE;
		self.regs.pc = 0x0100;
	}

	pub fn get_regs(&self) -> &Registers {
		&self.regs
	}
	pub fn get_flags(&self) -> &Flags {
		&self.regs.f
	}
	pub fn get_regs_mut(&mut self) -> &mut Registers {
		&mut self.regs
	}
	pub fn get_flags_mut(&mut self) -> &mut Flags {
		&mut self.regs.f
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
			// Interrupt step
			self.regs.int_step();

			// Fetch opcode
			let op: u8 = mem.rb(self.regs.pc);
		
			//debug!("PC:{:04X}, OP:{:02X}", self.regs.pc, op);
			let pc_before = self.regs.pc;

			// Increment PC
			self.regs.pc += 1;
			
			// Execute instruction
			let cycles = instructions::exec(op, &mut self.regs, mem);

			//println!("{} {}", self.regs.pc, pc_before);

			if op != 0x00 {
				match self.regs.pc as i32 - pc_before as i32 {
					1 => println!("[0x{:08X}] 0x{:02X}", pc_before, op),
					2 => println!("[0x{:08X}] 0x{:02X} 0x{:02X}",
						pc_before, op, mem.rb(pc_before + 1)),
					3 => println!("[0x{:08X}] 0x{:02X} 0x{:02X} 0x{:02X}",
						pc_before, op, mem.rb(pc_before + 1), mem.rb(pc_before + 2)),
					_ => {}//println!("Call or jump from 0x{:04X} to 0x{:04X}", pc_before, self.regs.pc),
				};
			};
			
			if self.regs.stop {self.stop(); return}
			self.total_cycles += cycles * 4;

			if self.is_stepping { return; }

			//debug!("Cycles: {}", self.total_cycles);

		}

		// Should this get reset or should we modulo above
		// Not sure yet...
		self.total_cycles = 0;
	}
}

impl fmt::Debug for Cpu {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "State: {}
			Cycles: {}",
			if self.is_running {"Running"} else {"Paused"},
			self.total_cycles,
			)
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

		assert_eq!(regs.pc, 0x0100);

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

		flags.z.set();
		flags.n.unset();
		flags.h.set();
		flags.c.unset();
		
		assert_eq!(flags.z.get(), 	true);
		assert_eq!(flags.n.get(), 	false);
		assert_eq!(flags.h.get(), 	true);
		assert_eq!(flags.c.get(), 	false);
	}
}
