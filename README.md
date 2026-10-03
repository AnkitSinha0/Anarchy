# Anarchy

My own RISC-V CPU, my own OS, running DOOM. Built from scratch.

- **CPU:** an RV32IM simulator written in Rust. It is the reference for everything above it.
- **OS:** a `no_std` Rust kernel with M-mode/U-mode separation, traps, a timer and syscalls.
- **DOOM:** `doomgeneric`, compiled with stock RISC-V GCC, running as a user program on the kernel.
- **Hardware (later):** the same kernel and DOOM binary on my own RISC-V core on an FPGA, verified by diffing per-instruction traces against the simulator.

## Roadmap
| Phase | Milestone | Status |
|---|---|---|
| 1 | Simulator passes the official `riscv-tests` (rv32ui, rv32um, rv32mi) | in progress |
| 2 | Bare-metal C prints text and draws to a framebuffer window | |
| 3 | Rust kernel runs a C user program through syscalls | |
| 4 | DOOM playable in the simulator | |
| 5 | DOOM on my own RTL core on an FPGA | |

## Build
```
cargo build --release
cargo test
cargo run -p sim --release -- <program.bin|.elf> [--trace]
```
