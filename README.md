# Game Boy Emulator
 
An educational Nintendo Game Boy (DMG) emulator written in Rust.

This project is currently focused on the Sharp SM83 CPU core. It can load a ROM
into a cartridge and execute parts of the unprefixed instruction set, but it is
not yet capable of running or displaying a game.

Once the instruction set is implemented, focus on making it verifiably correct before starting graphics.

## Recommended order

### 1. Finish writable memory and the bus


- VRAM
- Work RAM
- OAM
- I/O registers
- High RAM
- Interrupt-enable register

The official address ranges are documented in [Pan Docs’ memory map](https://gbdev.io/pandocs/Memory_Map.html).

The basic design will look something like:

```rust
pub struct Bus {
    cartridge: Cartridge,
    vram: [u8; 0x2000],
    wram: [u8; 0x2000],
    oam: [u8; 0xA0],
    io: [u8; 0x80],
    hram: [u8; 0x7F],
    interrupt_enable: u8,
}
```

Writes mutate memory, so change the API to:

```rust
pub fn write(&mut self, addr: u16, value: u8)
```

and CPU execution to:

```rust
pub fn step(&mut self, bus: &mut Bus)
```

Start with ordinary arrays and simple address routing. Hardware-specific restrictions can be added later.

### 2. Thoroughly test the CPU

“Every opcode has a match arm” does not necessarily mean the CPU is correct. Test:

- Result values
- `Z`, `N`, `H`, and `C` flags
- `PC` advancement
- Stack behavior
- Memory side effects
- Taken and untaken conditional instructions
- Boundary values such as `0x00`, `0x0F`, `0x7F`, `0x80`, and `0xFF`
- 16-bit wrapping
- All `0xCB`-prefixed instructions

Before considering the CPU complete, audit your bit selectors. A selector helper expects `0..=3` or `0..=7`, so masked bits usually need shifting:

```rust
let r16 = (instr >> 4) & 0b11;
let r8 = (instr >> 3) & 0b111;
```

Also consider changing `get_flag()` to return `bool`. Its current masked `u8` return value makes mistakes such as this easy:

```rust
self.get_flag(FLAG_Z) == 1
```

A set `Z` flag is actually `0x80`, not `1`.

```rust
fn get_flag(&self, flag: u8) -> bool {
    self.f & flag != 0
}
```

Then condition checks become clearer:

```rust
1 => self.get_flag(FLAG_Z),
3 => self.get_flag(FLAG_C),
```

### 3. Track instruction timing

Change `CPU::step()` to return the number of cycles consumed:

```rust
pub fn step(&mut self, bus: &mut Bus) -> u8 {
    let opcode = self.fetch_u8(bus);

    match opcode {
        0x00 => 1, // NOP: one machine cycle
        // ...
    }
}
```

Conditional instructions have different timing depending on whether the branch was taken.

Choose one unit—machine cycles or clock cycles—and use it consistently. For a DMG Game Boy:

```text
1 machine cycle = 4 clock cycles/dots
```

Other components then advance from the CPU’s elapsed cycles:

```rust
let cycles = cpu.step(&mut bus);
bus.tick(cycles);
```

Timing drives timers, interrupts, graphics, audio, and DMA, so it should be established before those components.

### 4. Add serial test output

Implement minimal handling for:

```text
0xFF01: SB — serial data
0xFF02: SC — serial control
```

Blargg’s tests write output characters to `SB` and then write `$81` to `SC`, making it possible to print test results in your terminal without having a PPU. [Blargg’s CPU tests](https://github.com/retrio/gb-test-roms/blob/master/cpu_instrs/readme.txt) exercise instructions with boundary values and verify that unrelated registers remain unchanged.

Start with the individual ROMs already in your repository:

```text
blarg/cpu_instrs/individual/
```

Then try the combined ROM:

```text
blarg/cpu_instrs/cpu_instrs.gb
```

Your first major milestone should be:

```text
Blargg cpu_instrs: Passed
```

### 5. Implement timers and interrupts

Add:

- `DIV`
- `TIMA`
- `TMA`
- `TAC`
- Interrupt request register `IF`
- Interrupt enable register `IE`
- Interrupt master enable state
- `DI`, `EI`, and delayed `EI` behavior
- `RETI`
- Interrupt vectors
- Proper `HALT` behavior

The timer advances alongside CPU cycles, and `TIMA` overflow reloads `TMA` and requests a timer interrupt. [Pan Docs timer reference](https://gbdev.io/pandocs/Timer_and_Divider_Registers.html)

Afterward, use the [Mooneye acceptance tests](https://github.com/Gekkio/mooneye-test-suite) for timers, interrupts, hardware sequencing, and model-specific behavior.

### 6. Implement cartridge types

Begin with:

1. ROM-only cartridges
2. Cartridge-header parsing
3. MBC1
4. MBC3 and its real-time clock
5. MBC5
6. Battery-backed save RAM

Do not try to implement every memory-bank controller immediately. ROM-only and MBC1 are enough for early progress.

### 7. Implement the PPU

Build graphics incrementally:

1. LCD registers
2. `LY` scanline progression
3. PPU modes
4. Background tiles
5. Scrolling
6. Window layer
7. Sprites
8. Palettes
9. OAM DMA
10. Access restrictions during PPU modes

Render into a simple framebuffer:

```rust
pub struct Ppu {
    framebuffer: [u8; 160 * 144],
}
```

Get a static background visible first. Pixel-perfect timing can improve afterward, although PPU timing eventually matters because VRAM and OAM access depend on the current mode. [Pan Docs rendering reference](https://gbdev.io/pandocs/Rendering.html)

### 8. Add a frontend and input

Once the PPU produces complete frames:

- Open a `160 × 144` window
- Scale it by an integer such as `4×`
- Copy the framebuffer to the window
- Map keyboard buttons to the joypad register
- Limit execution to approximately the Game Boy’s frame rate

Keep the emulator core independent of the windowing library. The core should produce frames and accept button state; the frontend should display those frames.

### 9. Add audio last

The APU is relatively self-contained but nuanced. You can play and test games without audio, so it is usually more productive to implement it after CPU, bus, interrupts, timers, PPU, and input.

## Practical milestone list

A satisfying progression would be:

```text
CPU unit tests pass
        ↓
Blargg CPU tests pass in terminal
        ↓
Timer and interrupt tests pass
        ↓
First static background appears
        ↓
Sprites and scrolling work
        ↓
Joypad input works
        ↓
First ROM-only or MBC1 game runs
        ↓
Audio and accuracy improvements
```

For your codebase specifically, I would implement writable WRAM/HRAM and `Bus::write()` next. Without that, stack instructions, calls, returns, test ROMs, and nearly every real program remain blocked.

Useful behavior references include [Pan Docs](https://gbdev.io/pandocs/) and
the [Game Boy opcode table](https://gbdev.io/gb-opcodes/optables/).

