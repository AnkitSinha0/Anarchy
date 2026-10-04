# Decisions

Decisions that would survive an interview follow-up ("why not X instead?"). Logged the day they are made.

## [2026-10-03] Simulated time is derived from instructions retired, not the host clock
Why: Phase 5 verifies the RTL core by diffing its per-instruction trace against the simulator. That only works if the same program produces the same trace every run. If `mtime` reads the host clock, timer interrupts land on different instructions each run, and every trace diverges at the first interrupt. The host instead throttles the simulator to wall-clock speed, so DOOM still runs at real speed when the sim is fast enough.
Alternative considered: `mtime` driven by the host's monotonic clock — rejected because it makes runs non-reproducible. That breaks trace diffing, the headless `-timedemo` regression test, and "run it again with `--trace`" debugging.
Would flip if: the simulator can't reach real-time speed for DOOM and we decide playability matters more than reproducibility (then wall-clock mode becomes an opt-in flag, with deterministic mode still the default for tests).

## [2026-10-03] RV32IM + Zicsr + Zifencei, no C (compressed) extension
Why: Fixed 32-bit instructions keep the decoder, the trace format and later the RTL fetch stage simple: every instruction is aligned at 4 bytes and fetch is one word. Zifencei is included because `rv32ui` contains the `fence_i` test and current GCC requires it to be named in `-march`.
Alternative considered: RV32IMC, which gives about 25-30% smaller binaries and lets us use Rust's `riscv32imc` target — rejected because 16-bit instructions mean fetch can straddle word boundaries, which complicates both the simulator and the FPGA fetch stage for a size win DOOM doesn't need (32 MB DRAM).
Would flip if: the FPGA board's memory turns out too small or too slow and code size starts to matter.

## [2026-10-03] Rust on Windows, C/asm toolchain in WSL
Why: The simulator needs a native window (`minifb`), which is simplest on the Windows host. The prebuilt bare-metal RISC-V GCC and picolibc packages are one `apt install` away in WSL Ubuntu, and WSL sees the same files through `/mnt/d`.
Alternative considered: a Windows build of the RISC-V GCC toolchain (e.g. xPack) — rejected because picolibc and the riscv-tests Makefiles assume a Unix shell anyway.
Alternative considered: everything inside WSL — rejected because the `minifb` window would depend on WSLg.
Would flip if: the CI runner (Linux) becomes the main way the project is built, at which point WSL-only is simpler.

## [2026-10-04] Decode into a Rust enum with named fields, not raw-word helpers at execute time
Why: `decode()` turns the scrambled bit fields into `Instr::Addi { rd, rs1, imm }` once, in the `isa` crate. The CPU, the tracer and the disassembler then all match on the same enum and never touch bit positions. The compiler checks every `match` is exhaustive, so a newly added instruction can't be silently skipped by the executor.
Alternative considered: keep the `u32` and extract fields inside each execute arm — rejected because it duplicates the bit-twiddling in every consumer, and immediate sign extension is exactly where first implementations go wrong.
Would flip if: profiling shows decode is the bottleneck for DOOM. The fix then is a pre-decoded instruction cache in front of the enum, not dropping it.

## [2026-10-04] DRAM is an ordinary `Device` on the bus, no fast path
Why: Every access goes through one `Bus::find` → `dyn Device` path, so faults, bounds checks and (later) tracing behave the same for RAM and MMIO. It's the simplest correct design.
Alternative considered: a direct `Vec<u8>` fast path for DRAM addresses in the CPU — rejected for now because speed doesn't matter until DOOM, and two access paths double the places a bounds or endianness bug can hide.
Would flip if: Phase 4 measurements show the simulator can't hold real-time DOOM speed and bus dispatch shows up in the profile.
