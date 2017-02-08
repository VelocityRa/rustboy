//
//      Memory Management Unit
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

use piston::window::Window;
use piston_window::PistonWindow;

use timer::Timer;
use gpu::Gpu;
use gpu;

#[derive(PartialEq, Eq, Debug)]
enum Mbc {
    RomOnly,
    Mbc1,
    Mbc2,
    Mbc3,
    Mbc4,
    Mbc5,
    Unknown,
}

const MEM_SIZE: usize = 0xFFFF + 1;

pub struct Memory {
    // Interrupt flags, http://problemkaputt.de/pandocs.htm#interrupts
    // The master enable flag will be on the cpu
    pub if_: u8,
    pub ie_: u8,

    raw_mem: Box<[u8; MEM_SIZE]>,

    pub rom_loaded: Vec<u8>,

    pub timer: Box<Timer>,
    pub gpu: Box<Gpu>,

    mbc: Mbc,
    cart_type: u8,
    enable_ext_ram: bool,
    is_ram_mode: bool,   // true -> RAM expansion mode, else ROM mode
    rom_bank: u8,
    rom_offset: u16,
    ram_bank: u8,
    ram_offset: u16,

    // OAM DMA stuff
    pub is_dma: bool,
    dma_left: usize,
    dma_value: u8,
}

impl Memory {
    // Allocate a 64k byte array and zero initialize it
    // This is all the system's RAM
    pub fn new<W: Window>(window: &PistonWindow<W>) -> Memory {
        let mut mem = Memory {
            if_: 1u8,
            ie_: 0u8,
            raw_mem: Box::new([0u8; MEM_SIZE]),
            rom_loaded: Vec::new(),

            timer: Box::new(Timer::new()),
            gpu: Box::new(Gpu::new(window)),

            mbc: Mbc::Unknown,
            cart_type: 0,
            enable_ext_ram: false,
            is_ram_mode: false,
            rom_bank: 0,
            rom_offset: 0x4000,
            ram_bank: 0,
            ram_offset: 0x0000,

            is_dma: false,
            dma_left: 0,
            dma_value: 0,
        };
        mem.power_on();
        mem.timer.reset_bios_skip();

        mem
    }

    // Copies VRAM from rom to the gpu's vrambanks
    pub fn copy_vram(&mut self) {
        // TODO: Make faster?
        const VRAM_START: u16 = 0x8000;
        const VRAM_SIZE: u16 = 8 << 10; // 8K;
        for addr in 0..VRAM_SIZE {
            self.gpu.vrambank[addr as usize] = self.rom_loaded[(addr + VRAM_START) as usize];
            //self.gpu.vrambanks[1][addr as usize] = self.rom_loaded[(addr + VRAM_EXT_START) as usize];
        }
    }

    pub fn copy_rom(&mut self) {
        // Copy ROM data to memory
        self.raw_mem[0x0000..0x7FFF].copy_from_slice(&self.rom_loaded[0x0000..0x7FFF]);
    }

    pub fn power_on(&mut self) {
        // From http://problemkaputt.de/pandocs.htm#powerupsequence
        self.wb(0xff05, 0x00); // TIMA
        self.wb(0xff06, 0x00); // TMA
        self.wb(0xff07, 0x00); // TAC
        self.wb(0xff10, 0x80); // NR10
        self.wb(0xff11, 0xbf); // NR11
        self.wb(0xff12, 0xf3); // NR12
        self.wb(0xff14, 0xbf); // NR14
        self.wb(0xff16, 0x3f); // NR21
        self.wb(0xff17, 0x00); // NR22
        self.wb(0xff19, 0xbf); // NR24
        self.wb(0xff1a, 0x7f); // NR30
        self.wb(0xff1b, 0xff); // NR31
        self.wb(0xff1c, 0x9F); // NR32
        self.wb(0xff1e, 0xbf); // NR33
        self.wb(0xff20, 0xff); // NR41
        self.wb(0xff21, 0x00); // NR42
        self.wb(0xff22, 0x00); // NR43
        self.wb(0xff23, 0xbf); // NR30
        self.wb(0xff24, 0x77); // NR50
        self.wb(0xff25, 0xf3); // NR51
        self.wb(0xff26, 0xf1); // NR52
        self.wb(0xff40, 0x91); // LCDC
        self.wb(0xff42, 0x00); // SCY
        self.wb(0xff43, 0x00); // SCX
        self.wb(0xff44, 0x00); // LY
        self.wb(0xff45, 0x00); // LYC
        self.wb(0xff47, 0x1b); // BGP   // 1b for entire palette
        self.wb(0xff48, 0xff); // OBP0
        self.wb(0xff49, 0xff); // OBP1
        self.wb(0xff4a, 0x00); // WY
        self.wb(0xff4b, 0x07); // WX, tweaked to position the window at (0, 0)
        self.wb(0xffff, 0x00); // IE

    }
    pub fn set_rom(&mut self, rom: Vec<u8>) {
        self.rom_loaded = rom;
    }
    // Borrow
    // pub fn borrow_rom_header(&mut self, header: &CartridgeHeader) {
    //  self.rom_header = Some(header);
    // }

