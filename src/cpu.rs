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
    
    pub fn fetch_u8(&mut self, bus: &Bus) -> u8 {
        let byte = bus.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        byte
    }
    
    pub fn sp_write(& mut self, low:u8, high:u8){ self.sp = u16::from_le_bytes([low, high]);}
    pub fn bc_write(&mut self, low: u8, high: u8) { self.b = high; self.c = low; }
    pub fn de_write(&mut self, low: u8, high: u8) { self.d = high; self.e = low; }
    pub fn hl_write(&mut self, low: u8, high: u8) { self.h = high; self.l = low; }

    pub fn bc(&self) -> u16 {u16::from_be_bytes([self.b, self.c])}
    pub fn de(&self) -> u16 {u16::from_be_bytes([self.d, self.e])}
    pub fn hl(&self) -> u16 {u16::from_be_bytes([self.h, self.l])}

    pub fn step(& mut self, bus: &Bus) {

        let instr = self.fetch_u8(bus);

        match instr {
            0x00 => { }, // nop
            0x01 | 0x11 | 0x21 | 0x31 => {
                let low = self.fetch_u8(bus);
                let high = self.fetch_u8(bus); 
                match instr {
                    0x01 => self.bc_write(low, high),
                    0x11 => self.de_write(low, high),
                    0x21 => self.hl_write(low, high),
                    0x31 => self.sp_write(low, high),
                    _ => println!("not possible")
                }
            },
            0x02 | 0x12 | 0x22 | 0x32 => { 
                match instr {
                    0x02 => bus.write(self.bc(), self.a),
                    0x12 => bus.write(self.de(), self.a),
                    0x22 => bus.write(self.hl(), self.a),
                    0x32 => bus.write(self.sp, self.a), 
                    _ => panic!("unreachable Code")
                }
            },
            0x0A | 0x1A | 0x2A | 0x3A => { 
                match instr {
                    0x0A => self.a = bus.read(self.bc()),
                    0x1A => self.a = bus.read(self.de()),
                    0x2A => self.a = bus.read(self.hl()),
                    0x3A => self.a = bus.read(self.sp),
                    _ => panic!("unreachable Code")
                }
            },

            0x08 => 
            
            0x03 | 0x13 | 0x23 | 0x33 => { 
            },
            _ => panic!("...")
            
        }
 

        
    }
}