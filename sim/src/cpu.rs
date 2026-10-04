//! The hart: 32 registers, a program counter, and one fetch-decode-execute step.

use isa::{Instr, decode};

use crate::bus::Bus;

/// Why the CPU stopped. Phase 1 has no trap handler yet, so every trap ends the run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trap {
    /// Fetch hit an address with no device behind it.
    InstructionAccessFault { pc: u32 },
    /// The word at `pc` isn't an instruction we implement.
    IllegalInstruction { pc: u32, word: u32 },
}

pub struct Cpu {
    regs: [u32; 32],
    pub pc: u32,
}

impl Cpu {
    pub fn new(pc: u32) -> Self {
        Self { regs: [0; 32], pc }
    }

    pub fn reg(&self, i: u8) -> u32 {
        self.regs[i as usize]
    }

    pub fn regs(&self) -> &[u32; 32] {
        &self.regs
    }

    /// x0 is hardwired to zero: writes to it are discarded.
    fn set_reg(&mut self, i: u8, val: u32) {
        if i != 0 {
            self.regs[i as usize] = val;
        }
    }

    /// Executes one instruction. On a trap, pc and registers are left as they were before it.
    pub fn step(&mut self, bus: &mut Bus) -> Result<(), Trap> {
        let pc = self.pc;
        let word = bus
            .read(pc, 4)
            .map_err(|_| Trap::InstructionAccessFault { pc })?;
        let next_pc = pc.wrapping_add(4);

        // wrapping_add: the CPU wraps on overflow; Rust's plain `+` would panic in debug builds.
        match decode(word) {
            Instr::Addi { rd, rs1, imm } => {
                self.set_reg(rd, self.reg(rs1).wrapping_add(imm as u32));
            }
            Instr::Add { rd, rs1, rs2 } => {
                self.set_reg(rd, self.reg(rs1).wrapping_add(self.reg(rs2)));
            }
            Instr::Illegal(word) => return Err(Trap::IllegalInstruction { pc, word }),
        }

        self.pc = next_pc;
        Ok(())
    }
}
