pub trait Memory {
    type Address;
    type Data;
    fn read(&self, index: Self::Address) -> Self::Data;
    fn store(&mut self, index: Self::Address, data: Self::Data);
}

#[derive(Debug)]
pub struct Memory8Bit64KB {
    bytes: [u8; 65536],
}

impl Memory8Bit64KB {
    fn new(bytes: &[u8]) -> Self {
        let mut mem = Self::default();
        for (i, &x) in bytes.iter().enumerate() {
            mem.store(i as u16, x);
        }
        mem
    }
}

impl Default for Memory8Bit64KB {
    fn default() -> Self {
        Memory8Bit64KB {
            bytes: [0u8; 65536],
        }
    }
}

impl Memory for Memory8Bit64KB {
    type Address = u16;
    type Data = u8;
    fn read(&self, index: u16) -> u8 {
        self.bytes[index as usize]
    }
    fn store(&mut self, index: u16, data: u8) {
        self.bytes[index as usize] = data
    }
}
