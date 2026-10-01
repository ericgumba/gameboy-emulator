use crate::cartridge::{Cartridge};


pub trait Memory {
    fn read(&mut self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);
}
pub struct Bus {
    cartridge: Cartridge
}

impl Bus {
    pub fn new(cartridge: Cartridge) -> Self {

        Self {cartridge}

    }
}
impl Memory for Bus {
    fn read(&mut self, addr: u16) -> u8 {
        self.cartridge.read(addr)
    }
    fn write(&mut self, _addr: u16, _val: u8) {
        unimplemented!(); 
    }
    
}