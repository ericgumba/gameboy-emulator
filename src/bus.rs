use crate::cartridge::Cartridge;

pub trait Memory {
    fn read(&mut self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);
}
pub struct Bus {
    cartridge: Cartridge,
    vram: [u8; 0x2000],
    wram: [u8; 0x2000],
    oam: [u8; 0xA0],
    io: [u8; 0x80],
    hram: [u8; 0x7F],
    interrupt_enable: u8,
}

impl Bus {
    pub fn new(cartridge: Cartridge) -> Self {
        Self {
            cartridge,
            vram: [0; 0x2000],
            wram: [0; 0x2000],
            oam: [0; 0xA0],
            io: [0; 0x80],
            hram: [0; 0x7F],
            interrupt_enable: 0,
        }
    }
}

impl Memory for Bus {
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7FFF => self.cartridge.read(addr),
            0x8000..=0x9FFF => self.vram[(addr - 0x8000) as usize],
            0xA000..=0xBFFF => self.cartridge.read(addr),
            0xC000..=0xDFFF => self.wram[(addr - 0xC000) as usize],
            0xE000..=0xFDFF => self.wram[(addr - 0xE000) as usize],
            0xFE00..=0xFE9F => self.oam[(addr - 0xFE00) as usize],
            0xFEA0..=0xFEFF => 0xFF,
            0xFF00..=0xFF7F => self.io[(addr - 0xFF00) as usize],
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize],
            0xFFFF => self.interrupt_enable,
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x7FFF => self.cartridge.write(addr, value),
            0x8000..=0x9FFF => self.vram[(addr - 0x8000) as usize] = value,
            0xA000..=0xBFFF => self.cartridge.write(addr, value),
            0xC000..=0xDFFF => self.wram[(addr - 0xC000) as usize] = value,
            0xE000..=0xFDFF => self.wram[(addr - 0xE000) as usize] = value,
            0xFE00..=0xFE9F => self.oam[(addr - 0xFE00) as usize] = value,
            0xFEA0..=0xFEFF => {}
            0xFF00..=0xFF7F => self.io[(addr - 0xFF00) as usize] = value,
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize] = value,
            0xFFFF => self.interrupt_enable = value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_bus() -> Bus {
        let mut rom = vec![0; 0x8000];
        rom[0x0000] = 0x10;
        rom[0x3FFF] = 0x11;
        rom[0x4000] = 0x12;
        rom[0x7FFF] = 0x13;

        Bus::new(Cartridge::from_bytes(rom))
    }

    #[test]
    fn reads_both_cartridge_rom_regions() {
        let mut bus = test_bus();

        assert_eq!(bus.read(0x0000), 0x10);
        assert_eq!(bus.read(0x3FFF), 0x11);
        assert_eq!(bus.read(0x4000), 0x12);
        assert_eq!(bus.read(0x7FFF), 0x13);
    }

    #[test]
    fn selects_each_writable_memory_region() {
        let mut bus = test_bus();
        let cases = [
            (0x8000, 0x20),
            (0x9FFF, 0x21),
            (0xA000, 0x30),
            (0xBFFF, 0x31),
            (0xC000, 0x40),
            (0xDFFF, 0x41),
            (0xFE00, 0x50),
            (0xFE9F, 0x51),
            (0xFF00, 0x60),
            (0xFF7F, 0x61),
            (0xFF80, 0x70),
            (0xFFFE, 0x71),
            (0xFFFF, 0x80),
        ];

        for (addr, value) in cases {
            bus.write(addr, value);
        }

        for (addr, expected) in cases {
            assert_eq!(
                bus.read(addr),
                expected,
                "incorrect region selected for address {addr:#06X}",
            );
        }
    }

    #[test]
    fn echo_ram_mirrors_work_ram() {
        let mut bus = test_bus();

        bus.write(0xC123, 0x42);
        assert_eq!(bus.read(0xE123), 0x42);

        bus.write(0xFDFF, 0x84);
        assert_eq!(bus.read(0xDDFF), 0x84);
    }

    #[test]
    fn unusable_memory_reads_as_ff_and_ignores_writes() {
        let mut bus = test_bus();

        bus.write(0xFEA0, 0x12);
        bus.write(0xFEFF, 0x34);

        assert_eq!(bus.read(0xFEA0), 0xFF);
        assert_eq!(bus.read(0xFEFF), 0xFF);
    }

    #[test]
    fn writes_to_rom_do_not_modify_rom_data() {
        let mut bus = test_bus();

        bus.write(0x0000, 0xFF);
        bus.write(0x4000, 0xFF);

        assert_eq!(bus.read(0x0000), 0x10);
        assert_eq!(bus.read(0x4000), 0x12);
    }
}