    pub fn get_timers(&self) -> &Timer {
        &self.timer.as_ref()
    }

    // Private members

    fn read_byte_raw(&self, addr: u16) -> u8 {
        let addr = addr as usize;
        assert!(addr <= MEM_SIZE,
         "Invalid memory read: {:04X}", addr);

        self.raw_mem[addr]
    }

    fn read_word_raw(&self, addr: u16) -> u16 {
        let addr = addr as usize;
        assert!(addr <= MEM_SIZE - 1,
         "Invalid memory read: {:04X}", addr as usize);

        (self.raw_mem[addr] as u16) << 8 |
        (self.raw_mem[addr + 1] as u16)
    }

    fn write_byte_raw(&mut self, addr: u16, data: u8) {
        let addr = addr as usize;
        assert!(addr <= MEM_SIZE,
         "Invalid memory read: {:04X}", addr as usize);

        self.raw_mem[addr] = data
    }

    fn write_word_raw(&mut self, addr: u16, data: u16) {
        let addr = addr as usize;
        assert!(addr <= MEM_SIZE - 1,
         "Invalid memory write: {:04X}", addr);

        self.raw_mem[addr] = (data >> 8) as u8;
        self.raw_mem[addr + 1] = (data & 0x00FF) as u8;
    }

    // Public members

    // Read Byte
    // TODO: add 4 to total_cycles for cycle accuracy (not that simple)
    pub fn rb(&mut self, addr: u16) -> u8 {
        //self.debug_print_addr(addr, true);
        //self.timer.step(4, &mut self.if_);
        match addr {
            // ROM (switched bank)
            0x4000 ... 0x7FFF => {
                // if addr == 0x4000 {
                //     info!("read bank: {}  offset: {:04X}  is_ram: {}  addr & 0x3FFF: {:04X}  final: {:04X}", self.rom_bank, self.rom_offset, self.is_ram_mode, addr & 0x3FFF, self.rom_offset + (addr & 0x3FFF));
                // }
                self.read_byte_raw(self.rom_offset + (addr & 0x3FFF))
            },
            // VRAM so let the gpu handle it
            0x8000 ... 0x9FFF => self.gpu.rb_vram(addr),
            // External RAM
            0xA000 ... 0xBFFF => if self.enable_ext_ram {
                self.read_byte_raw(self.ram_offset + (addr & 0x1FFF))
                } else {
                    0xFF
                },
            // Mirrored memory
            0xE000 ... 0xFDFF => self.read_byte_raw(addr - 0x2000),
            0xFEA0 ... 0xFEFF => 0xFF, // { warn!("Unusable memory accessed"); 0xFF },
            0xFF00 ... 0xFF79 => self.ioreg_rb(addr),

            // Timer Registers
            //0xFF04 => self.

            // Interrupt enable
            0xFFFF => {
                info!("Interrupt enable read ie: {:08b}", self.ie_);
                self.ie_
            },
            _ => self.read_byte_raw(addr),
        }
    }

    // Read word
    pub fn rw(&mut self, addr: u16) -> u16 {
        assert!(addr <= 0xFFFF - 1,
         "Invalid memory read: {:04X}", addr);

        (self.rb(addr) as u16) |
        (self.rb(addr + 1) as u16) << 8
    }

