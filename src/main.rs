use gameboy_emulator::{bus::Bus, cartridge::{self, Cartridge}, cpu::CPU};
use std::fmt;
 
fn main() -> Result<(), Box<dyn std::error::Error>>{ 
    let cartridge: Cartridge = Cartridge::new("blarg/cpu_instrs/individual/11-op a,(hl).gb")?; 

    let mut bus: Bus = Bus::new(cartridge);
    let mut cpu: CPU = CPU::new(); 
    loop {
        cpu.step(&mut bus);
    }


 
    Ok(())
}
