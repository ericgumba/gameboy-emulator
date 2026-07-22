use crate::cartridge::{Cartridge};

pub struct Bus {
    cartridge: Cartridge
}

impl Bus {
    pub fn new(cartridge: Cartridge) -> Self {

        Self {cartridge}

    }

    pub fn read(&self, addr: u16) -> u8 {
        self.cartridge.read(addr)
    }
    pub fn write(&self, addr: u16, val: u8) {
        unimplemented!();
    }
    
}