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
}
impl BitwiseOps for u8 {}
impl BitwiseOps for u16 {}
impl BitwiseOps for u32 {}
impl BitwiseOps for u64 {}
impl BitwiseOps for usize {}

pub mod register;

pub mod instruction;

pub mod alu;
