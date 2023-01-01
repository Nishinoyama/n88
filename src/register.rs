pub trait RegisterCode {}

pub trait RegisterSet<R: RegisterCode> {
    type Size;
    fn read_of(&self, code: R) -> Self::Size;
    fn load_of(&mut self, code: R, bits: Self::Size);
}

pub trait RegisterFlag {
    type Flag;
    fn flag_get(&self, flag: Self::Flag) -> bool;
    fn flag_change(&mut self, flag: Self::Flag, set: bool);
    fn flag_set(&mut self, flag: Self::Flag) {
        self.flag_change(flag, true);
    }
    fn flag_reset(&mut self, flag: Self::Flag) {
        self.flag_change(flag, false);
    }
    fn flag_change_in(&mut self, flag_statuses: Vec<(Self::Flag, bool)>) {
        for (flag, set) in flag_statuses {
            self.flag_change(flag, set)
        }
    }
}

pub trait Register {
    type Size;
    fn load(&mut self, bits: Self::Size);
    fn read(&self) -> Self::Size;
}

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

pub(crate) struct Register16In8Writer<'a> {
    pub register: &'a mut Register16,
    pub low: bool,
}

impl<'a> Register16In8Writer<'a> {
    pub(crate) fn write(&mut self, bits: u8) {
        let t = if self.low {
            self.register.read() & (!0x00ff) | (bits as u16)
        } else {
            self.register.read() & (!0xff00) | ((bits as u16) << 8)
        };
        self.register.load(t)
    }
}

pub(crate) struct Register16In8Loader<'a> {
    pub register: &'a Register16,
    pub low: bool,
}

impl<'a> Register16In8Loader<'a> {
    pub(crate) fn load(&self) -> u8 {
        let t = self.register.read();
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

    impl RegisterCode for Register16Code {}

    impl RegisterCode for Register8Code {}

    impl RegisterSet<Register16Code> for Register16Set {
        type Size = u16;

        fn read_of(&self, code: Register16Code) -> Self::Size {
            match code {
                Register16Code::AF => self.af.read(),
                Register16Code::HL => self.hl.read(),
            }
        }

        fn load_of(&mut self, code: Register16Code, bits: Self::Size) {
            match code {
                Register16Code::AF => self.af.load(bits),
                Register16Code::HL => self.hl.load(bits),
            }
        }
    }

    impl RegisterSet<Register8Code> for Register16Set {
        type Size = u8;

        fn read_of(&self, code: Register8Code) -> Self::Size {
            let register = match code {
                Register8Code::A => &self.af,
                Register8Code::H | Register8Code::L => &self.hl,
            };
            Register16In8Loader {
                low: code.is_low(),
                register,
            }.load()
        }

        fn load_of(&mut self, code: Register8Code, bits: Self::Size) {
            let register = match code {
                Register8Code::A => &mut self.af,
                Register8Code::H | Register8Code::L => &mut self.hl,
            };
            Register16In8Writer {
                low: code.is_low(),
                register,
            }.write(bits)
        }
    }

    enum Flag {
        Overflow = 1,
        Neg = 2,
    }

    pub(crate) struct Register16FlagWriter<'a> {
        register: &'a mut Register16,
        flag_bit: u8,
    }

    impl<'a> Register16FlagWriter<'a> {
        fn write(&mut self, set: bool) {
            let t = Register16In8Loader { register: self.register, low: true }.load();
            if set {
                Register16In8Writer { register: self.register, low: true }.write(t | self.flag_bit);
            } else {
                Register16In8Writer { register: self.register, low: true }.write(t & !self.flag_bit);
            }
        }
    }

    impl RegisterFlag for Register16Set {
        type Flag = Flag;

        fn flag_get(&self, flag: Self::Flag) -> bool {
            let flag_bit: u8 = unsafe {
                std::mem::transmute(flag)
            };
            (Register16In8Loader {
                register: &self.af,
                low: true,
            }.load() & flag_bit) == flag_bit
        }

        fn flag_change(&mut self, flag: Self::Flag, set: bool) {
            let flag_bit = unsafe {
                std::mem::transmute(flag)
            };
            Register16FlagWriter {
                register: &mut self.af,
                flag_bit,
            }.write(set)
        }
    }

    #[test]
    fn register_modifier() {
        let mut reg = Register16::default();
        reg.load(0x1234);
        assert_eq!(reg.read(), 0x1234);
        let mut reg_mod = Register16In8Writer {
            register: &mut reg,
            low: true,
        };
        reg_mod.write(0x56);
        assert_eq!(reg.read(), 0x1256);
        let mut reg_mod = Register16In8Writer {
            register: &mut reg,
            low: false,
        };
        reg_mod.write(0x78);
        assert_eq!(reg.read(), 0x7856);
    }

    #[test]
    fn register_set() {
        use self::Register8Code::*;
        use self::Register16Code::*;
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
    fn register_flag() {
        use self::Register16Code::*;
        use self::Flag::*;
        let mut regs = Register16Set::default();
        regs.load_of(AF, 0x0000);
        regs.flag_set(Overflow);
        assert!(regs.flag_get(Overflow));
        regs.flag_set(Neg);
        assert!(regs.flag_get(Neg));
        regs.flag_change_in(vec![
            (Overflow, false),
            (Neg, true),
        ]);
        assert!(!regs.flag_get(Overflow));
        assert!(regs.flag_get(Neg));
    }
}
