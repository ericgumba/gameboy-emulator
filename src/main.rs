use gameboy_emulator::cpu::CPU;
use std::fmt;

struct Test {
    f: u8
}

impl fmt::Display for Test {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.f)
    }
}

fn main() {
    let t: Test = Test { f: 3 };
    println!("{}", t);

    println!("Hello, world!");
}
