use crate::BitwiseOps;

pub trait RegisterCode {}

#[allow(clippy::needless_lifetimes)]
pub trait RegisterSet<C> {
    type Size;
    fn load_of(&mut self, code: C, bits: Self::Size);
    fn read_of(&self, code: C) -> Self::Size;
}

pub trait Register {
    type Size;
    fn load(&mut self, bits: Self::Size);
    fn read(&self) -> Self::Size;
}

pub(crate) trait RegisterLoader: RegisterReader {
    fn load(&mut self, bits: Self::Size);
}

pub(crate) trait RegisterReader {
    type Size;
    fn read(&self) -> Self::Size;
}

#[derive(Debug)]
pub(crate) struct MaskedRegisterLoader<B, L> {
    loader: L,
    mask: B,
}

impl<B: BitwiseOps, L: RegisterLoader<Size = B>> MaskedRegisterLoader<B, L> {
    pub fn new(loader: L, mask: B) -> Self {
        Self { loader, mask }
    }
}

impl<B: BitwiseOps, L: RegisterLoader<Size = B>> RegisterReader for MaskedRegisterLoader<B, L> {
    type Size = B;

    fn read(&self) -> Self::Size {
        self.loader.read() & self.mask
    }
}

impl<B: BitwiseOps, L: RegisterLoader<Size = B>> RegisterLoader for MaskedRegisterLoader<B, L> {
    fn load(&mut self, bits: Self::Size) {
        let res = bits & self.mask | self.loader.read() & !self.mask;
        self.loader.load(res)
    }
}

pub mod typical {
    use super::*;

    #[derive(Debug, Default, Clone)]
    pub struct Register16 {
        bits: u16,
    }

    impl Register for Register16 {
        type Size = u16;

        fn load(&mut self, bits: Self::Size) {
            self.bits = bits;
        }

        fn read(&self) -> Self::Size {
            self.bits
        }
    }

    pub struct Register16In8Loader<'a> {
        register: &'a mut Register16,
        low: bool,
    }

    impl<'a> Register16In8Loader<'a> {
        pub fn new(register: &'a mut Register16, low: bool) -> Self {
            Self { register, low }
        }
    }

    impl<'a> RegisterReader for Register16In8Loader<'a> {
        type Size = u8;

        fn read(&self) -> Self::Size {
            Register16In8Reader::new(self.register, self.low).read()
        }
    }

    impl<'a> RegisterLoader for Register16In8Loader<'a> {
        fn load(&mut self, bits: Self::Size) {
            let t = if self.low {
                self.register.read() & (!0x00ff) | (bits as u16)
            } else {
                self.register.read() & (!0xff00) | ((bits as u16) << 8)
            };
            self.register.load(t)
        }
    }

    #[derive(Debug)]
    pub struct Register16Loader<'a> {
        pub register: &'a mut Register16,
    }

    impl<'a> Register16Loader<'a> {
        pub fn new(register: &'a mut Register16) -> Self {
            Self { register }
        }
    }

    impl<'a> RegisterReader for Register16Loader<'a> {
        type Size = u16;

        fn read(&self) -> Self::Size {
            self.register.read()
        }
    }

    impl<'a> RegisterLoader for Register16Loader<'a> {
        fn load(&mut self, bits: u16) {
            self.register.load(bits)
        }
    }

    pub struct Register16In8Reader<'a> {
        register: &'a Register16,
        low: bool,
    }

    impl<'a> RegisterReader for Register16In8Reader<'a> {
        type Size = u8;
        fn read(&self) -> Self::Size {
            let t = self.register.read();
            if self.low {
                (t & 0x00ff) as u8
            } else {
                (t >> 8) as u8
            }
        }
    }

    impl<'a> Register16In8Reader<'a> {
        pub fn new(register: &'a Register16, low: bool) -> Self {
            Self { register, low }
        }
    }

    #[derive(Debug)]
    pub struct Register16Reader<'a> {
        register: &'a Register16,
    }

    impl<'a> RegisterReader for Register16Reader<'a> {
        type Size = u16;
        fn read(&self) -> Self::Size {
            self.register.read()
        }
    }

    impl<'a> Register16Reader<'a> {
        pub fn new(register: &'a Register16) -> Self {
            Self { register }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::typical::*;
    use super::*;

    #[derive(Default, Debug)]
    struct Register16Set {
        af: Register16,
        hl: Register16,
    }

    enum Register16Code {
        AF,
        HL,
    }

    enum Register8Code {
        A,
        H,
        L,
    }

    impl Register8Code {
        fn is_low(&self) -> bool {
            match self {
                Register8Code::A | Register8Code::H => false,
                Register8Code::L => true,
            }
        }
    }

    #[allow(clippy::needless_lifetimes)]
    impl RegisterSet<Register16Code> for Register16Set {
        type Size = u16;

        fn load_of(&mut self, code: Register16Code, bits: Self::Size) {
            Register16Loader::new(match code {
                Register16Code::AF => &mut self.af,
                Register16Code::HL => &mut self.hl,
            })
            .load(bits)
        }

        fn read_of(&self, code: Register16Code) -> Self::Size {
            Register16Reader::new(match code {
                Register16Code::AF => &self.af,
                Register16Code::HL => &self.hl,
            })
            .read()
        }
    }

    #[allow(clippy::needless_lifetimes)]
    impl RegisterSet<Register8Code> for Register16Set {
        type Size = u8;

        fn load_of(&mut self, code: Register8Code, bits: Self::Size) {
            let register = match code {
                Register8Code::A => &mut self.af,
                Register8Code::H | Register8Code::L => &mut self.hl,
            };
            Register16In8Loader::new(register, code.is_low()).load(bits);
        }

        fn read_of(&self, code: Register8Code) -> Self::Size {
            let register = match code {
                Register8Code::A => &self.af,
                Register8Code::H | Register8Code::L => &self.hl,
            };
            Register16In8Reader::new(register, code.is_low()).read()
        }
    }

    #[test]
    fn register_modifier() {
        let mut reg = Register16::default();
        reg.load(0x1234);
        assert_eq!(reg.read(), 0x1234);
        let mut reg_mod = Register16In8Loader::new(&mut reg, true);
        reg_mod.load(0x56);
        assert_eq!(reg.read(), 0x1256);
        let mut reg_mod = Register16In8Loader::new(&mut reg, false);
        reg_mod.load(0x78);
        assert_eq!(reg.read(), 0x7856);
    }

    #[test]
    fn register_set() {
        use self::Register16Code::*;
        use self::Register8Code::*;
        let mut regs = Register16Set::default();
        regs.load_of(AF, 0x1234);
        assert_eq!(regs.read_of(AF), 0x1234);
        regs.load_of(A, 0x56);
        assert_eq!(regs.read_of(AF), 0x5634);
        regs.load_of(HL, 0x9abc);
        assert_eq!(regs.read_of(H), 0x9a);
        assert_eq!(regs.read_of(L), 0xbc);
    }

    #[test]
    fn bitwise_loader() {
        let mut reg = Register16::default();
        let mut loader = MaskedRegisterLoader::new(Register16Loader::new(&mut reg), 0xf0f0);
        loader.load(0x1234);
        assert_eq!(loader.read(), 0x1030);
    }
}
