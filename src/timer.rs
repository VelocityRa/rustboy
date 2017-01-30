
use cpu::Interrupt;
use std::fmt;

#[allow(dead_code)]
#[allow(unused_variables)]

const DIV_AFTER_BIOS: u16 = 0xABCC;

pub struct Timer {
    // This register is incremented at rate of 16384Hz
    // Writing any value to this register resets it to 00h
    pub div: u16,
    // This timer is incremented by a clock frequency specified by the TAC register ($FF07)
    // When the value overflows (gets bigger than FFh) then it will be reset to the
    // value specified in TMA (FF06), and an interrupt will be requested
    pub tima: u8,

    pub tma: u8,
    pub tac: u8,

    tima_speed: u32,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            div: DIV_AFTER_BIOS,
            tima: 0,
            tma: 0,
            tac: 0,
            tima_speed: 256
        }
    }

    pub fn update(&mut self) {
        // See step() function for timings
        match self.tac & 0x3 {
            0x0 => { self.tima_speed = 256; }
            0x1 => { self.tima_speed = 4; }
            0x2 => { self.tima_speed = 16; }
            0x3 => { self.tima_speed = 64; }
            _ => {}
        }
    }

    pub fn step(&mut self, ticks: u32, if_: &mut u8) {
        self.div = self.div.wrapping_add(1);
        if (self.tac & 0b100) != 0 {
            // Check for 8-bit overflow
            if ((self.tima as u16) + 1) & 0xFF == 0 {
                self.tima = self.tma;

                // Fire Timer interrupt
                *if_ |= Interrupt::Timer as u8;
            } else {
                // Increment TIMA
                self.tima += 1;
            }
        }
    }

    pub fn reset_bios_skip(&mut self) {
        self.div = DIV_AFTER_BIOS
    }
}


impl fmt::Debug for Timer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, " div: {}\n tima: {}\n tma: {}\n tac: {}\n tima_speed: {}
            ",
            self.div,
            self.tima,
            self.tma,
            self.tac,
            self.tima_speed,
            )
    }
}