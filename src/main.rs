use gameboy_emulator::{bus::Bus, cartridge::{self, Cartridge}, cpu::CPU};
use std::fmt;
 
fn main() -> Result<(), Box<dyn std::error::Error>>{ 
    let cartridge: Cartridge = Cartridge::new("blarg/cpu_instrs/cpu_instrs.gb")?; 

    let mut bus: Bus = Bus::new(cartridge);
    let mut cpu: CPU = CPU::new();

    while !cpu.halted() {
        println!("Instruction yielded {} cycles",cpu.step(&mut bus));
    }



    println!("Hello, world!");
    Ok(())
}
