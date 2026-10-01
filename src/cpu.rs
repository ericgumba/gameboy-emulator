use crate::bus::Memory;

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
    pc: u16,
    halted: bool,
    ime: bool
}

impl CPU {

    fn get_flag(&self, flag: u8) -> u8 {
        u8::from(self.f & flag != 0)
    }

    fn set_flag(&mut self, flag: u8, val: bool) {
        if val {
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
            halted: false,
            ime: false
            
        }
    }

    pub fn halted(& self) -> bool {self.halted}

    // imm8 = fetch_u8
    // imm16 = call fetch_u8 twice
    fn fetch_u8<M: Memory>(&mut self, bus: &mut M) -> u8 {
        let byte = bus.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        byte
    }
    fn fetch_u16<M: Memory>(&mut self, bus: &mut M) -> u16 {
        let low = self.fetch_u8(bus);
        let high = self.fetch_u8(bus);
        u16::from_le_bytes([low, high])

    }

    fn inc_u8(&mut self, value: u8) -> u8 {
        let result = value.wrapping_add(1);

        // Z: Set if result is zero.
        self.set_flag(FLAG_Z, result == 0);

        // N: INC is an addition, so clear subtract flag.
        self.set_flag(FLAG_N, false);

        // H: Set if there is a carry from bit 3 to bit 4.
        self.set_flag(FLAG_H, (value & 0x0F) == 0x0F);

        // C: Unchanged.

        result
    }
    fn dec_u8(&mut self, value: u8) -> u8 {
        let result = value.wrapping_sub(1);

        // Z: Set if result is zero.
        self.set_flag(FLAG_Z, result == 0);

        // N: DEC is a subtraction.
        self.set_flag(FLAG_N, true);

        // H: Set if borrowing from bit 4.
        self.set_flag(FLAG_H, (value & 0x0F) == 0);

        // C: Unchanged.

        result
    }
    pub fn bc(&self) -> u16 {u16::from_be_bytes([self.b, self.c])}
    pub fn de(&self) -> u16 {u16::from_be_bytes([self.d, self.e])}
    pub fn hl(&self) -> u16 {u16::from_be_bytes([self.h, self.l])}

    fn add_to_a(&mut self, val: u8) {
        let a = self.a;
        let (val, carry) = a.overflowing_add(val);
        let half_carry = (a & 0xF) + (val & 0xF) > 0xF;

        self.a = val;

        self.set_flag(FLAG_C, carry);
        self.set_flag(FLAG_H, half_carry);
        self.set_flag(FLAG_Z, val == 0);
        self.set_flag(FLAG_N, false);
    }

    fn sub_from_a(&mut self, val: u8) {
        let a = self.a;
        let (val, carry) = a.overflowing_sub(val);

        let half_carry = (a & 0xF) < (val & 0xF);

        self.a = val;
        
        
        self.set_flag(FLAG_Z, val == 0);
        self.set_flag(FLAG_N, true);
        self.set_flag(FLAG_H, half_carry);
        self.set_flag(FLAG_C, carry);


    }

    fn add_to_hl(&mut self, val: u16) {

        let hl = self.hl();
        let (val, carry) = hl.overflowing_add(val);
        let half_carry = (hl & 0xFFF) + (val & 0xFFF) > 0xFFF;

        [self.h, self.l] = val.to_be_bytes();

        self.set_flag(FLAG_C, carry);
        self.set_flag(FLAG_H, half_carry);
        self.set_flag(FLAG_N, false);

    }

    fn add_sp_offset(&mut self, offset: u8) -> u16 {
        let sp = self.sp;

        self.set_flag(FLAG_Z, false);
        self.set_flag(FLAG_N, false);
        self.set_flag(
            FLAG_H,
            (sp & 0x000F) + u16::from(offset & 0x0F) > 0x000F,
        );
        self.set_flag(
            FLAG_C,
            (sp & 0x00FF) + u16::from(offset) > 0x00FF,
        );

        sp.wrapping_add_signed(i16::from(offset as i8))
    }

    fn update_bitwise_flags(&mut self, h_flag_val: bool) {
        self.set_flag(FLAG_Z, false);
        self.set_flag(FLAG_N, false);
        self.set_flag(FLAG_H, h_flag_val);
        self.set_flag(FLAG_C, false);
        
    }

