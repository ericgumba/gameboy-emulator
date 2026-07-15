use std::{fs, io};

pub struct Cartridge {
    rom: Vec<u8>
}


impl Cartridge {
    pub fn new(filename: &str) -> io::Result<Self> {
        let rom = fs::read(filename)?;
        Ok(Self { rom })
    }

    pub fn read(&self, addr:u16) -> u8 {
        self.rom[addr as usize]
    }

    pub fn get_bytes(&self) {
        let mut i = 0;
        for b in &self.rom {
            println!("{}", b);
            i+=1;
            if i > 10 {
                break;
            }
        }
    }
}