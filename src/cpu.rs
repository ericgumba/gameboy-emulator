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

    fn get_flag(&self, flag: u8) -> u8 {
        self.f & flag
    }

    fn set_flag(&mut self, val: u8, flag: u8) {
        if val == 1 {
            self.f |= flag
        } else {
            self.f &= !flag
        }
    }
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
        let half_carry = (hl & 0xFFF) + (val & 0xFFF) > 0xFFF;

        self.hl_write(val);

        self.set_flag(carry as u8, FLAG_C);
        self.set_flag(half_carry as u8, FLAG_H);
        self.set_flag(0, FLAG_N);

    }

    fn rla(&mut self) {
        let left_bit = self.a >> 7;
        let old_carry = self.get_flag(FLAG_C);
        self.a = self.a << 1 | old_carry;

        self.set_flag(left_bit, FLAG_C);
        self.set_flag(0, FLAG_H);
        self.set_flag(0, FLAG_N);
        self.set_flag(0, FLAG_Z);
    }

    fn rra(&mut self) {
        let right_bit = self.a & 0x1;
        let old_carry = self.get_flag(FLAG_C);
        self.a = self.a >> 1 | old_carry;

        self.set_flag(right_bit, FLAG_C);
        self.set_flag(0, FLAG_H);
        self.set_flag(0, FLAG_N);
        self.set_flag(0, FLAG_Z);
    }

    fn cpl(&mut self) {
        self.a = !self.a;
        self.set_flag(1, FLAG_N);
        self.set_flag(1, FLAG_H);

    }

    fn daa(&mut self) {
        let mut adjustment: u8 = 0;
        let subtract = self.get_flag(FLAG_N) != 0;
        let half_carry = self.get_flag(FLAG_H) != 0;
        let mut carry = self.get_flag(FLAG_C) != 0;

        if subtract {
            if half_carry {
                adjustment += 0x06;
            }
            if carry {
                adjustment += 0x60;
            }

            self.a = self.a.wrapping_sub(adjustment);
        } else {
            if half_carry || (self.a & 0x0F) > 0x09 {
                adjustment += 0x06;
            }
            if carry || self.a > 0x99 {
                adjustment += 0x60;
                carry = true;
            }

            self.a = self.a.wrapping_add(adjustment);
        }

        self.set_flag((self.a == 0) as u8, FLAG_Z);
        self.set_flag(0, FLAG_H);
        self.set_flag(carry as u8, FLAG_C);
    }

    fn rlca(&mut self) {
        let left_bit = self.a >> 7;
        self.a = self.a.rotate_left(1);
        self.a |= left_bit;

        self.set_flag(left_bit, FLAG_C);
        self.set_flag(0, FLAG_H);
        self.set_flag(0, FLAG_N);
        self.set_flag(0, FLAG_Z);
    }

    fn rrca(&mut self) {
        let right_bit = self.a & 0x1;
        self.a = self.a.rotate_right(1);
        self.a |= right_bit;

        self.set_flag(right_bit, FLAG_C);
        self.set_flag(0, FLAG_H);
        self.set_flag(0, FLAG_N);
        self.set_flag(0, FLAG_Z);
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
            0x04 | 0x14 | 0x24 | 0x34 | 0x44 | 0x54 | 0x64 | 0x74 => {
                let rhs: u8 = 1;
                match instr {
                    0x04 => self.b = self.b.wrapping_add(rhs),
                    0x14 => self.c = self.c.wrapping_add(rhs),
                    0x24 => self.d = self.d.wrapping_add(rhs),
                    0x34 => self.e = self.e.wrapping_add(rhs),
                    0x44 => self.h = self.h.wrapping_add(rhs),
                    0x54 => self.l = self.l.wrapping_add(rhs),
                    0x64 => bus.write(self.hl(), bus.read(self.hl()).wrapping_add(rhs)),
                    0x74 => self.a = self.a.wrapping_add(rhs),
                    _ => panic!("Unreachable code")
                }
            },
            0x05 | 0x15 | 0x25 | 0x35 | 0x45 | 0x55 | 0x65 | 0x75 => {
                let rhs: u8 = 1;
                match instr {
                    0x04 => self.b = self.b.wrapping_sub(rhs),
                    0x14 => self.c = self.c.wrapping_sub(rhs),
                    0x24 => self.d = self.d.wrapping_sub(rhs),
                    0x34 => self.e = self.e.wrapping_sub(rhs),
                    0x44 => self.h = self.h.wrapping_sub(rhs),
                    0x54 => self.l = self.l.wrapping_sub(rhs),
                    0x64 => bus.write(self.hl(), bus.read(self.hl()).wrapping_sub(rhs)),
                    0x74 => self.a = self.a.wrapping_sub(rhs),
                    _ => panic!("Unreachable code")
                }
            },
            // ld r8, imm8	| 0 	0	 x  x  x	1	1	0 
            0x06 | 0x16 | 0x26 | 0x36 | 0x0E | 0x1E | 0x2E | 0x3E => {
                let val = self.fetch_u8(bus);
                match instr {
                    0x06 => self.b = val,
                    0x0E => self.c = val,
                    0x16 => self.d = val,
                    0x1E => self.e = val,
                    0x26 => self.h = val,
                    0x2E => self.l = val,
                    0x36 => bus.write(self.hl(), val),
                    0x3E => self.a = val,
                    _ => panic!("UC")
                }
            },
            0x07 => self.rlca(),
            0x0F => self.rrca(),
            0x17 => self.rla(),
            0x1F => self.rra(),
            0x27 => self.daa(),
            0x2F => self.cpl(),
            0x37 => {
                self.set_flag(1, FLAG_C);
                self.set_flag(0, FLAG_N);
                self.set_flag(0, FLAG_H);
            },
            0x3F => {
                self.set_flag(!self.get_flag(FLAG_C), FLAG_C);
                self.set_flag(0, FLAG_N);
                self.set_flag(0, FLAG_H);
            },
            0x18 => {
                let offset = self.fetch_u8(bus) as i8;
                self.pc = self.pc.wrapping_add_signed(offset as i16);
            },
            0x20 | 0x28 | 0x30 | 0x38 => {

            },












            _ => panic!("...")
            
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn daa_adjusts_low_digit_after_addition() {
        let mut cpu = CPU::new();
        cpu.a = 0x0A;

        cpu.daa();

        assert_eq!(cpu.a, 0x10);
        assert_eq!(cpu.f, 0);
    }

    #[test]
    fn daa_adjusts_both_digits_and_sets_carry_and_zero() {
        let mut cpu = CPU::new();
        cpu.a = 0x9A;

        cpu.daa();

        assert_eq!(cpu.a, 0x00);
        assert_ne!(cpu.get_flag(FLAG_Z), 0);
        assert_ne!(cpu.get_flag(FLAG_C), 0);
        assert_eq!(cpu.get_flag(FLAG_H), 0);
        assert_eq!(cpu.get_flag(FLAG_N), 0);
    }

    #[test]
    fn daa_subtracts_adjustment_and_preserves_subtract_and_carry() {
        let mut cpu = CPU::new();
        cpu.a = 0x73;
        cpu.f = FLAG_N | FLAG_H | FLAG_C;

        cpu.daa();

        assert_eq!(cpu.a, 0x0D);
        assert_ne!(cpu.get_flag(FLAG_N), 0);
        assert_ne!(cpu.get_flag(FLAG_C), 0);
        assert_eq!(cpu.get_flag(FLAG_H), 0);
        assert_eq!(cpu.get_flag(FLAG_Z), 0);
    }
}