    fn update_rotate_flags(&mut self, c_flag_val: bool, z_flag_val: bool) {
        self.set_flag(FLAG_C, c_flag_val);
        self.set_flag(FLAG_H, false);
        self.set_flag(FLAG_N, false);
        self.set_flag(FLAG_Z, z_flag_val);
    }

    fn cpl(&mut self) {
        self.a = !self.a;
        self.set_flag(FLAG_N, true);
        self.set_flag(FLAG_H, true);

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

        self.set_flag(FLAG_Z, self.a == 0);
        self.set_flag(FLAG_H, false);
        self.set_flag(FLAG_C, carry);
    }

    fn rla(&mut self) {
        let left_bit = self.a >> 7;
        let old_carry = self.get_flag(FLAG_C);
        self.a = self.a << 1 | old_carry;
        self.update_rotate_flags(left_bit == 1, false);
    }

    fn rl(&mut self, value: u8) -> u8 {
        let left_bit = value >> 7;
        let old_carry = self.get_flag(FLAG_C);
        let ret = value << 1 | old_carry;
        self.update_rotate_flags(left_bit == 1, ret == 0);
        ret
    }

    fn sla(&mut self, value: u8) -> u8 {
        let left_bit = value >> 7;
        let ret = value << 1;
        self.update_rotate_flags(left_bit == 1, ret == 0);
        ret
    }



    fn rra(&mut self) {
        let right_bit = self.a & 0x1;
        let old_carry = self.get_flag(FLAG_C);
        self.a = (self.a >> 1) | (old_carry << 7);
        self.update_rotate_flags(right_bit == 1, false);
    }

    fn rr(&mut self, value: u8) -> u8 {
        let right_bit = value & 1;
        let old_carry = self.get_flag(FLAG_C);
        let ret = value >> 1 | old_carry << 7;
        self.update_rotate_flags(right_bit == 1, ret == 0);
        ret
    }
    fn sra(&mut self, value: u8) -> u8 {
        let right_bit = value & 1;
        let left_bit = value >> 7;
        let ret = value >> 1 | left_bit << 7;
        self.update_rotate_flags(right_bit == 1, ret == 0);
        ret
    }

    fn srl(&mut self, value: u8) -> u8 {
        let right_bit = value & 1;
        let ret = value >> 1;
        self.update_rotate_flags(right_bit == 1, ret == 0);
        ret
    }

    fn rlca(&mut self) {
        let left_bit = self.a >> 7;
        self.a = self.a.rotate_left(1);
        self.update_rotate_flags(left_bit == 1, false);
    }
    
    pub fn rlc(& mut self, value: u8) -> u8 {
        let left_bit = value >> 7;
        let ret = value.rotate_left(1);
        self.update_rotate_flags(left_bit == 1, ret == 0);
        ret
    }

    fn rrca(&mut self) {
        let right_bit = self.a & 0x1;
        self.a = self.a.rotate_right(1);
        self.update_rotate_flags(right_bit == 1, false);
    }

    fn rrc(&mut self, value: u8) -> u8 {
        let right_bit = value & 0x1;
        let ret = value.rotate_right(1);
        self.update_rotate_flags(right_bit == 1, ret == 0);
        ret
    }

    // 1010 0110

    fn swap(&mut self, value: u8) -> u8 { 
        let result = value.rotate_left(4);

        self.set_flag(FLAG_Z, result == 0);
        self.set_flag(FLAG_N, false);
        self.set_flag(FLAG_H, false);
        self.set_flag(FLAG_C, false);

        result
    }

 

    fn read_r8<M: Memory>(&self, register: u8, bus: &mut M) -> u8 {
        match register {
            0 => self.b,
            1 => self.c,
            2 => self.d,
            3 => self.e,
            4 => self.h,
            5 => self.l,
            6 => bus.read(self.hl()),
            7 => self.a,
            _ => unreachable!("an 8-bit register selector is always three bits"),
        }
    }

    fn read_r16(&self, register: u8) -> u16 {
        match register {
            0 => self.bc(),
            1 => self.de(),
            2 => self.hl(),
            3 => self.sp,
            _ => unreachable!("a 16 bit register selector is always two bits"),
        }
    }

