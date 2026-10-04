//! The Anarchy RISC-V simulator.

pub mod bus;
pub mod cpu;
pub mod ram;

use bus::Bus;
use cpu::{Cpu, Trap};
use ram::Ram;

pub const DRAM_BASE: u32 = 0x8000_0000;
pub const DRAM_SIZE: u32 = 32 * 1024 * 1024;

/// Builds the machine with `program` copied to the start of DRAM and pc pointing at it.
pub fn boot(program: &[u8]) -> Result<(Cpu, Bus), String> {
    if program.len() > DRAM_SIZE as usize {
        return Err(format!(
            "program is {} bytes, DRAM is {DRAM_SIZE} bytes",
            program.len()
        ));
    }
    let mut ram = Ram::new(DRAM_SIZE as usize);
    ram.load(0, program);

    let mut bus = Bus::new();
    bus.map(DRAM_BASE, DRAM_SIZE, Box::new(ram));
    Ok((Cpu::new(DRAM_BASE), bus))
}

/// Steps until the CPU traps. Returns the trap and how many instructions completed.
pub fn run(cpu: &mut Cpu, bus: &mut Bus) -> (Trap, u64) {
    let mut retired = 0;
    loop {
        if let Err(trap) = cpu.step(bus) {
            return (trap, retired);
        }
        retired += 1;
    }
}
