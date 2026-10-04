//! RV32 instruction decoding, shared by the simulator, disassembler and debugger.

/// One decoded instruction. Register fields are indices 0..=31.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instr {
    /// rd = rs1 + imm
    Addi { rd: u8, rs1: u8, imm: i32 },
    /// rd = rs1 + rs2
    Add { rd: u8, rs1: u8, rs2: u8 },
    /// A word we can't decode. Keeps the raw bits for the trap value.
    Illegal(u32),
}

// Major opcodes: bits [6:0] of every instruction.
const OP_IMM: u32 = 0b001_0011;
const OP: u32 = 0b011_0011;

fn rd(w: u32) -> u8 {
    ((w >> 7) & 0x1f) as u8
}

fn rs1(w: u32) -> u8 {
    ((w >> 15) & 0x1f) as u8
}

fn rs2(w: u32) -> u8 {
    ((w >> 20) & 0x1f) as u8
}

fn funct3(w: u32) -> u32 {
    (w >> 12) & 0x7
}

fn funct7(w: u32) -> u32 {
    w >> 25
}

/// I-type immediate: bits [31:20], sign-extended. Casting to i32 first makes `>>` an
/// arithmetic shift, which copies bit 31 into the vacated top bits.
fn imm_i(w: u32) -> i32 {
    (w as i32) >> 20
}

/// Turns a 32-bit instruction word into an `Instr`.
pub fn decode(w: u32) -> Instr {
    match (w & 0x7f, funct3(w), funct7(w)) {
        (OP_IMM, 0b000, _) => Instr::Addi { rd: rd(w), rs1: rs1(w), imm: imm_i(w) },
        (OP, 0b000, 0b000_0000) => Instr::Add { rd: rd(w), rs1: rs1(w), rs2: rs2(w) },
        _ => Instr::Illegal(w),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_addi() {
        // addi x1, x0, 2
        assert_eq!(decode(0x0020_0093), Instr::Addi { rd: 1, rs1: 0, imm: 2 });
    }

    #[test]
    fn addi_immediate_is_sign_extended() {
        // addi x1, x0, -1  (imm field = 0xfff)
        assert_eq!(decode(0xfff0_0093), Instr::Addi { rd: 1, rs1: 0, imm: -1 });
        // addi x5, x6, -2048 (most negative 12-bit immediate)
        assert_eq!(decode(0x8003_0293), Instr::Addi { rd: 5, rs1: 6, imm: -2048 });
    }

    #[test]
    fn decodes_add() {
        // add x3, x1, x2
        assert_eq!(decode(0x0020_81b3), Instr::Add { rd: 3, rs1: 1, rs2: 2 });
    }

    #[test]
    fn sub_is_not_add() {
        // sub x3, x1, x2: same opcode and funct3 as add, funct7 = 0b0100000.
        assert_eq!(decode(0x4020_81b3), Instr::Illegal(0x4020_81b3));
    }

    #[test]
    fn all_zero_word_is_illegal() {
        assert_eq!(decode(0), Instr::Illegal(0));
    }
}
