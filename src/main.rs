use gameboy_emulator::{bus::Bus, cartridge::{self, Cartridge}, cpu::CPU};
use std::fmt;
 
fn main() -> Result<(), Box<dyn std::error::Error>>{ 
    let cartridge: Cartridge = Cartridge::new("blarg/cpu_instrs/cpu_instrs.gb")?; 

    let bus: Bus = Bus::new(cartridge);

    bus.step();



    println!("Hello, world!");
    Ok(())
}
