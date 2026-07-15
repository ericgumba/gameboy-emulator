use crate::bus::Bus;

pub struct CPU {
    a: u8,
    f: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    sp: u16,
    pc: u16
}

impl CPU {
    pub fn new() -> Self {
        Self {
            a: 0,
            f: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            sp: 0,
            pc: 0x100, // Or 0x0100 for a post-bootrom Game Boy state
        }
    }
    pub fn load_into_bc(& mut self, val1:u8, val2:u8){
        self.b = val1;
        self.c = val2;
    }
    pub fn load_into_de(& mut self, val1:u8, val2:u8){
        self.d = val1;
        self.e = val2;
    }
    pub fn load_into_hl(& mut self, val1:u8, val2:u8){
        self.h = val1;
        self.l = val2;
    }
    pub fn load_into_sp(& mut self, val1:u8, val2:u8){
        self.sp = u16::from_le_bytes([val1, val2])
    }
    pub fn step(& mut self, bus: &Bus) {

        let instr = bus.read(self.pc);

        match instr {
            0x00 => { }, // nop
            0x01 | 0x11 | 0x21 | 0x31 => {
                let val1 = bus.read(self.pc+1);
                let val2 = bus.read(self.pc+2);
                self.pc += 2;
                match instr {
                    0x01 => self.load_into_bc(val1, val2),
                    0x11 => self.load_into_de(val1, val2),
                    0x21 => self.load_into_hl(val1, val2),
                    0x31 => self.load_into_sp(val1, val2)
                }


            }
            _ => panic!("...")
            
        }

        self.pc += 1;

        
    }
}