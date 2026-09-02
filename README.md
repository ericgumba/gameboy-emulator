# Game Boy Emulator
 
An educational Nintendo Game Boy (DMG) emulator written in Rust.

This project is currently focused on the Sharp SM83 CPU core. It can load a ROM
into a cartridge and execute parts of the unprefixed instruction set, but it is
not yet capable of running or displaying a game.

## Project status

Implemented or in progress:

- SM83 register storage: `A`, `F`, `B`, `C`, `D`, `E`, `H`, `L`, `SP`, and `PC`
- Combined `BC`, `DE`, and `HL` register access
- Instruction and little-endian immediate-value fetching
- CPU flag manipulation for zero, subtract, half-carry, and carry
- Portions of the unprefixed opcode table, including loads, arithmetic, jumps,
  returns, rotates, and miscellaneous accumulator operations
- Pattern-based decoding for the `0x40..=0x7F` register-load block
- Basic `HALT` state tracking
- Loading ROM bytes from a file
- Unit tests for the `DAA` instruction

Major pieces still missing:

- A complete and verified CPU instruction set
- The `0xCB`-prefixed instruction table
- Accurate instruction timing and machine-cycle accounting
- Interrupts, the interrupt master enable flag, and complete `HALT`/`STOP`
  behavior
- Writable memory and the full Game Boy memory map
- Cartridge memory-bank controllers and external cartridge RAM
- Timers, serial I/O, joypad input, DMA, and hardware registers
- PPU/video output and APU/audio output
- A working emulator loop in the binary target

`Bus::write` is currently `unimplemented!()`. Any instruction that writes to
memory will therefore panic. The code should be treated as a work in progress,
not as a compatible emulator.

## Code layout

```text
src/
├── cpu.rs        CPU state, flags, instruction helpers, and opcode dispatch
├── bus.rs        Address-bus interface between the CPU and cartridge
├── cartridge.rs  ROM file loading and ROM byte reads
├── lib.rs        Public library modules
└── main.rs       Placeholder binary entry point
```

### CPU

`CPU::step` executes one instruction:

1. Return immediately if the CPU is halted.
2. Read the opcode at `PC` through the bus.
3. Increment `PC`.
4. Decode and execute the opcode.
5. Fetch additional immediate bytes when required.

The CPU starts at `0x0100`, the cartridge entry point used when boot-ROM
execution is skipped. The rest of the initial register state is not yet a full
post-boot hardware state.

The Game Boy flag register uses its upper four bits:

| Bit | Flag | Meaning |
| ---: | :--- | :------ |
| 7 | `Z` | The result was zero |
| 6 | `N` | The last operation was a subtraction |
| 5 | `H` | Carry or borrow across bit 3 |
| 4 | `C` | Carry or borrow across bit 7 |

Several opcode families encode registers directly in their bits. For example,
the register-to-register load block has the form:

```text
01 ddd sss
   │   └── source register
   └────── destination register
```

The three-bit register selector is:

| Value | Register |
| ----: | :------- |
| `0` | `B` |
| `1` | `C` |
| `2` | `D` |
| `3` | `E` |
| `4` | `H` |
| `5` | `L` |
| `6` | Memory at `(HL)` |
| `7` | `A` |

For example, `0x51` is `01 010 001`, so it decodes to `LD D, C`. The
`read_r8` and `write_r8` helpers allow the entire load family to share one
implementation instead of using a separate match arm for every register pair.
Opcode `0x76` is the exception: it represents `HALT`.

### Bus

The bus is intended to own the system components and route CPU reads and writes
according to the Game Boy memory map. At present, it only forwards reads to the
cartridge. RAM and memory-mapped hardware still need to be added.

### Cartridge

`Cartridge::new` reads a ROM file into a `Vec<u8>`. `Cartridge::read` currently
indexes that vector directly, so there is no header parsing, bounds handling,
RAM, or bank switching yet.

## Building and testing

The crate uses Rust 2024 edition. Install a recent stable Rust toolchain, then
check the library with:

```bash
cargo check --lib
```

Run the current unit tests with:

```bash
cargo test --lib
```

The binary in `src/main.rs` is not wired into a working CPU loop yet, so
`cargo run` and the full `cargo test` command are not expected to succeed at
this stage.

## Test ROMs

The repository contains Blargg's CPU instruction test ROMs under
`blarg/cpu_instrs/`. The individual ROMs are useful because each one targets a
smaller instruction category, making failures easier to diagnose than the
combined `cpu_instrs.gb` ROM.

These ROMs cannot pass yet. They require substantially more CPU coverage,
writable work RAM, interrupts and timing behavior, and a way to capture output
from the serial registers or display.

## Suggested development order

1. Implement writable work RAM and route the basic memory map through `Bus`.
2. Add table-driven tests for every implemented opcode, including flags and
   boundary values.
3. Complete all unprefixed CPU instructions.
4. Implement the `0xCB`-prefixed instructions.
5. Track instruction cycles and implement timers and interrupts.
6. Capture serial output and begin running the individual Blargg ROMs.
7. Add cartridge banking, PPU rendering, input, and audio.

Useful behavior references include [Pan Docs](https://gbdev.io/pandocs/) and
the [Game Boy opcode table](https://gbdev.io/gb-opcodes/optables/).
