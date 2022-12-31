pub trait RegisterCode {
    type Register: Register;
}

pub trait RegisterSet<R: RegisterCode> {
    type Size;
    fn load_of(&self, code: R) -> Self::Size;
    fn write_of(&mut self, code: R, bits: Self::Size);
}

pub trait Register {
    type Size;
    fn write(&mut self, bits: Self::Size);
    fn load(&self) -> Self::Size;
}

#[derive(Debug, Default, Clone)]
pub struct Register16 {
    bits: u16,
}

impl Register for Register16 {
    type Size = u16;

    fn write(&mut self, bits: Self::Size) {
        self.bits = bits;
    }

    fn load(&self) -> Self::Size {
        self.bits
    }
}

pub(crate) struct Register16In8Writer<'a> {
    register: &'a mut Register16,
    low: bool,
}

impl<'a> Register16In8Writer<'a> {
    fn write(&mut self, bits: u8) {
        let t = if self.low {
            self.register.load() & (!0x00ff) | (bits as u16)
        } else {
            self.register.load() & (!0xff00) | ((bits as u16) << 8)
        };
        self.register.write(t)
    }
}

pub(crate) struct Register16In8Loader<'a> {
    register: &'a Register16,
    low: bool,
}

impl<'a> Register16In8Loader<'a> {
    fn load(&self) -> u8 {
        let t = self.register.load();
        if self.low {
            (t & 0x00ff) as u8
        } else {
            (t >> 8) as u8
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default, Debug)]
    struct Register16s {
        af: Register16,
        hl: Register16,
    }

    enum Register16Code {
        AF,
        HL,
    }

    enum Register8Code {
        A,
        F,
        H,
        L,
    }

    impl Register8Code {
        fn is_low(&self) -> bool {
            match self {
                Register8Code::A | Register8Code::H => false,
                Register8Code::F | Register8Code::L => true,
            }
        }
    }

    impl RegisterCode for Register16Code {
        type Register = Register16;
    }

    impl RegisterCode for Register8Code {
        type Register = Register16;
    }

    impl RegisterSet<Register16Code> for Register16s {
        type Size = u16;

        fn load_of(&self, code: Register16Code) -> Self::Size {
            match code {
                Register16Code::AF => self.af.load(),
                Register16Code::HL => self.hl.load(),
            }
        }

        fn write_of(&mut self, code: Register16Code, bits: Self::Size) {
            match code {
                Register16Code::AF => self.af.write(bits),
                Register16Code::HL => self.hl.write(bits),
            }
        }
    }

    impl RegisterSet<Register8Code> for Register16s {
        type Size = u8;

        fn load_of(&self, code: Register8Code) -> Self::Size {
            let register = match code {
                Register8Code::A | Register8Code::F => &self.af,
                Register8Code::H | Register8Code::L => &self.hl,
            };
            Register16In8Loader {
                low: code.is_low(),
                register,
            }.load()
        }

        fn write_of(&mut self, code: Register8Code, bits: Self::Size) {
            let register = match code {
                Register8Code::A | Register8Code::F => &mut self.af,
                Register8Code::H | Register8Code::L => &mut self.hl,
            };
            Register16In8Writer {
                low: code.is_low(),
                register,
            }.write(bits)
        }
    }

    #[test]
    fn register_modifier() {
        let mut reg = Register16::default();
        reg.write(0x1234);
        assert_eq!(reg.load(), 0x1234);
        let mut reg_mod = Register16In8Writer {
            register: &mut reg,
            low: true,
        };
        reg_mod.write(0x56);
        assert_eq!(reg.load(), 0x1256);
        let mut reg_mod = Register16In8Writer {
            register: &mut reg,
            low: false,
        };
        reg_mod.write(0x78);
        assert_eq!(reg.load(), 0x7856);
    }

    #[test]
    fn register_set() {
        use self::Register8Code::*;
        use self::Register16Code::*;
        let mut regs = Register16s::default();
        regs.write_of(AF, 0x1234);
        assert_eq!(regs.load_of(AF), 0x1234);
        regs.write_of(A, 0x56);
        assert_eq!(regs.load_of(AF), 0x5634);
        regs.write_of(F, 0x78);
        assert_eq!(regs.load_of(AF), 0x5678);
        regs.write_of(HL, 0x9abc);
        assert_eq!(regs.load_of(H), 0x9a);
        assert_eq!(regs.load_of(L), 0xbc);
    }
}
