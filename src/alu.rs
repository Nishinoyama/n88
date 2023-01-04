use crate::BitwiseOps;

pub trait FlagSet<F> {
    fn change(&mut self, flag: F, set: bool);
    fn get_flag(&mut self, flag: F) -> bool;
    fn set(&mut self, flag: F) {
        self.change(flag, true);
    }
    fn reset(&mut self, flag: F) {
        self.change(flag, false);
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct FlagSetBits<B: BitwiseOps>(B);

impl<B: BitwiseOps, F: Into<B>> FlagSet<F> for FlagSetBits<B> {
    fn change(&mut self, flag: F, set: bool) {
        if set {
            self.0 |= flag.into()
        } else {
            self.0 &= !flag.into()
        }
    }
    fn get_flag(&mut self, flag: F) -> bool {
        let b = flag.into();
        (self.0 & b) == b
    }
}

impl<B: BitwiseOps> PartialEq<B> for FlagSetBits<B> {
    fn eq(&self, other: &B) -> bool {
        self.0.eq(other)
    }
}

impl<B: BitwiseOps> From<B> for FlagSetBits<B> {
    fn from(b: B) -> Self {
        Self(b)
    }
}

impl<B: BitwiseOps> FlagSetBits<B> {
    pub fn bits(&self) -> B {
        self.0
    }
}

pub trait ALU {
    type Data;
    type Control;
    type Flag;
    type FlagSet: FlagSet<Self::Flag>;
    fn op(&self, code: Self::Control, a: Self::Data, b: Self::Data) -> (Self::Data, Self::FlagSet);
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Default, Debug, Copy, Clone)]
    struct Adder {}

    #[derive(Debug, Copy, Clone)]
    enum AdderFlag {
        Overflow,
        Signed,
    }

    impl From<AdderFlag> for u8 {
        fn from(flag: AdderFlag) -> Self {
            match flag {
                AdderFlag::Overflow => 1,
                AdderFlag::Signed => 2,
            }
        }
    }

    impl ALU for Adder {
        type Data = u8;
        /// if true, then sub.
        type Control = bool;
        type Flag = AdderFlag;
        type FlagSet = FlagSetBits<u8>;

        fn op(
            &self,
            sub: Self::Control,
            a: Self::Data,
            b: Self::Data,
        ) -> (Self::Data, Self::FlagSet) {
            let (t, overflowed) = if sub {
                a.overflowing_sub(b)
            } else {
                a.overflowing_add(b)
            };
            let mut flag = FlagSetBits::default();
            use AdderFlag::*;
            flag.change(Overflow, overflowed);
            flag.change(Signed, t >= 0x80);
            (t, flag)
        }
    }

    #[test]
    fn test() {
        let adder = Adder::default();
        assert_eq!(adder.op(false, 20, 50), (70, 0.into()));
        assert_eq!(adder.op(false, 120, 50), (170, 2.into()));
        assert_eq!(adder.op(false, 220, 50), (14, 1.into()));
        assert_eq!(adder.op(true, 20, 50), (226, 3.into()));
        assert_eq!(adder.op(true, 120, 50), (70, 0.into()));
        assert_eq!(adder.op(true, 220, 50), (170, 2.into()));
    }
}
