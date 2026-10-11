use gameboy_emulator::{bus::Bus, cartridge::{self, Cartridge}, cpu::CPU};
use std::fmt;
 
// 01-special.gb.      P
// 02-interrupts.gb
// 03-op sp,hl.gb      P
// 04-op r,imm.gb      P
// 05-op rp.gb         P
// 06-ld r,r.gb        P
// 08-misc instrs.gb   P
// 09-op r,r.gb        P
// 10-bit ops.gb       P
// 11-op a,(hl).gb     P
fn main() -> Result<(), Box<dyn std::error::Error>>{ 
    // let cartridge: Cartridge = Cartridge::new("blarg/cpu_instrs/cpu_instrs.gb")?; 
    let cartridge: Cartridge = Cartridge::new("blarg/cpu_instrs/individual/02-interrupts.gb")?; 

    let mut bus: Bus = Bus::new(cartridge);
    let mut cpu: CPU = CPU::new(); 
    loop {
        cpu.step(&mut bus);
        // println!("Step count {}", step_count);
        // step_count += 1;
        // println!("Instruction yielded {} cycles",);
    }


 
    Ok(())
}