    fn read_r16mem(&mut self, register: u8) -> u16 {
        let ret = match register {
            0 => self.bc(),
            1 => self.de(),
            2 => {
                let ret = self.hl();
                self.write_r16(2, ret.wrapping_add(1));
                ret
            },
            3 => {
                let ret = self.hl();
                self.write_r16(2, ret.wrapping_sub(1));
                ret
            },
            _ => unreachable!("a 16 bit register selector is always two bits"),
        };

        ret
    }

    fn read_r16stk(&mut self, register: u8) -> u16 {
        match register {
            0 => u16::from_be_bytes([self.b, self.c]),
            1 => u16::from_be_bytes([self.d, self.e]),
            2 => u16::from_be_bytes([self.h, self.l]),
            3 => u16::from_be_bytes([self.a, self.f]),
            _ => unreachable!("a 16 bit register selector is always two bits")
        }
    }
    
    fn write_r16stk(&mut self, register: u8, value: u16) {
        match register {
            0 => [self.b, self.c] = value.to_be_bytes(),
            1 => [self.d, self.e] = value.to_be_bytes(),
            2 => [self.h, self.l] = value.to_be_bytes(),
            3 => {[self.a, self.f] = value.to_be_bytes(); self.f &= 0xF0},
            _ => unreachable!("a 16 bit register selector is always two bits")

        }
    }

    fn write_r16(&mut self, register: u8, value: u16) {
        match register {
            0 => [self.b, self.c] = value.to_be_bytes(),
            1 => [self.d, self.e] = value.to_be_bytes(),
            2 => [self.h, self.l] = value.to_be_bytes(),
            3 => self.sp = value,
            _ => unreachable!("a 16 bit register selector is always two bits")

        }
    }

    fn write_r8<M: Memory>(&mut self, register: u8, value: u8, bus: &mut M) {
        match register {
            0 => self.b = value,
            1 => self.c = value,
            2 => self.d = value,
            3 => self.e = value,
            4 => self.h = value,
            5 => self.l = value,
            6 => bus.write(self.hl(), value),
            7 => self.a = value,
            _ => unreachable!("an 8-bit register selector is always three bits"),
        }
    }

    fn cond_flag(&self, flag: u8) -> bool {

        match flag {
            0 => self.get_flag(FLAG_Z) == 0,
            1 => self.get_flag(FLAG_Z) == 1,
            2 => self.get_flag(FLAG_C) == 0,
            3 => self.get_flag(FLAG_C) == 1,
            _ => unreachable!("what?")
        }

    }

    fn compare_a(&mut self, reg: u8) {
        let val = self.a.wrapping_sub(reg);
        self.set_flag(FLAG_Z, val == 0);
        self.set_flag(FLAG_N, true);
        self.set_flag(FLAG_H, self.a & 0x0F < reg & 0x0F);
        self.set_flag(FLAG_C, self.a < reg);

    }
    fn execute_cb_operation(&mut self, value: u8, opcode: u8) -> u8 {
        match opcode {
            0x00..0x08 => self.rlc(value),
            0x08..0x10 => self.rrc(value),
            0x10..0x18 => self.rl(value),
            0x18..0x20 => self.rr(value),
            0x20..0x28 => self.sla(value),
            0x28..0x30 => self.sra(value),
            0x30..0x38 => self.swap(value),
            0x38..0x40 => self.srl(value),
            _ => unreachable!("")
        }

    }

    fn cb_bit_op(&mut self, value: u8, opcode: u8) -> Option<u8> {
        let bit_op = opcode >> 6;
        let bit_index = opcode >> 3 & 0b111;
        match bit_op {
            1 => {
                self.set_flag(FLAG_Z, ((value >> bit_index) & 1) == 0);
                self.set_flag(FLAG_N, false);
                self.set_flag(FLAG_H, true);
                None
            },
            2 => {
                Some(value & !(1u8 << bit_index))
            },
            3 => {

                Some(value | (1u8 << bit_index))
            },
            _ => unreachable!("")

        }
        
    }

    fn handle_cb<M: Memory>(&mut self, opcode: u8, bus: &mut M) -> u8 {
        let register_index = opcode & 0b0000_0111;
        let is_bit_index_op = opcode >> 6 != 0;
        let value = self.read_r8(register_index, bus);
        if is_bit_index_op {
            if let Some(result) = self.cb_bit_op(value, opcode) {
                self.write_r8(register_index, result, bus);
            }
        }
        else {
            let res = self.execute_cb_operation(value, opcode);
            self.write_r8(register_index, res, bus);
        }

        if register_index != 6 {
            2
        } else if opcode >> 6 == 1 {
            3
        } else {
            4
        }
    }


