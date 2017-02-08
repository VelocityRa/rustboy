//
//      Central Processing Unit
//

#![allow(dead_code)]

pub mod instructions;

use std::str;
use std::fmt;
use std::io::prelude::*;
use std::fs::{OpenOptions, File};

use colored::*;
use mmu::Memory;


// CPU Clock speed
// TODO: Disable if log level > TRACE
pub const INSTR_DEBUG: bool = false;    // very laggy, needs 'trace' log level
pub const WADATSUMI_DEBUG: bool = true;    // Format debug text the same way Wadatsume does (for easy comparison)

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
    pub ime: bool,
    halt: bool,
    pub stop: bool,

    a: u8,      // A: Accumulator
    b: u8,
    c: u8,      // BC: General purpose
    d: u8,
    e: u8,      // DE: General purpose
    h: u8,
    l: u8,      // HL: General purpose

    f: Flags,   // Flags

    sp: u16,    // SP: Stack pointer
    pc: u16,    // PC: Program counter

    delay: u32
}


impl Registers {
    fn f(&self) -> u8 {
        ((self.f.z.value as u8) << 7) | (self.f.n.value as u8) << 6
        |(self.f.h.value as u8) << 5 | (self.f.c.value as u8) << 4
    }
    pub fn af(&self) -> u16 { (self.a as u16) << 8 | self.f() as u16 }
    pub fn bc(&self) -> u16 { (self.b as u16) << 8 | self.c as u16 }
    pub fn de(&self) -> u16 { (self.d as u16) << 8 | self.e as u16 }
    pub fn hl(&self) -> u16 { (self.h as u16) << 8 | self.l as u16 }

    pub fn af_set(&mut self, new: u16){ self.a = (new >> 8) as u8;
        self.f.z.set_if(new & 0x80 != 0); self.f.n.set_if(new & 0x40 != 0);
        self.f.h.set_if(new & 0x20 != 0); self.f.c.set_if(new & 0x10 != 0);}
    pub fn bc_set(&mut self, new: u16){ self.b = (new >> 8) as u8; self.c = new as u8; }
    pub fn de_set(&mut self, new: u16){ self.d = (new >> 8) as u8; self.e = new as u8; }
    pub fn hl_set(&mut self, new: u16){ self.h = (new >> 8) as u8; self.l = new as u8; }

    pub fn pc(&self) -> u16 { self.pc }

    #[inline]
    pub fn bump(&mut self) -> u16 {
        let ret = self.pc;
        self.pc += 1;
        // Could just a return ret++ work here?
        return ret;
    }

    // Update IME (for interrupts)
    pub fn int_step(&mut self) {
        match self.delay {
            1 => { self.delay = 0; self.ime = true; }
            2 => { self.delay = 1; }
            _ => return
        }
        debug!("Interrupt delay: {}", self.delay);
    }

    // Schedule enabling of interrupts
    pub fn ei(&mut self, m: &mut Memory) {
        if self.delay == 2 || m.rb(self.pc) == 0x76 {   // 0x76 == HALT
            self.delay = 1;
        } else {
            self.delay = 2;
        }
        info!("Enable interrupts, delay: {}", self.delay);
    }

    pub fn di(&mut self) {
        info!("Disable interrupts");
        self.ime = false;
        self.delay = 0;
    }

    // Instructions

    fn inc_hl(&mut self) {
        self.l = self.l.wrapping_add(1);
        if self.l == 0 {
            self.h = self.h.wrapping_add(1);
        }
    }

    fn dec_hl(&mut self) {
        self.l = self.l.wrapping_sub(1);
        if self.l == 0xff {
            self.h = self.h.wrapping_sub(1);
        }
    }

    fn ret(&mut self, m: &mut Memory) {
        self.pc = m.rw(self.sp);
        debug!("RET to {:04X}", self.pc);
        self.sp += 2;
    }

    fn inc_hlm(&mut self, m: &mut Memory) {
        self.f.n.unset();
        let hl = self.hl();
        let v = m.rb(hl).wrapping_add(1);
        m.wb(hl, v);
        if v == 0 {self.f.z.set()} else {self.f.z.unset()};
        if v & 0xF == 0 {self.f.h.set()} else {self.f.h.unset()};
    }

