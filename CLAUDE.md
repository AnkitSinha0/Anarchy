# Anarchy

A from-scratch computer stack whose finish line is: **boot my own OS on my own RISC-V CPU simulator and run DOOM.**

Later: the same kernel and DOOM binary on my own RISC-V core on an FPGA.
Side track (after DOOM): my own small programming language that compiles to RV32.

## Scope rules (important)
- Build only what DOOM needs. No paging, no S-mode, no scheduler, no filesystem, no networking, no multitasking, no AI.
- Do not add features, abstractions, or dependencies beyond the current phase.
- If a design decision affects later phases, stop and ask me before implementing.
- I am learning this stack (and Rust). When you introduce a new concept (CSR, trap, ELF segment, syscall ABI, etc.), explain it in 2-4 sentences in your reply.

## Architecture
- ISA: RV32IM, plus Zicsr and Zifencei. No C (compressed) extension. Privilege modes: M (kernel) and U (DOOM). No MMU.
- The simulator is the source of truth. Hardware (RTL) later must match it exactly.
- Kernel is written in Rust (`no_std`). DOOM and libc are C, compiled with the stock RISC-V GCC toolchain.

### Memory map (QEMU `virt`-style)
| Address       | Device                          |
|---------------|---------------------------------|
| `0x0200_0000` | CLINT (mtime, mtimecmp)         |
| `0x1000_0000` | UART (write byte = print char)  |
| `0x1000_1000` | Keyboard (scancode queue)       |
| `0x2000_0000` | Framebuffer, 320x200, 32-bit    |
| `0x8000_0000` | DRAM, 32 MB; programs load here |

### Workspace layout
```
anarchy/
  isa/       # instruction encodings, Instr enum, decode(). Shared by sim, disassembler, debugger
  sim/       # Cpu, Bus, Device trait, devices, trap logic, trace, minifb window
  kernel/    # no_std Rust kernel (Phase 3)
  user/      # C user programs, libc stubs, doomgeneric port (Phases 2-4)
  lang/      # my language's compiler (side track, after Phase 4)
  tests/     # riscv-tests runner and test programs
  JIT/       # learning docs, one brief + one write-up per phase
```

### Core simulator abstractions
- `trait Device { fn read(&mut self, off: u32, size: u8) -> u32; fn write(&mut self, off: u32, size: u8, val: u32); fn tick(&mut self); fn irq(&self) -> bool; }`
- `Bus`: list of `(base, len, Box<dyn Device>)`; routes addresses; unmapped access raises an access-fault trap.
- `Cpu`: `regs: [u32; 32]` (x0 hardwired to 0), `pc`, CSR file, privilege mode. `step(&mut Bus) -> Result<(), Trap>`.
- Main loop: `cpu.step()` -> `devices.tick()` -> check interrupts.
- `--trace` flag prints `pc, instruction (disassembled), register written, value` per step.

## Locked decisions (see DECISIONS.md for full reasoning)
1. **Deterministic time.** `mtime` advances at a fixed rate per instruction retired, never from the host clock. The host throttles the simulator to wall-clock speed so DOOM runs at real speed. Two runs of the same program must produce identical traces, because Phase 5 verifies the RTL by diffing them.
2. **No C extension.** RV32IM + Zicsr + Zifencei only. GCC: `-march=rv32im_zicsr_zifencei -mabi=ilp32`. Rust kernel target: `riscv32im-unknown-none-elf` (never `riscv32imc`).
3. **Toolchain split.** Rust crates build natively on Windows. C, assembly and riscv-tests build in WSL Ubuntu against the same files via `/mnt/d/das Projekt/Anarchy`.
4. **FPGA board: open.** Must be decided before Phase 3 ends. It needs >= 32 MB RAM to keep the memory map. Candidates: ULX3S, Tang Primer 20K, Arty A7.

## Toolchain
- Windows: Rust stable (`cargo`, `rustc`). Phase 3 adds `rustup target add riscv32im-unknown-none-elf`.
- WSL Ubuntu, one-time:
  ```
  sudo apt update
  sudo apt install gcc-riscv64-unknown-elf binutils-riscv64-unknown-elf picolibc-riscv64-unknown-elf make
  ```
