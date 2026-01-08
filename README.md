# Setun Emulator

An emulator for the Soviet Setun computer (1958), the only balanced ternary computer ever built for practical use.

## What is Balanced Ternary?

Regular computers use binary (0 and 1). The Setun uses balanced ternary with three values:
- N = -1 (negative)
- O = 0 (zero)  
- P = +1 (positive)

This makes negative numbers natural - no special handling needed.

## Quick Start

Run the test program:
```
cargo run -- run examples/add_test.asm --trace
```

This loads two numbers (42 and 17), adds them, and outputs 59.

## Commands

```
cargo run -- run <file>           Run a program
cargo run -- run <file> --trace   Run with step-by-step output
cargo run -- debug <file>         Interactive debugger (TUI)
cargo run -- asm <file>           Assemble .asm to .trom
cargo run -- disasm <file>        Disassemble .trom to text
cargo run -- test                 Run self-tests
```

## Writing Assembly

Example program that adds two numbers:

```asm
; Load value from address 4, add value from address 5, halt
    LDA 4
    ADD 5  
    STA 6
    HLT

; Data follows code
    DAT 42
    DAT 17
    DAT 0
```

## The Debugger

Press `s` to step, `r` to run, `b` for breakpoint, `q` to quit.

Arrow keys scroll the memory view.

## Technical Details

- 162 memory cells (9 trits each)
- 5 registers: S (accumulator), R (multiplier), F (index), C (program counter), omega (sign)
- 24 instructions: arithmetic, data transfer, jumps, shifts

## Building

```
cargo build --release
```

## Web Demo

The web version runs in browsers using WebAssembly. See the `web/` folder.

## License

MIT