    // Write byte
    pub fn wb(&mut self, addr: u16, data: u8) {
        //self.debug_print_addr(addr, false);
        //self.timer.step(4, &mut self.if_);
        match addr {
            // Enable external RAM if 0x0A was writtten. Disable it otherwise
            0x0000 ... 0x1FFF => if self.cart_type == 2 || self.cart_type == 3 {
                self.enable_ext_ram = data & 0x0F == 0x0A;
            },
            // Switch ROM bank
            0x2000 ... 0x3FFF => {
                match self.mbc {
                    Mbc::Mbc1 => {
                        let mut data_lower = data & 0x1F;
                        if data_lower == 0 { data_lower = 1 };

                        self.rom_bank = self.rom_bank & 0x60 + data_lower;
                        self.rom_offset = self.rom_bank as u16 * 0x4000;
                    },
                    //Mbc::Mbc2 => {},
                    //Mbc::Mbc3 => {},
                    //Mbc::Mbc4 => {},
                    //Mbc::Unknown => {},
                    _ => panic!("Unsupported MBC {:?}", self.mbc),
                }
            }
            // Switch ROM bank "set" {1-31}-{97-127} and RAM bank
            0x4000 ... 0x5FFF => {
                match self.mbc {
                    Mbc::Mbc1 => {
                        if self.is_ram_mode {
                            // RAM mode: Set bank
                            self.ram_bank = data & 3;
                            self.ram_offset = self.ram_bank as u16 * 0x2000;
                            info!("Switch RAM bank. bank: {}  offset: {:04X}", self.ram_bank, self.ram_offset);
                        } else {
                            // ROM mode: Set high bits of bank
                            self.rom_bank = self.rom_bank & 0x1F + ((data & 3) << 5);
                            self.rom_offset = self.rom_bank as u16 * 0x4000;
                            info!("Switch ROM bank. bank: {}  offset: {:04X}", self.rom_bank, self.rom_offset);
                        }
                    },
                    _ => panic!("Unsupported MBC {:?}", self.mbc),
                }
            }
            // Mode
            // 0: ROM mode (no RAM banks, up to 2MB ROM)
            // 1: RAM mode (4 RAM banks, up to 512kB ROM)
            0x6000 ... 0x7FFF => self.is_ram_mode = data & 1 == 1,
            0xA000 ... 0xBFFF => if self.enable_ext_ram {
                self.write_byte_raw(addr, data);
            },
            // Mirrored memory
            0xE000 ... 0xFDFF => self.write_byte_raw(addr - 0x2000, data),
            0xFEA0 ... 0xFEFF => debug!("Unusable memory written to"),
            // VRAM so let the gpu handle it
            0x8000 ... 0x9FFF => self.gpu.wb_vram(addr, data),
            // IO Ports
            0xFF00 ... 0xFF79 => self.ioreg_wb(addr, data),
            // Interrupt enable
            0xFFFF => {
                self.ie_ = data;
                info!("Interrupt enable write ie: {:08b}", self.ie_);
            },
            _ => {
                //debug!("raw byte write to addr: {:04X}  data: {:02X}", addr, data);
                self.write_byte_raw(addr, data);
            },
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
        //debug!("ioreg_rb {:x}", addr);
        match (addr >> 4) & 0xF {
            // I/O Ports (0xFF0x)
            0x0 => {
                match addr & 0xF {
                    // TODO: Input
                    //0x0 => self.input.rb(addr),
                    0x0 => {
                        warn!("Input requested (unimplemented) in address {:04X}", addr);
                        // Return no buttons pressed for now
                        0xFF // self.read_byte_raw(addr) & 0b00110000
                    },
                    0x4 => (self.timer.div >> 8) as u8,
                    0x5 => self.timer.tima,
                    0x6 => self.timer.tma,
                    0x7 => (self.timer.tac | 0xF8),
                    0xf => 0xE0 | self.if_,

                    _ => 0xFF//self.read_byte_raw(addr),
                }
            }
            // Video I/O Registers (0xFF4x)
            0x4 => {
                match addr & 0xF {
                    0...5 | 7...0xB | 0xF => {
                        //debug!("gpu_rb {:x}", addr);
                        self.gpu.rb(addr)
                    },
                    _ => 0xFF//self.read_byte_raw(addr),
                }
            }
            _ => 0xFF//self.read_byte_raw(addr),
        }
    }

    fn ioreg_wb(&mut self, addr: u16, data: u8) {
        use std::str;
        use std::io::prelude::*;
        use std::fs::OpenOptions;

        //debug!("ioreg_wb {:x} {:x}", addr, data);
        match (addr >> 4) & 0xF {

            // I/O Ports (0xFF0x)
            0x0 => {
                match addr & 0xF {
                    0x0 => self.write_byte_raw(addr, data),
                    0x1 => {
                        info!("Serial data transfer in address {:04X}, data {}", addr, data as char);

                        // TODO: Maybe open at constructor like trace_log.txt
                        // Open a file in write-only mode, returns `io::Result<File>`
                        let mut file = OpenOptions::new()
                            .append(true)
                            .create(true)
                            .open("serial_out.txt")
                            .unwrap();

                        file.write(&[data]).unwrap();
                    }
                    0x2 => {/* Serial transfer start */}
                    0x4 => { self.timer.div = 0; }
                    0x5 => { self.timer.tima = data; }
                    0x6 => { self.timer.tma = data; }
                    0x7 => {
                        self.timer.tac = data & 0b111;
                        self.timer.update();
                    }
                    0xf => { self.if_ = data; }
                    _ => {
                        warn!("Unhandled ioreg_wb address {:04X} written to. data: {:02X}", addr, data);
                        self.write_byte_raw(addr, data);
                    }
                }
            }
            // Video I/O Registers (0xFF4x)
            0x4 => {
                match addr & 0xF {
                    0...3 | 5 | 7...0xB => {
                        let dt = self.gpu.wb(addr, data);
                        //debug!("gpu_wb {:x} {:x}", addr, data);
                        dt
                    },
                    // Write to LY normally resets it, but it leads
                    // to challenging timings so just do nothing
                    4 => {},
                    6 => {
                            self.start_dma_transfer(data);
                            self.timer.step(4, &mut self.if_);
                        }
                    _ => self.write_byte_raw(addr, data)
                }
            }
            _ => {
                self.write_byte_raw(addr, data);
            }
        }
    }

    pub fn find_mbc(&mut self, cartridge_type: u8) {
        self.cart_type = cartridge_type;

        match cartridge_type {
            2 | 3 | 8 | 9 | 0xC | 0xD | 0x10 |
            0x12 | 0x13 | 0x16 | 0x17 | 0x1A |
            0x1B | 0x1D | 0x1E | 0xFF => {
                self.enable_ext_ram = true;
            }
            _ => {}
        };

        self.mbc = match cartridge_type {
            0x00 => Mbc::RomOnly,
            0x01 ... 0x03 => Mbc::Mbc1,
            0x05 ... 0x06 => Mbc::Mbc2,
            0x0F ... 0x13 => Mbc::Mbc3,
            0x15 ... 0x17 => Mbc::Mbc4,
            0x19 ... 0x1E => Mbc::Mbc5,

            _ => Mbc::Unknown,
        };

        // Only support MBC1 for now
        match self.mbc {
            Mbc::RomOnly | Mbc::Mbc1 => {},
            _ => panic!("Unsupported MBC: {:?}", self.mbc),
        };
        info!("Mbc: {:?}. External RAM: {}", self.mbc, self.enable_ext_ram);
    }


    fn debug_print_addr(&self, addr: u16, read: bool) {
        debug!("{} {:04X} in {}", if read {"Read from"} else {"Write to"}, addr,

        match addr {
            0x0000 ... 0x3FFF => "16KB ROM Bank 00",    // (in cartridge, fixed at bank 00)
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


    pub fn start_dma_transfer(&mut self, val: u8) {

        debug!("OAM DMA tranfer from 0x{:02X}00", val);

        if val > 0xF1 { error!("Invalid OAM DMA address"); return; }

        self.is_dma = true;
        self.dma_left = gpu::OAM_SIZE;
        self.dma_value = val;

        self.timer.step(4, &mut self.if_);
    }

    pub fn handle_dma_transfer(&mut self) {
        self.dma_left -= 1;
        if self.dma_left == 0 {
            self.is_dma = false;
        }
        // TODO: OPTIMIZE (compute above)
        let high_byte = (self.dma_value as u16) << 8;
        let low_byte = gpu::OAM_SIZE - self.dma_left -1;

        self.gpu.oam[low_byte] = self.rb(high_byte | low_byte as u16 );

        self.timer.step(4, &mut self.if_);

        // println!("{:04X} becomes {:02X}",
        // high_byte | low_byte as u16, self.rb(high_byte | low_byte as u16));
    }
}

//  ======================================
//  |               TESTS                |
//  ======================================

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