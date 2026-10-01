use std::{fs, io};

pub struct Cartridge {
    rom: Vec<u8>,
    ram: [u8; 0x2000],
}

impl Cartridge {
    pub fn new(filename: &str) -> io::Result<Self> {
        let rom = fs::read(filename)?;
        Ok(Self::from_bytes(rom))
    }

    pub fn from_bytes(rom: Vec<u8>) -> Self {
        Self {
            rom,
            ram: [0; 0x2000],
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7FFF => self.rom.get(addr as usize).copied().unwrap_or(0xFF),
            0xA000..=0xBFFF => self.ram[(addr - 0xA000) as usize],
            _ => unreachable!("address {addr:#06X} is outside cartridge memory"),
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            // Writes in this range will control the memory-bank controller once
            // cartridge types and ROM banking are implemented.
            0x0000..=0x7FFF => {}
            0xA000..=0xBFFF => self.ram[(addr - 0xA000) as usize] = value,
            _ => unreachable!("address {addr:#06X} is outside cartridge memory"),
        }
    }
}