- Assemble a raw binary (example):
  ```
  riscv64-unknown-elf-as -march=rv32im_zicsr_zifencei -mabi=ilp32 prog.s -o prog.o
  riscv64-unknown-elf-ld -m elf32lriscv -Ttext=0x80000000 prog.o -o prog.elf
  riscv64-unknown-elf-objcopy -O binary prog.elf prog.bin
  ```
- Calling WSL from Git Bash: prefix with `MSYS_NO_PATHCONV=1` and `cd` inside the bash command (`wsl --cd` fails on the space in "das Projekt").

## Phases (do not skip ahead)

### Phase 1: Simulator
1. Workspace with `isa` and `sim` crates. Registers, pc, byte-array RAM, fetch-decode-execute loop.
2. Load a raw `.bin` at `0x8000_0000`. Implement `ADDI` and `ADD`; run a 3-line asm program; confirm `x3 = 5`.
3. Full RV32I, then M extension. Unit test every instruction.
4. UART device. Instruction trace and register dump.
5. Zicsr, `fence.i`, traps (`ecall`, `mret`, illegal instruction), CLINT timer interrupt.
6. Run the official `riscv-tests` (rv32ui, rv32um, rv32mi). **Done when all pass.**

### Phase 2: Bare-metal C
1. ELF loader in the simulator.
2. C hello world built with `riscv64-unknown-elf-gcc -march=rv32im_zicsr_zifencei -mabi=ilp32` plus picolibc, linker script at `0x8000_0000`.
3. Framebuffer device shown in a `minifb` window; keyboard from the same window.
4. **Done when a C program prints text and draws a test pattern.**

### Phase 3: Minimal kernel (Rust)
1. Boot in M-mode, trap handler with a `TrapFrame`, timer interrupt, UART/framebuffer/keyboard drivers, bump or linked-list heap.
2. ELF loader that runs one user program in U-mode.
3. Syscalls via `ecall`: `write`, `exit`, `sbrk`, `get_ticks_ms`, `sleep_ms`, `read_key`, `fb_blit`, `open`, `read`, `seek` (files served from an in-memory ramdisk holding `DOOM1.WAD`).
4. libc stubs (`_write`, `_sbrk`, `_read`, etc.) that call these syscalls.
5. **Done when a C user program runs under the kernel using syscalls.**

### Phase 4: DOOM
1. Port `doomgeneric`: implement `DG_Init`, `DG_DrawFrame`, `DG_SleepMs`, `DG_GetTicksMs`, `DG_GetKey`, `DG_SetWindowTitle` on top of the syscalls. Build with `DOOMGENERIC_RESX=320`, `DOOMGENERIC_RESY=200`.
2. Embed the shareware `DOOM1.WAD` in the ramdisk.
3. Headless `-timedemo demo1` run as an end-to-end regression test (deterministic, so the result must be identical every run).
4. **Done when DOOM is playable in the simulator window.**

### Phase 5 (later): FPGA
RTL core (multi-cycle first, then 5-stage pipeline). Verify by diffing per-instruction traces against the simulator.
The framebuffer is scanned out of DRAM. Playable speed on hardware needs the pipelined core.

### Side track: my language (start after Phase 4)
Lexer -> parser -> type checker -> RV32 assembly codegen, assembled with GNU `as`/`ld`.
v1 features only: integers, pointers, functions, `if`/`while`, structs, calls into C via the standard RISC-V calling convention.
Goal: write a small program (e.g. a shell or mini game) that runs on the Anarchy OS. Do **not** use it for the kernel or DOOM.

## Working rules
- Test each layer before building the next. Never write code for a later phase while the current one's "done" check fails.
- Every new instruction or device gets a test.
- Keep commits small, one milestone each, with a clear message.
- When debugging, use the trace first; don't guess.
- After finishing a milestone, update the "Current status" section below.
- **Engineering decisions:** follow `d:\das Projekt\Engineering_Decisions_WorkFlow.md`. Log to `DECISIONS.md` as decisions happen, write the phase entry in `RETRO.md` at each phase boundary, and promote at most 1-2 to an Engineering Desk post (drafts in `engineering-desk/`) only when there is a real contested trade-off or failure.

