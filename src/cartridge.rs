
use std::fmt;
use emulator::Emulator;

#[derive(Default)]
#[repr(C, packed)]
pub struct CartridgeHeader {
    // Usually a NOP and a JP to 0x0150
    entry_point: [u16; 2],

    // Bitmap of the Nintendo logo
    // Use u16 so that we can use the default Default trait
    // TODO: Don't be lazy and implement our own Default trait
    nintendo_logo: [u16; 24],

    // Game title in upper case ASCII
    game_title: [u8; 16],
    //manufacturer_code: [u8; 4],

    //80h - Game supports CGB functions, but works on old gameboys also.
    //C0h - Game works on CGB only (physically the same as 80h).
    //cgb_flag: u8,

    // Used by newer games
    new_licence_code: [u8; 2],

    // Specifies whether the game supports SGB functions, common values:
    // 00h = No SGB functions (Normal Gameboy or CGB only game)
    // 03h = Game supports SGB functions
    // The SGB disables its SGB functions if this byte
    // is set to another value than 03h.
    sgb_flag: u8,

    // Specifies which Memory Bank Controller (if any) is used in the
    // cartridge, and if further external hardware exists in the cartridge.
    pub cartridge_type: u8,

    // Typically calculated as "32KB << N"
    pub rom_size: u8,

    // Specifies the size of the external RAM in the cartridge (if any).
    // 00h - None
    // 01h - 2 KBytes
    // 02h - 8 Kbytes
    // 03h - 32 KBytes (4 banks of 8KBytes each)
    ram_size: u8,

    // 0 = Japanese, 1 = Non-Japanese
    dest_code: u8,

    // If 0x33 new_licence_code is used instead
    old_licence_code: u8,

    // Usually 0
    rom_version_number: u8,
    header_checksum: u8,
    global_checksum: u16,
}

impl CartridgeHeader {
    pub fn get_game_title(&self) -> String {
        use std::str;
        use std::env;
        
        let args: Vec<_> = env::args().collect();
        let mut title = String::from(
            match str::from_utf8(&self.game_title) {
                Ok(val) => val,
                Err(err) => {
                    warn!("Couldn't read rom name from header, using file name instead");
                    &args[1]
                }
            }
        );
        // Get rid of trailing null values
        let end_offset = title.find('\0').unwrap_or(title.len());
        title.drain(end_offset..).collect::<String>();
        title
    }
}

impl fmt::Debug for CartridgeHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CartridgeHeader {{
            entry_point: {:04X}{:04X}
            game_title: {:?}
            sgb_flag: {}
            cartridge_type: {}
            rom_size: {}
            dest_code: {}{}
            header_checksum: {:04X}
            global_checksum: {:04X}
        }}",
            self.entry_point[0], self.entry_point[1],
            self.get_game_title(),
            self.sgb_flag,
            self.cartridge_type,
            self.rom_size,
            if self.dest_code == 0 {""} else {"Non-"}, "Japanese",
            self.header_checksum,
            self.global_checksum
            )
    }
}


pub fn read_header_impl(emu: &Emulator) -> CartridgeHeader {
    use std::slice;
    use std::io::Read;

    const HEADER_SIZE: usize = 0x50;
    const HEADER_OFFSET: usize = 0x100;

    let mut buffer: [u8; HEADER_SIZE] = [0u8; HEADER_SIZE];

    for i in 0..HEADER_SIZE {
        buffer[i] = emu.mem.rom_loaded[i + HEADER_OFFSET];
    }

    let mut buffer_slice: &[u8] = &buffer;

    let mut header: CartridgeHeader = Default::default();

    unsafe {
        let header_slice = slice::from_raw_parts_mut(
            &mut header as *mut _ as *mut u8,
            HEADER_SIZE
        );
        
        // `read_exact()` comes from `Read` impl for `&[u8]`
        buffer_slice.read_exact(header_slice).unwrap();
    }

    println!("Read header: {:#?}", header);
    header
}

#[cfg(test)]
mod cart_tests {
    use super::*;

    #[test]
    fn header_size() {
        use std::mem;

        assert_eq!(0x50, mem::size_of::<CartridgeHeader>());
    }
}