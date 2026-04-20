/// LC-3 word-addressed memory: 65 536 × 16-bit words.
///
/// Memory-mapped I/O registers at the top of the address space are handled
/// transparently on read; writes go directly to the backing array.
pub struct Memory {
    words: Box<[u16; 65536]>,
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

impl Memory {
    pub fn new() -> Self {
        let mut mem = Self {
            words: Box::new([0u16; 65536]),
        };
        // MCR (Machine Control Register): bit 15 = clock enable; start enabled.
        mem.words[0xFFFE] = 0x8000;
        mem
    }

    /// Read a word, returning synthetic values for MMIO status registers.
    pub fn read(&self, addr: u16) -> u16 {
        match addr {
            0xFE00 => 0x8000, // KBSR: keyboard always ready
            0xFE04 => 0x8000, // DSR:  display always ready
            _ => self.words[addr as usize],
        }
    }

    /// Write a word directly to the backing array (including MMIO space).
    pub fn write(&mut self, addr: u16, val: u16) {
        self.words[addr as usize] = val;
    }

    /// Read bypassing MMIO substitution — used by the disassembler / listing.
    #[inline]
    pub fn raw(&self, addr: u16) -> u16 {
        self.words[addr as usize]
    }

    /// Load a program image starting at `orig`.
    pub fn load(&mut self, orig: u16, words: &[u16]) {
        let start = orig as usize;
        let end = (start + words.len()).min(65536);
        self.words[start..end].copy_from_slice(&words[..end - start]);
    }

    /// Clock-enable bit of the Machine Control Register.
    #[inline]
    pub fn clock_enabled(&self) -> bool {
        self.words[0xFFFE] & 0x8000 != 0
    }
}