    fn dec_hlm(&mut self, m: &mut Memory) {
        self.f.n.set();
        let hl = self.hl();
        let v = m.rb(hl).wrapping_sub(1);
        m.wb(hl, v);
        if v == 0 {self.f.z.set()} else {self.f.z.unset()};
        if v & 0xF == 0xF {self.f.h.set()} else {self.f.h.unset()};
    }

    fn add_hlsp(&mut self) {
        let hl = self.hl() as u32;
        let s = hl + self.sp as u32;
        self.f.h.set_if(hl & 0xfff > s & 0xfff);
        self.f.c.set_if(s > 0xffff);
        self.f.n.unset();
        self.h = (s >> 8) as u8;
        self.l = s as u8;
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
pub struct Flags {
    z: Flag,        // Zero Flag
    n: Flag,        // Add/Sub-Flag (BCD)
    h: Flag,        // Half Carry Flag (BCD)
    c: Flag,        // Carry Flag
}

impl Flags {
    pub fn reset(&mut self) {
        self.z.unset();
        self.n.unset();
        self.h.unset();
        self.c.unset();
    }
}

// Converts a flat to either its symbol (like 'z' or 'c'),
// or the character '-'. Used for debugging in exec().
macro_rules! flag_to_ch (
    ($sel:ident, $fl:ident, $ch:expr) => ({
        if $sel.regs.f.$fl.value {$ch} else {'-'}
    })
);

// Interrupt handlers
macro_rules! rst (
($sel:ident, $mem:ident, $isr:expr) => ({
    $sel.regs.ime = false;
    $sel.regs.sp -= 2;
    $mem.ww($sel.regs.sp, $sel.regs.pc);
    $sel.regs.pc = $isr;
    $sel.total_cycles += 12;
}) );

pub struct Cpu {
    regs: Registers,

    pub total_cycles: u32,
    pub is_running: bool,
    trace_file: Option<File>,
}

impl Cpu {
    pub fn new() -> Cpu {
        let mut cpu: Cpu = Cpu {
            regs: Default::default(),
            total_cycles: 0,
            is_running: true,
            trace_file: None,
        };
        cpu.trace_file = Some(OpenOptions::new()
                            .create(true)
                            .write(true)
                            .truncate(true)
                            .open("trace_log.txt")
                            .unwrap());

        cpu.reset_state();
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

    // Dispatcher
    // Executes 1 instruction
    pub fn exec(&mut self, mem: &mut Memory) -> u32 {

        // Interrupts
        if self.handle_interrupts(mem) {return 0};

        // Fetch opcode
        let op: u8 = mem.rb(self.regs.pc);

        // Save previous pc
        let pc_before = self.regs.pc;

        if WADATSUMI_DEBUG {
            let ff44 = mem.rb(0xff44);
            // let line = format!("PC[0x{:02X}]: 0x{:04X} AF: 0x{:04X} BC: 0x{:04X} DE: 0x{:04X} HL: 0x{:04X} SP: 0x{:04X} IE: {:08b} IF: {:08b} DIV: {} LY: {:02X}\n",
            let line = format!("PC[0x{:02X}]: 0x{:04X} AF: 0x{:04X} BC: 0x{:04X} DE: 0x{:04X} HL: 0x{:04X} SP: 0x{:04X} IE: {:08b} IF: {:08b}\n",
                     //self.total_cycles,
                     //op.format(&self.ctx, bus).unwrap(),
                     op,
                     self.regs.pc,
                     self.regs.af(),
                     self.regs.bc(),
                     self.regs.de(),
                     self.regs.hl(),
                     self.regs.sp,
                      mem.ie_,
                      mem.if_,
                    //  mem.timer.div,
                    //  ff44
            );

            let trace_file = self.trace_file.as_mut().unwrap();
            trace_file.write_all(line.as_bytes()).unwrap();
        }

        // OAM DMA
        if mem.is_dma {
            mem.handle_dma_transfer();
        }

        // HALT
        if self.regs.halt {
            if mem.ie_ & mem.if_ != 0 {
                self.regs.halt = false;
            }
        }
        if self.regs.halt {
            return 4;
        }

        // Increment PC
        self.regs.pc += 1;

        // Execute instruction
        let cycles = instructions::exec(op, &mut self.regs, mem) * 4;

        if INSTR_DEBUG {
            let pc_diff = self.regs.pc as i32 - pc_before as i32;

            let addr_and_instr =
                match pc_diff {
                    1 => format!("[0x{:04X}] 0x{:02X}          ",
                        pc_before, op),
                    2 => format!("[0x{:04X}] 0x{:02X} 0x{:02X}     ",
                        pc_before, op, mem.rb(pc_before + 1)),
                    3 => format!("[0x{:04X}] 0x{:02X} 0x{:02X} 0x{:02X}",
                        pc_before, op, mem.rb(pc_before + 1), mem.rb(pc_before + 2)),
                    _ => format!("[0x{:04X}] 0x{:02X} (JUMP)   ",
                        pc_before, op),
                           //print!("Jump offset: {}", self.regs.pc as i32
                           //                        - pc_before as i32,

                };
            macro_rules! y (
                ($v:expr) => ($v.yellow())
            );
            macro_rules! o (
                ($v:expr) => ($v.red())
            );

            let regs_and_flags = format!("\t\t{} {:02X}   {} {:02X} {:02X}   {} {:02X} {:02X}   {} {:02X} {:02X}   {} {:04X}   {} {}{}{}{}",
                y!("A:"), self.regs.a,
                y!("BC:"), self.regs.b, self.regs.c,
                y!("DE:"), self.regs.d, self.regs.e,
                y!("HL:"), self.regs.h ,self.regs.l,
                y!("SP:"), self.regs.sp,
                o!("FLAGS: "),
                flag_to_ch!(self, z,'z'), flag_to_ch!(self, n,'n'),
                flag_to_ch!(self, h,'h'), flag_to_ch!(self, c,'c'));
            trace!("{}{}", addr_and_instr, regs_and_flags);
        };

        self.total_cycles += cycles;

        //debug!("Cycles: {}", self.total_cycles);

        return cycles;
    }

    fn handle_interrupts(&mut self, mem: &mut Memory) -> bool {
        self.regs.int_step();
        let interrupts = mem.ie_ & mem.if_;
            macro_rules! print_interrupt (
            ($s:expr) => {
                warn!("{} IF: {:#08b}", $s.magenta(), mem.if_);
            } );

        if self.regs.ime && (interrupts != 0) {
            // Vertical blank (ISR: 40 )
            if interrupts & Interrupt::Vblank as u8 != 0 {
                mem.if_ &= !(Interrupt::Vblank as u8);
                rst!(self, mem, 0x40);
                print_interrupt!("VBLANK");
            }
            // LCD status triggers (ISR: 48 )
            if interrupts & Interrupt::LCDStat as u8 != 0 {
                mem.if_ &= !(Interrupt::LCDStat as u8);
                rst!(self, mem, 0x48);
                print_interrupt!("LCD status triggers");
            }
            // Timer overflow (ISR: 50 )
            if interrupts & Interrupt::Timer as u8 != 0 {
                mem.if_ &= !(Interrupt::Timer as u8);
                rst!(self, mem, 0x50);
                print_interrupt!("Timer overflow");
            }
            // Serial link (ISR: 58 )
            if interrupts & Interrupt::Serial as u8 != 0 {
                mem.if_ &= !(Interrupt::Serial as u8);
                rst!(self, mem, 0x58);
                print_interrupt!("Serial link");
            }
            // LCD status triggers (ISR: 60 )
            if interrupts & Interrupt::Joypad as u8 != 0 {
                mem.if_ &= !(Interrupt::Joypad as u8);
                rst!(self, mem, 0x60);
                print_interrupt!("Joypad press");
            }
            return true;
        }
        return false;
    }

}

impl fmt::Debug for Registers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
" PC: {:04X}  SP: {:04X}
 A: {:02X}
 B: {:02X}  BC: {:04X}
 C: {:02X}
 D: {:02X}  DE: {:04X}
 E: {:02X}
 H: {:02X}  HL: {:04X}
 L: {:02X}",
            self.pc, self.sp,
            self.a,
            self.b, self.bc(),
            self.c,
            self.d, self.de(),
            self.e,
            self.h, self.hl(),
            self.l,
        )
    }
}

impl fmt::Debug for Flags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
" Z: {}
 N: {}
 H: {}
 C: {}",
            self.z,
            self.n,
            self.h,
            self.c,
        )
    }
}


//  ======================================
//  |               TESTS                |
//  ======================================

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

        assert_eq!(flags.z.get(),   true);
        assert_eq!(flags.n.get(),   false);
        assert_eq!(flags.h.get(),   true);
        assert_eq!(flags.c.get(),   false);
    }
}
