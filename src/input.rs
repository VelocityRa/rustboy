use piston::input::Key;

pub struct Input {
    rows: [u8; 2],
    column: u8,
}

impl Input {
    pub fn new() -> Self {
        Input {rows: [0x0F, 0x0F], column: 0}
    }

    pub fn reset(&mut self) {
        self.rows = [0x0F, 0x0F];
        self.column = 0;
    }

    pub fn rb(&self) -> u8 {
        match self.column {
            0x10 => self.rows[0],
            0x20 => self.rows[1],
            _ => unreachable!("Invalid input column {:#X}", self.column)
        }
    }

    pub fn wb(&mut self, data: u8) {
        self.column = data & 0x30;
    }

    pub fn key_press(&mut self, key: &Key) {
        debug!("{:?} pressed", key);
        match *key {
            Key::Return => {self.rows[0] &= 0x7}
            Key::Space =>  {self.rows[0] &= 0xB}
            Key::Left =>   {self.rows[1] &= 0xD}
            Key::Up =>     {self.rows[1] &= 0xB}
            Key::Right =>  {self.rows[1] &= 0xE}
            Key::Down =>   {self.rows[1] &= 0x7}
            Key::X =>      {self.rows[0] &= 0xD}
            Key::Z =>      {self.rows[0] &= 0xE}
            _ => {}
        }
    }
    pub fn key_release(&mut self, key: &Key) {
        debug!("{:?} released", key);
        match *key {
            Key::Return => {self.rows[0] |= 0x8}
            Key::Space =>  {self.rows[0] |= 0x4}
            Key::Left =>   {self.rows[1] |= 0x2}
            Key::Up =>     {self.rows[1] |= 0x4}
            Key::Right =>  {self.rows[1] |= 0x1}
            Key::Down =>   {self.rows[1] |= 0x8}
            Key::X =>      {self.rows[0] |= 0x2}
            Key::Z =>      {self.rows[0] |= 0x1}
            _ => {}     
        }
    }
}