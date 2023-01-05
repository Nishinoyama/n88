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
    const ALL_ON: Self;
    const ALL_OFF: Self;
}
impl BitwiseOps for u8 {
    const ALL_ON: Self = u8::MAX;
    const ALL_OFF: Self = u8::MIN;
}
impl BitwiseOps for u16 {
    const ALL_ON: Self = u16::MAX;
    const ALL_OFF: Self = u16::MIN;
}
impl BitwiseOps for u32 {
    const ALL_ON: Self = u32::MAX;
    const ALL_OFF: Self = u32::MIN;
}
impl BitwiseOps for u64 {
    const ALL_ON: Self = u64::MAX;
    const ALL_OFF: Self = u64::MIN;
}
impl BitwiseOps for usize {
    const ALL_ON: Self = usize::MAX;
    const ALL_OFF: Self = usize::MIN;
}

pub mod register;

pub mod instruction;

pub mod alu;

pub mod memory;

pub mod cpu;
