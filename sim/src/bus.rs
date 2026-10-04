//! The bus routes a physical address to the device mapped there.

/// Anything that sits on the bus: RAM, UART, CLINT, framebuffer, keyboard.
/// `off` is relative to the device's base address; `size` is 1, 2 or 4 bytes.
pub trait Device {
    fn read(&mut self, off: u32, size: u8) -> u32;
    fn write(&mut self, off: u32, size: u8, val: u32);
    fn tick(&mut self) {}
    fn irq(&self) -> bool {
        false
    }
}

/// An access that hit no device, or ran off the end of one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BusFault {
    pub addr: u32,
}

struct Region {
    base: u32,
    len: u32,
    dev: Box<dyn Device>,
}

#[derive(Default)]
pub struct Bus {
    regions: Vec<Region>,
}

impl Bus {
    pub fn new() -> Self {
        Self::default()
    }

    /// Maps `dev` at `[base, base + len)`. Panics on overlap: that's a wiring bug, not a guest error.
    pub fn map(&mut self, base: u32, len: u32, dev: Box<dyn Device>) {
        let end = base as u64 + len as u64;
        for r in &self.regions {
            let r_end = r.base as u64 + r.len as u64;
            assert!(
                end <= r.base as u64 || base as u64 >= r_end,
                "device at {base:#010x} overlaps device at {:#010x}",
                r.base
            );
        }
        self.regions.push(Region { base, len, dev });
    }

    pub fn read(&mut self, addr: u32, size: u8) -> Result<u32, BusFault> {
        let (r, off) = self.find(addr, size)?;
        Ok(r.dev.read(off, size))
    }

    pub fn write(&mut self, addr: u32, size: u8, val: u32) -> Result<(), BusFault> {
        let (r, off) = self.find(addr, size)?;
        r.dev.write(off, size, val);
        Ok(())
    }

    /// Finds the region holding all `size` bytes starting at `addr`.
    fn find(&mut self, addr: u32, size: u8) -> Result<(&mut Region, u32), BusFault> {
        for r in &mut self.regions {
            // wrapping_sub: if addr < base this becomes huge and fails the check below.
            let off = addr.wrapping_sub(r.base);
            if off < r.len && size as u32 <= r.len - off {
                return Ok((r, off));
            }
        }
        Err(BusFault { addr })
    }
}
