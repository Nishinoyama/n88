use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};

pub trait BitwiseOps:
    BitAnd<Output = Self>
    + BitOr<Output = Self>
    + BitAndAssign
    + BitOrAssign
    + Not<Output = Self>
    + Eq
    + PartialEq
    + Copy
{
    const ALL_ONE: Self;
    const ALL_ZERO: Self;
}

macro_rules! bitwise_ops_impl {
    ($($t:ty)*) => {$(
        impl BitwiseOps for $t {
            const ALL_ONE: Self = <$t>::MAX;
            const ALL_ZERO: Self = <$t>::MIN;
        }
    )*}
}

bitwise_ops_impl!(u8 u16 u32 u64 usize);

pub mod register;

// pub mod instruction;

pub mod alu;

pub mod memory;

pub mod cpu;

pub mod addressing;

pub mod typical;