## Commands
- Build: `cargo build --release`
- Test: `cargo test`
- Run: `cargo run -p sim --release -- <program.bin|.elf> [--trace]`

---

## JIT (Just-In-Time Learning) — mandatory, two files per phase

### Step A — BEFORE writing any code for a phase: `JIT/PhaseX.brief.md`
Write this first, then STOP and wait for me. Do not generate phase code until I say "go".
1. **Goal** — what the phase must do, 3–5 lines. No design, no code.
2. **Design questions (3–5)** — the real decisions this phase forces, phrased as
   questions I answer myself before seeing your design. No hints, no answers.
3. **Where first-timers break things** — 3–5 danger zones (more if genuinely important), named only, not solved.
I write my answers in `JIT/PhaseX.predict.md`. Keep the brief under one screen.

### Step B — AFTER the phase: `JIT/PhaseX.md`
Sections, in this order:
1. **What this phase built** — plain words + diagram, how it fits the project.
2. **Prerequisites** — exactly what the code and commands use, nothing more.
   Concepts already taught: link them via JIT/INDEX.md (see "Concept reuse").
3. **Side-by-side code teaching** — every file, in dependency order, each concept
   taught at the line that uses it. At each real decision point (max ~5 per
   phase), add a decision box:
   > **Decision:** what was chosen
   > **Alternatives:** only ones a competent engineer would actually weigh here
   > **Rejected because:** concrete, tied to this code
   > **Would flip if:** the condition under which the alternative is right
   > **First-timer trap:** the common mistake here + the symptom they'd see. If it
   > happened in git history, cite the commit and show real output. Otherwise
   > label it "common, not hit here". Never invent output or history.
   Mirror each decision box as an entry in DECISIONS.md.
4. **What this code does NOT handle** — gaps, unimplemented edge cases, missing
   checks. Link the retro. Be honest; this is where breadth comes from.
5. **Prediction review** — go through JIT/PhaseX.predict.md question by question:
   matched / wrong / missed entirely. Be blunt. If there are no predictions,
   say so in one line.
6. **Every command run** — shell (Windows PowerShell or WSL, say which), real output,
   failed attempts included.
7. **Verify it yourself** — commands + expected output, including failure cases.
8. **Reconstruct challenges (3–4)** — decision-point code only, not boilerplate.
   Each gets: what to rewrite from a blank file, plus an acceptance check (a
   command + expected output, or a test that must pass). Add one **modify**
   challenge for something not in the code yet.
9. **Transfer** — 2–3 lines: principle → where else it shows up.
10. **Self-check + interview follow-ups** — questions first; every answer inside
    a `<details>` block. Include the 3 "why not X?" follow-ups an interviewer
    is most likely to ask.

Also add one retro bullet per phase: "Predictions I got wrong" (from section 5).

### Concept reuse across JIT phases
- Keep `JIT/INDEX.md`: a lookup table only, never explanations. Columns:
  Concept | Taught in (file + section anchor) | Covered so far | Extended in
- Before writing a phase JIT, read INDEX.md. For a concept already taught, link
  its section and explain only what's new here: a new API, behavior,
  constraint, failure mode, or interaction. Then update "Covered so far" and
  "Extended in".
- A concept counts as taught only if a JIT file explained it. A brief mention or
  an unexplained use doesn't count.
- If a later phase shows an earlier explanation was wrong or incomplete, fix it
  in the original file with a dated note ("Corrected in Phase X: …"). Don't
  rewrite captured output.

### Rules
- Concrete to this code, real output only. When code disagrees with a comment,
  a doc, or this file, say so.
- Prerequisites stay strict. Decision boxes are capped. No general tutorials.
- For a backfilled phase, check out the phase-end commit in a separate worktree
  (`git worktree add ../anarchy-phaseX <commit>`) and capture output from that
  code, not from today's working tree.

---

## Current status
Phase 1, steps 1–2 done 2026-10-04: workspace (`isa`, `sim`), Bus + `Device` trait, RAM as a device, fetch-decode-execute with ADDI/ADD. `tests/asm/add.s` assembled with GCC 13.2 runs to `x3 = 5`. User skipped predictions for Phase 1.
Next: Phase 1 step 3 (full RV32I, then M; unit test every instruction).
