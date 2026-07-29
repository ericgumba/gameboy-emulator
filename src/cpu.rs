use crate::bus::Bus;

macro_rules! make_reg16_writer {
    ($name:ident, $hi:ident, $lo:ident) => {
        pub fn $name(&mut self, val: u16) {
            [self.$hi, self.$lo] = val.to_be_bytes();
        }
    };
}

const FLAG_C: u8 = 1 << 4;
const FLAG_H: u8 = 1 << 5;
const FLAG_Z: u8 = 1 << 7;
const FLAG_N: u8 = 1 << 6;

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
    
    pub fn sp_write(& mut self, val:u16){ self.sp = val;}
    pub fn af_write(&mut self, val: u16) { [self.a, self.f] = val.to_be_bytes() }
    pub fn bc_write(&mut self, val: u16) { [self.b, self.c] = val.to_be_bytes() }
    pub fn de_write(&mut self, val: u16) { [self.d, self.e] = val.to_be_bytes() }
    pub fn hl_write(&mut self, val: u16) { [self.h, self.l] = val.to_be_bytes() }
    pub fn af(&self) -> u16 {u16::from_be_bytes([self.a, self.f])}
    pub fn bc(&self) -> u16 {u16::from_be_bytes([self.b, self.c])}
    pub fn de(&self) -> u16 {u16::from_be_bytes([self.d, self.e])}
    pub fn hl(&self) -> u16 {u16::from_be_bytes([self.h, self.l])}

    fn add_to_hl(&mut self, val: u16) {

        let hl = self.hl();
        let (val, carry) = hl.overflowing_add(val);

        self.hl_write(val);

        let half_carry = (hl & 0xFFF) + (val & 0xFFF) > 0xFFF;
        if carry {
            self.f |= FLAG_C;
        } else {
            self.f &= !FLAG_C;

        }
        if half_carry {
            self.f |= FLAG_H;
        } else {
            self.f &= !FLAG_H;
        }
        self.f &= !FLAG_N;

    }

    pub fn step(& mut self, bus: &Bus) {

        let instr = self.fetch_u8(bus);

        match instr {
            0x00 => { }, // nop
            // ld r16, imm16
            0x01 | 0x11 | 0x21 | 0x31 => {
                let low = self.fetch_u8(bus);
                let high = self.fetch_u8(bus); 
                let val = u16::from_le_bytes([low, high]);
                match instr {
                    0x01 => self.bc_write(val),
                    0x11 => self.de_write(val),
                    0x21 => self.hl_write(val),
                    0x31 => self.sp = val,
                    _ => println!("not possible")
                }
            },
            // ld [r16mem], a
            0x02 | 0x12 | 0x22 | 0x32 => { 
                match instr {
                    0x02 => bus.write(self.bc(), self.a),
                    0x12 => bus.write(self.de(), self.a),
                    0x22 => bus.write(self.hl(), self.a),
                    0x32 => bus.write(self.sp, self.a), 
                    _ => panic!("unreachable Code")
                }
            },
            // ld a, [r16mem]
            0x0A | 0x1A | 0x2A | 0x3A => { 
                match instr {
                    0x0A => self.a = bus.read(self.bc()),
                    0x1A => self.a = bus.read(self.de()),
                    0x2A => self.a = bus.read(self.hl()),
                    0x3A => self.a = bus.read(self.sp),
                    _ => panic!("unreachable Code")
                }
            },
            //ld [imm16], sp

            0x08 => { 
                let low = self.fetch_u8(bus) as u16;
                let high = self.fetch_u8(bus) as u16;
                let addr = (high << 8) ^ low;
                
                // 0x1234
                let high_sp = (self.sp >> 8) as u8; // 0x12
                let low_sp = (self.sp & 0x00FF) as u8; // 0x34

                bus.write(addr, low_sp);
                bus.write(addr.wrapping_add(1), high_sp);



            },
            // inc r16 
            0x03 | 0x13 | 0x23 | 0x33 => {
                match instr {
                    0x03 => self.bc_write(self.bc() + 1),
                    0x13 => self.de_write(self.de() + 1),
                    0x23 => self.hl_write(self.hl()+1),
                    0x33 => self.sp = self.sp + 1, 
                    _ => panic!("unreachable Code")
                }

            },
            // dec r16
            0x0B | 0x1B | 0x2B | 0x3B => {
                match instr {
                    0x0B => self.bc_write(self.bc() - 1),
                    0x1B => self.de_write(self.de() - 1),
                    0x2B => self.hl_write(self.hl() - 1),
                    0x3B => self.sp = self.sp - 1,
                    _ => panic!("unreachable code")
                }
            },

            0x09 | 0x19 | 0x29 | 0x39 => {
                match instr {
                    0x09 => self.add_to_hl(self.bc()),
                    0x19 => self.add_to_hl(self.de()),
                    0x29 => self.add_to_hl(self.hl()),
                    0x39 => self.hl_write(self.sp),
                    _ => panic!("Unreachable code")

                }
            },
            



            _ => panic!("...")
            
        }
    }
}