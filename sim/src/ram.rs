//! DRAM: a flat byte array. RISC-V is little-endian, so the lowest address holds the lowest byte.

use crate::bus::Device;

pub struct Ram {
    mem: Vec<u8>,
}

impl Ram {
    pub fn new(size: usize) -> Self {
        Self { mem: vec![0; size] }
    }

    /// Copies `data` in at byte offset `off`. Panics if it doesn't fit; callers check first.
    pub fn load(&mut self, off: usize, data: &[u8]) {
        self.mem[off..off + data.len()].copy_from_slice(data);
    }
}

impl Device for Ram {
    fn read(&mut self, off: u32, size: u8) -> u32 {
        let o = off as usize;
        match size {
            1 => self.mem[o] as u32,
            2 => u16::from_le_bytes([self.mem[o], self.mem[o + 1]]) as u32,
            4 => u32::from_le_bytes(self.mem[o..o + 4].try_into().unwrap()),
            _ => unreachable!("bad access size {size}"),
        }
    }

    fn write(&mut self, off: u32, size: u8, val: u32) {
        let o = off as usize;
        let n = size as usize;
        self.mem[o..o + n].copy_from_slice(&val.to_le_bytes()[..n]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn little_endian_round_trip() {
        let mut ram = Ram::new(8);
        ram.write(0, 4, 0x1122_3344);
        assert_eq!(ram.read(0, 1), 0x44);
        assert_eq!(ram.read(2, 2), 0x1122);
        assert_eq!(ram.read(0, 4), 0x1122_3344);
    }

    #[test]
    fn narrow_write_leaves_neighbours_alone() {
        let mut ram = Ram::new(4);
        ram.write(0, 4, 0xffff_ffff);
        ram.write(1, 1, 0x00);
        assert_eq!(ram.read(0, 4), 0xffff_00ff);
    }
}
