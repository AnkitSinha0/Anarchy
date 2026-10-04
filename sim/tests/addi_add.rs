use sim::cpu::Trap;

fn words_to_bytes(words: &[u32]) -> Vec<u8> {
    words.iter().flat_map(|w| w.to_le_bytes()).collect()
}

#[test]
fn three_line_program_sets_x3_to_5() {
    let program = words_to_bytes(&[
        0x0020_0093, // addi x1, x0, 2
        0x0030_0113, // addi x2, x0, 3
        0x0020_81b3, // add  x3, x1, x2
    ]);
    let (mut cpu, mut bus) = sim::boot(&program).unwrap();

    let (trap, retired) = sim::run(&mut cpu, &mut bus);

    assert_eq!(cpu.reg(3), 5);
    assert_eq!(retired, 3);
    // Zeroed RAM after the program decodes as an illegal instruction, which ends the run.
    assert_eq!(trap, Trap::IllegalInstruction { pc: sim::DRAM_BASE + 12, word: 0 });
}

#[test]
fn writes_to_x0_are_discarded() {
    let program = words_to_bytes(&[0x0050_0013]); // addi x0, x0, 5
    let (mut cpu, mut bus) = sim::boot(&program).unwrap();

    sim::run(&mut cpu, &mut bus);

    assert_eq!(cpu.reg(0), 0);
}

#[test]
fn add_wraps_on_overflow() {
    let program = words_to_bytes(&[
        0xfff0_0093, // addi x1, x0, -1   (0xffff_ffff)
        0x0010_0113, // addi x2, x0, 1
        0x0020_81b3, // add  x3, x1, x2
    ]);
    let (mut cpu, mut bus) = sim::boot(&program).unwrap();

    sim::run(&mut cpu, &mut bus);

    assert_eq!(cpu.reg(3), 0);
}

#[test]
fn running_off_the_end_of_dram_is_an_access_fault() {
    let (mut cpu, mut bus) = sim::boot(&[]).unwrap();
    cpu.pc = sim::DRAM_BASE + sim::DRAM_SIZE;

    let (trap, _) = sim::run(&mut cpu, &mut bus);

    assert_eq!(trap, Trap::InstructionAccessFault { pc: sim::DRAM_BASE + sim::DRAM_SIZE });
}