    pub fn step<M: Memory>(&mut self, bus: &mut M) -> u8 {
        if self.halted {
            return 1;
        }

        let instr = self.fetch_u8(bus);

        match instr {
            0x00 => 1, // NOP
            // LD r16, imm16
            0x01 | 0x11 | 0x21 | 0x31 => {
                let val = self.fetch_u16(bus);
                self.write_r16((instr >> 4) & 0b11, val);
                3
            }
            // LD [r16mem], A
            0x02 | 0x12 | 0x22 | 0x32 => {
                let addr = self.read_r16mem((instr >> 4) & 0b11);
                bus.write(addr, self.a);
                2
            }
            // LD A, [r16mem]
            0x0A | 0x1A | 0x2A | 0x3A => {
                let addr = self.read_r16mem((instr >> 4) & 0b11);
                self.a = bus.read(addr);
                2
            }
,
            // LD [imm16], SP
            0x08 => {
                let addr = self.fetch_u16(bus);
                let [low, high] = self.sp.to_le_bytes();
                bus.write(addr, low);
                bus.write(addr.wrapping_add(1), high);
                5
            }

            // INC r16
            0x03 | 0x13 | 0x23 | 0x33 => {
                let register = (instr >> 4) & 0b11;
                let value = self.read_r16(register).wrapping_add(1);
                self.write_r16(register, value);
                2
            }

            // DEC r16
            0x0B | 0x1B | 0x2B | 0x3B => {
                let register = (instr >> 4) & 0b11;
                let value = self.read_r16(register).wrapping_sub(1);
                self.write_r16(register, value);
                2
            }
            // ADD HL, r16
            0x09 | 0x19 | 0x29 | 0x39 => {
                let value = self.read_r16((instr >> 4) & 0b11);
                self.add_to_hl(value);
                2
            }
            // INC r8
            0x04 | 0x0C | 0x14 | 0x1C | 0x24 | 0x2C | 0x34 | 0x3C => {
                let register = (instr >> 3) & 0b111;
                let value = self.read_r8(register, bus);
                let result = self.inc_u8(value);
                self.write_r8(register, result, bus);
                if register == 6 { 3 } else { 1 }
            }
            // DEC r8
            0x05 | 0x0D | 0x15 | 0x1D | 0x25 | 0x2D | 0x35 | 0x3D => {
                let register = (instr >> 3) & 0b111;
                let value = self.read_r8(register, bus);
                let result = self.dec_u8(value);
                self.write_r8(register, result, bus);
                if register == 6 { 3 } else { 1 }
            }
            // LD r8, imm8
            0x06 | 0x16 | 0x26 | 0x36 | 0x0E | 0x1E | 0x2E | 0x3E => {
                let value = self.fetch_u8(bus);
                let register = (instr >> 3) & 0b111;
                self.write_r8(register, value, bus);
                if register == 6 { 3 } else { 2 }
            }
            0x07 => {
                self.rlca();
                1
            }
            0x0F => {
                self.rrca();
                1
            }
            0x17 => {
                self.rla();
                1
            }
            0x1F => {
                self.rra();
                1
            }
            0x27 => {
                self.daa();
                1
            }
            0x2F => {
                self.cpl();
                1
            }
            0x37 => {
                self.set_flag(FLAG_C, true);
                self.set_flag(FLAG_N, false);
                self.set_flag(FLAG_H, false);
                1
            }
            0x3F => {
                let carry = self.get_flag(FLAG_C) == 0;
                self.set_flag(FLAG_C, carry);
                self.set_flag(FLAG_N, false);
                self.set_flag(FLAG_H, false);
                1
            }
            0x18 => {
                let offset = self.fetch_u8(bus) as i8;
                self.pc = self.pc.wrapping_add_signed(offset as i16);
                3
            }
            // JR cond, imm8
            0x20 | 0x28 | 0x30 | 0x38 => {
                let offset = self.fetch_u8(bus) as i8;
                let condition = self.cond_flag((instr >> 3) & 0b11);
                if condition {
                    self.pc = self.pc.wrapping_add_signed(offset as i16);
                    3
                } else {
                    2
                }
            }
            0x10 => {
                println!("St0p???");
                1
            }
            0x76 => {
                self.halted = true;
                1
            }
            // LD r8, r8
            0x40..=0x7F => {
                let source = instr & 0b111;
                let destination = (instr >> 3) & 0b111;
                let value = self.read_r8(source, bus);
                self.write_r8(destination, value, bus);
                if source == 6 || destination == 6 { 2 } else { 1 }
            }
            // ADD A, r8
            0x80..=0x87 => {
                let source = instr & 0b111;
                let value = self.read_r8(source, bus);
                self.add_to_a(value);
                if source == 6 { 2 } else { 1 }
            }
            // ADC A, r8
            0x88..=0x8F => {
                let source = instr & 0b111;
                let carry = self.get_flag(FLAG_C);
                let value = self.read_r8(source, bus).wrapping_add(carry);
                self.add_to_a(value);
                if source == 6 { 2 } else { 1 }
            }
            // SUB A, r8
            0x90..=0x97 => {
                let source = instr & 0b111;
                let value = self.read_r8(source, bus);
                self.sub_from_a(value);
                if source == 6 { 2 } else { 1 }
            }
            // SBC A, r8
            0x98..=0x9F => {
                let source = instr & 0b111;
                let carry = self.get_flag(FLAG_C);
                let value = self.read_r8(source, bus).wrapping_add(carry);
                self.sub_from_a(value);
                if source == 6 { 2 } else { 1 }
            }
            // AND A, r8
            0xA0..=0xA7 => {
                let source = instr & 0b111;
                self.a &= self.read_r8(source, bus);
                self.update_bitwise_flags(true);
                if source == 6 { 2 } else { 1 }
            }
            // XOR A, r8
            0xA8..=0xAF => {
                let source = instr & 0b111;
                self.a ^= self.read_r8(source, bus);
                self.update_bitwise_flags(false);
                if source == 6 { 2 } else { 1 }
            }
            // OR A, r8
            0xB0..=0xB7 => {
                let source = instr & 0b111;
                self.a |= self.read_r8(source, bus);
                self.update_bitwise_flags(false);
                if source == 6 { 2 } else { 1 }
            }
            // CP A, r8
            0xB8..=0xBF => {
                let source = instr & 0b111;
                let value = self.read_r8(source, bus);
                self.compare_a(value);
                if source == 6 { 2 } else { 1 }
            }
            // Arithmetic with immediate operands
            0xC6 => {
                let value = self.fetch_u8(bus);
                self.add_to_a(value);
                2
            }
            0xCE => {
                let carry = self.get_flag(FLAG_C);
                let value = self.fetch_u8(bus).wrapping_add(carry);
                self.add_to_a(value);
                2
            }
            0xD6 => {
                let value = self.fetch_u8(bus);
                self.sub_from_a(value);
                2
            }
            0xDE => {
                let carry = self.get_flag(FLAG_C);
                let value = self.fetch_u8(bus).wrapping_add(carry);
                self.sub_from_a(value);
                2
            }
            0xE6 => {
                let value = self.fetch_u8(bus);
                self.a &= value;
                self.update_bitwise_flags(true);
                2
            }
            0xEE => {
                let value = self.fetch_u8(bus);
                self.a ^= value;
                self.update_bitwise_flags(false);
                2
            }
            0xF6 => {
                let value = self.fetch_u8(bus);
                self.a |= value;
                self.update_bitwise_flags(false);
                2
            }
            0xFE => {
                let value = self.fetch_u8(bus);
                self.compare_a(value);
                2
            }
            // RET cond
            0xC0 | 0xC8 | 0xD0 | 0xD8 => {
                if self.cond_flag((instr >> 3) & 0b11) {
                    let low = bus.read(self.sp) as u16;
                    self.sp = self.sp.wrapping_add(1);
                    let high = bus.read(self.sp) as u16;
                    self.sp = self.sp.wrapping_add(1);
                    self.pc = (high << 8) | low;
                    5
                } else {
                    2
                }
            }
            0xC9 => {
                let low = bus.read(self.sp) as u16;
                self.sp = self.sp.wrapping_add(1);
                let high = bus.read(self.sp) as u16;
                self.sp = self.sp.wrapping_add(1);
                self.pc = (high << 8) | low;
                4
            }
            0xD9 => {
                println!("Implement reti!");
                4
            }
            // JP cond, imm16
            0xC2 | 0xCA | 0xD2 | 0xDA => {
                let address = self.fetch_u16(bus);
                if self.cond_flag((instr >> 3) & 0b11) {
                    self.pc = address;
                    4
                } else {
                    3
                }
            }
            0xC3 => {
                self.pc = self.fetch_u16(bus);
                4
            }
            0xE9 => {
                self.pc = self.hl();
                1
            }
            // CALL cond, imm16
            0xC4 | 0xCC | 0xD4 | 0xDC => {
                let condition = self.cond_flag((instr >> 3) & 0b11);
                let address = self.fetch_u16(bus);
                if condition {
                    let [low, high] = self.pc.to_le_bytes();
                    self.sp = self.sp.wrapping_sub(1);
                    bus.write(self.sp, high);
                    self.sp = self.sp.wrapping_sub(1);
                    bus.write(self.sp, low);
                    self.pc = address;
                    6
                } else {
                    3
                }
            }
            0xCD => {
                let address = self.fetch_u16(bus);
                let [low, high] = self.pc.to_le_bytes();
                self.sp = self.sp.wrapping_sub(1);
                bus.write(self.sp, high);
                self.sp = self.sp.wrapping_sub(1);
                bus.write(self.sp, low);
                self.pc = address;
                6
            }
            0xC7 | 0xCF | 0xD7 | 0xDF | 0xE7 | 0xEF | 0xF7 | 0xFF => {
                let target = (instr & 0b0011_1000) as u16;
                let [low, high] = self.pc.to_le_bytes();
                self.sp = self.sp.wrapping_sub(1);
                bus.write(self.sp, high);
                self.sp = self.sp.wrapping_sub(1);
                bus.write(self.sp, low);
                self.pc = target;
                4
            }
            // POP r16
            0xC1 | 0xD1 | 0xE1 | 0xF1 => {
                let register = (instr >> 4) & 0b11;
                let low = bus.read(self.sp);
                self.sp = self.sp.wrapping_add(1);
                let high = bus.read(self.sp);
                self.sp = self.sp.wrapping_add(1);
                self.write_r16stk(register, u16::from_le_bytes([low, high]));
                3
            }
            // PUSH r16
            0xC5 | 0xD5 | 0xE5 | 0xF5 => {
                let register = (instr >> 4) & 0b11;
                let [low, high] = self.read_r16stk(register).to_le_bytes();
                self.sp = self.sp.wrapping_sub(1);
                bus.write(self.sp, high);
                self.sp = self.sp.wrapping_sub(1);
                bus.write(self.sp, low);
                4
            }
            0xCB => {
                let cb_instruction = self.fetch_u8(bus);
                self.handle_cb(cb_instruction, bus)
            }
            0xE2 => {
                bus.write(0xFF00 + self.c as u16, self.a);
                2
            }
            0xE0 => {
                let addr_offset = self.fetch_u8(bus);
                bus.write(0xFF00 + addr_offset as u16, self.a);
                3
            }
            0xEA => {
                let address = self.fetch_u16(bus);
                bus.write(address, self.a);
                4
            }
            0xF2 => {
                self.a = bus.read(0xFF00 + self.c as u16);
                2
            }
            0xF0 => {
                self.a = self.fetch_u8(bus);
                3
            }
            0xFA => {
                let address = self.fetch_u16(bus);
                self.a = bus.read(address);
                4
            }
            0xE8 => {
                let offset = self.fetch_u8(bus);
                self.sp = self.add_sp_offset(offset);
                4
            }
            0xF8 => {
                let offset = self.fetch_u8(bus);
                let value = self.add_sp_offset(offset);
                [self.h, self.l] = value.to_be_bytes();
                3
            }
            0xF9 => {
                self.sp = self.hl();
                2
            }
            0xF3 => {
                self.ime = false;
                1
            }
            0xFB => {
                self.ime = true;
                1
            }
            _ => panic!("unimplemented opcode {instr:#04X}"),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::Memory;

    #[derive(Debug, PartialEq, Eq)]
    enum Access {
        Read(u16),
        Write(u16, u8)
    }

    struct TestMemory {
        bytes:[u8; 0x10000],
        accesses: Vec<Access>
    }

    impl Memory for TestMemory {
        fn read(& mut self, address: u16) -> u8 {

            self.accesses.push(Access::Read(address));
            self.bytes[address as usize]
        }

        fn write(&mut self, address: u16, value: u8) {
            self.bytes[address as usize] = value;
            self.accesses.push(Access::Write(address, value));
        }
    }

    fn system_with_program(program: &[u8]) -> (CPU, TestMemory) {
        let cpu = CPU::new();
        let mut memory = TestMemory {
            bytes: [0; 0x10000],
            accesses: Vec::new(),
        };
        let start = cpu.pc as usize;
        memory.bytes[start..start + program.len()].copy_from_slice(program);

        (cpu, memory)
    }

    #[test]
    fn ld_imm16_sp() {
        let (mut cpu, mut memory) = system_with_program(&[0x08, 0x34, 0x12]);
        cpu.sp = 0xF88F;
        let cycles = cpu.step(&mut memory);
        assert_eq!(memory.bytes[0x1234], 0x8F);
        assert_eq!(memory.bytes[0x1235], 0xF8);

        assert_eq!(cycles, 5)


    }


    //
            // 0x01 | 0x11 | 0x21 | 0x31 => {
            //     let val = self.fetch_u16(bus);
            //     self.write_r16((instr >> 4) & 0b11, val);
            //     3
            // }
    #[test]
    fn ld_r16_imm16() {
        let cases = [
            (0x01, [0x34, 0x12, 0x00, 0x00, 0x00, 0x00], 0x0000),
            (0x11, [0x00, 0x00, 0x34, 0x12, 0x00, 0x00], 0x0000),
            (0x21, [0x00, 0x00, 0x00, 0x00, 0x34, 0x12], 0x0000),
            (0x31, [0x00, 0x00, 0x00, 0x00, 0x00, 0x00], 0x3412),
        ];

        for (opcode, expected_registers, expected_sp) in cases {
            let (mut cpu, mut memory) =
                system_with_program(&[opcode, 0x12, 0x34]);

            let cycles = cpu.step(&mut memory);

            assert_eq!(
                [cpu.b, cpu.c, cpu.d, cpu.e, cpu.h, cpu.l],
                expected_registers,
                "incorrect register state after opcode {opcode:#04X}",
            );
            assert_eq!(
                cpu.sp, expected_sp,
                "incorrect SP after opcode {opcode:#04X}",
            );
            assert_eq!(cycles, 3, "incorrect cycles for opcode {opcode:#04X}");
            assert_eq!(cpu.pc, 0x0103, "incorrect PC for opcode {opcode:#04X}");
        }
    }



            // LD [r16mem], A
            // 0x02 | 0x12 | 0x22 | 0x32 => {
            //     let addr = self.read_r16mem((instr >> 4) & 0b11);
            //     bus.write(addr, self.a);
            //     2
            // }
    #[test]
    fn ld_a_r16mem() {
        let cases = [
            (0x0A, [0x34, 0x12, 0x00, 0x00, 0x00, 0x00]),
            (0x1A, [0x00, 0x00, 0x34, 0x12, 0x00, 0x00]),
            (0x2A, [0x00, 0x00, 0x00, 0x00, 0x34, 0x12]),
            (0x3A, [0x00, 0x00, 0x00, 0x00, 0x34, 0x12]),
        ];

        for (opcode, expected_registers) in cases {
            let (mut cpu, mut memory) =
                system_with_program(&[opcode]);
            memory.bytes[0x3412] = 69; // aye

            [cpu.b, cpu.c, cpu.d, cpu.e, cpu.h, cpu.l] = expected_registers;

            let cycles = cpu.step(&mut memory);

            assert_eq!(cpu.a, 69);

            if cpu.h != 0 || cpu.l != 0 {
                assert_eq!(cpu.h, 0x34);
                assert!(cpu.l == 0x11 || cpu.l == 0x13);
            }

            assert_eq!(cycles, 2, "incorrect cycles for opcode {opcode:#04X}"); 
        }
    }

    #[test]
    fn ld_r16mem_a() {
        let cases = [
            (0x02, [0x34, 0x12, 0x00, 0x00, 0x00, 0x00]),
            (0x12, [0x00, 0x00, 0x34, 0x12, 0x00, 0x00]),
            (0x22, [0x00, 0x00, 0x00, 0x00, 0x34, 0x12]),
            (0x32, [0x00, 0x00, 0x00, 0x00, 0x34, 0x12]),
        ];

        for (opcode, expected_registers) in cases {
            let (mut cpu, mut memory) =
                system_with_program(&[opcode]);
            cpu.a = 70;
            [cpu.b, cpu.c, cpu.d, cpu.e, cpu.h, cpu.l] = expected_registers;

            let cycles = cpu.step(&mut memory);

            assert_eq!(memory.bytes[0x3412], 70);

            if cpu.h != 0 || cpu.l != 0 {
                assert_eq!(cpu.h, 0x34);
                assert!(cpu.l == 0x11 || cpu.l == 0x13);
            }

            assert_eq!(cycles, 2, "incorrect cycles for opcode {opcode:#04X}"); 
        }
    }


    #[test]
    fn step_returns_fixed_instruction_cycles() {
        let (mut cpu, mut memory) = system_with_program(&[
            0x00,             // NOP: 1
            0x06, 0x42,       // LD B, imm8: 2
            0x01, 0x34, 0x12, // LD BC, imm16: 3
        ]);

        assert_eq!(cpu.step(&mut memory), 1);
        assert_eq!(cpu.step(&mut memory), 2);
        assert_eq!(cpu.step(&mut memory), 3);
    }

    #[test]
    fn jr_condition_cycles_depend_on_whether_branch_is_taken() {
        let cases = [
            (0x20, FLAG_Z, false), // JR NZ
            (0x28, FLAG_Z, true),  // JR Z
            (0x30, FLAG_C, false), // JR NC
            (0x38, FLAG_C, true),  // JR C
        ];

        for (opcode, flag, taken_when_set) in cases {
            let (mut taken_cpu, mut taken_memory) =
                system_with_program(&[opcode, 0x02]);
            taken_cpu.set_flag(flag, taken_when_set);

            assert_eq!(
                taken_cpu.step(&mut taken_memory),
                3,
                "incorrect taken cycles for opcode {opcode:#04X}",
            );
            assert_eq!(
                taken_cpu.pc, 0x0104,
                "incorrect taken PC for opcode {opcode:#04X}",
            );

            let (mut untaken_cpu, mut untaken_memory) =
                system_with_program(&[opcode, 0x02]);
            untaken_cpu.set_flag(flag, !taken_when_set);

            assert_eq!(
                untaken_cpu.step(&mut untaken_memory),
                2,
                "incorrect untaken cycles for opcode {opcode:#04X}",
            );
            assert_eq!(
                untaken_cpu.pc, 0x0102,
                "incorrect untaken PC for opcode {opcode:#04X}",
            );
        }
    }

    #[test]
    fn register_and_hl_operands_have_different_cycles() {
        let (mut register_cpu, mut register_memory) = system_with_program(&[0x80]);
        assert_eq!(register_cpu.step(&mut register_memory), 1); // ADD A, B

        let (mut memory_cpu, mut memory) = system_with_program(&[0x86]);
        memory_cpu.h = 0xC0;
        memory_cpu.l = 0x00;
        assert_eq!(memory_cpu.step(&mut memory), 2); // ADD A, [HL]
    }

    #[test]
    fn cb_cycles_distinguish_register_bit_and_hl_operations() {
        let (mut register_cpu, mut register_memory) = system_with_program(&[0xCB, 0x00]);
        assert_eq!(register_cpu.step(&mut register_memory), 2); // RLC B

        let (mut bit_cpu, mut bit_memory) = system_with_program(&[0xCB, 0x46]);
        bit_cpu.h = 0xC0;
        bit_cpu.l = 0x00;
        assert_eq!(bit_cpu.step(&mut bit_memory), 3); // BIT 0, [HL]

        let (mut rotate_cpu, mut rotate_memory) = system_with_program(&[0xCB, 0x06]);
        rotate_cpu.h = 0xC0;
        rotate_cpu.l = 0x00;
        assert_eq!(rotate_cpu.step(&mut rotate_memory), 4); // RLC [HL]
    }


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
