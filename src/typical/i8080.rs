use crate::addressing::Addressing;
use crate::alu::typical::*;
use crate::cpu::*;
use crate::memory::typical::*;
use crate::memory::Memory;
use crate::register::typical::*;
use crate::register::{RegisterLoader, RegisterReader, RegisterSet};

#[derive(Debug, Default, Copy, Clone)]
pub struct I8080 {
    data_bus: u8,
    address: u16,
    psw: u16,
    b: u16,
    d: u16,
    h: u16,
    sp: u16,
    pc: u16,
}

impl CPU for I8080 {
    type Data = u8;
    type Address = u16;

    fn data(&self) -> Self::Data {
        self.data_bus
    }

    fn address(&self) -> Self::Address {
        self.address
    }

    fn load_data(mut self, data: Self::Data) -> Self {
        self.data_bus = data;
        self
    }

    fn load_address(mut self, address: Self::Address) -> Self {
        self.address = address;
        self
    }

    fn cycle(self) -> Self {
        todo!()
    }

    fn run(self) -> Option<Self> {
        todo!()
    }
}

impl<M> CPUMemory<M> for I8080 {}

impl RegisterSet<I8080RegisterCode8Bit> for I8080 {
    type Register = u8;
    fn load_of(&mut self, code: I8080RegisterCode8Bit, bits: Self::Register) {
        let low = code.is_low();
        let register = match code {
            I8080RegisterCode8Bit::A => &mut self.psw,
            I8080RegisterCode8Bit::B => &mut self.b,
            I8080RegisterCode8Bit::C => &mut self.b,
            I8080RegisterCode8Bit::D => &mut self.d,
            I8080RegisterCode8Bit::E => &mut self.d,
            I8080RegisterCode8Bit::H => &mut self.h,
            I8080RegisterCode8Bit::L => &mut self.h,
        };
        Register16In8Loader::new(register, low).load(bits)
    }

    fn read_of(&self, code: I8080RegisterCode8Bit) -> Self::Register {
        let low = code.is_low();
        let register = match code {
            I8080RegisterCode8Bit::A => &self.psw,
            I8080RegisterCode8Bit::B => &self.b,
            I8080RegisterCode8Bit::C => &self.b,
            I8080RegisterCode8Bit::D => &self.d,
            I8080RegisterCode8Bit::E => &self.d,
            I8080RegisterCode8Bit::H => &self.h,
            I8080RegisterCode8Bit::L => &self.h,
        };
        Register16In8Reader::new(register, low).read()
    }
}

impl RegisterSet<I8080RegisterCode16Bit> for I8080 {
    type Register = u16;

    fn load_of(&mut self, code: I8080RegisterCode16Bit, bits: Self::Register) {
        Register16Loader::new(match code {
            I8080RegisterCode16Bit::PSW => &mut self.psw,
            I8080RegisterCode16Bit::BC => &mut self.b,
            I8080RegisterCode16Bit::DE => &mut self.d,
            I8080RegisterCode16Bit::HL => &mut self.h,
        })
        .load(bits)
    }

    fn read_of(&self, code: I8080RegisterCode16Bit) -> Self::Register {
        Register16Reader::new(match code {
            I8080RegisterCode16Bit::PSW => &self.psw,
            I8080RegisterCode16Bit::BC => &self.b,
            I8080RegisterCode16Bit::DE => &self.d,
            I8080RegisterCode16Bit::HL => &self.h,
        })
        .read()
    }
}

#[derive(Debug, Copy, Clone)]
enum I8080Addressing8Bit {
    ImmediateValue(u8),
    ImmediateRegister(I8080RegisterCode8Bit),
    DirectValue(u16),
    DirectRegister(I8080RegisterCode16Bit),
}

impl Addressing<I8080> for I8080Addressing8Bit {
    type Size = u8;

    fn value(&self, cpu: &I8080) -> Self::Size {
        match *self {
            I8080Addressing8Bit::ImmediateValue(v) => v,
            I8080Addressing8Bit::ImmediateRegister(reg) => cpu.read_of(reg),
            I8080Addressing8Bit::DirectValue(addr) => cpu.fetch_memory(addr),
            I8080Addressing8Bit::DirectRegister(reg) => {
                I8080Addressing8Bit::DirectValue(cpu.read_of(reg)).value(cpu)
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum I8080Addressing16Bit {
    ImmediateValue(u16),
    ImmediateRegister(I8080RegisterCode16Bit),
}

impl Addressing<I8080> for I8080Addressing16Bit {
    type Size = u16;

    fn value(&self, cpu: &I8080) -> Self::Size {
        match *self {
            I8080Addressing16Bit::ImmediateValue(v) => v,
            I8080Addressing16Bit::ImmediateRegister(reg) => cpu.read_of(reg),
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum I8080RegisterCode8Bit {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

impl I8080RegisterCode8Bit {
    pub(crate) fn is_low(self) -> bool {
        match self {
            I8080RegisterCode8Bit::A => false,
            I8080RegisterCode8Bit::B => false,
            I8080RegisterCode8Bit::D => false,
            I8080RegisterCode8Bit::H => false,
            I8080RegisterCode8Bit::C => true,
            I8080RegisterCode8Bit::E => true,
            I8080RegisterCode8Bit::L => true,
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum I8080RegisterCode16Bit {
    PSW,
    BC,
    DE,
    HL,
}

#[derive(Debug)]
pub struct I8080ALU {
    stats: FlagSetBits<u8>,
}

#[derive(Debug)]
pub enum I8080ALUFlag {
    Sign,
    Zero,
    AuxiliaryCarry,
    Parity,
    Carry,
    // flags.change(Sign, acc >= 0x80);
    // flags.change(Zero, acc == 0x00);
    // flags.change(Parity, acc.count_ones() % 2 == 0);
}

impl From<I8080ALUFlag> for u8 {
    fn from(value: I8080ALUFlag) -> Self {
        match value {
            I8080ALUFlag::Sign => 128,
            I8080ALUFlag::Zero => 64,
            I8080ALUFlag::AuxiliaryCarry => 16,
            I8080ALUFlag::Parity => 4,
            I8080ALUFlag::Carry => 1,
        }
    }
}

#[derive(Debug)]
pub enum I8080ALUControl {
    Add,
    Subtract,
    BitAnd,
    BitOr,
    BitXor,
    Increase,
    Decrease,
    Right,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instruction::typical::*;
    use crate::instruction::Instruction;
    use I8080Addressing8Bit::*;
    use I8080RegisterCode16Bit::*;
    use I8080RegisterCode8Bit::*;

    #[test]
    fn load() {
        use crate::instruction::typical::Load;
        let mut cpu = I8080::default();
        let mut memory = Memory8Bit64KB::default();
        Load::new(A, ImmediateValue(36)).execute(&mut cpu);
        Store::new(
            I8080Addressing16Bit::ImmediateValue(0x1234),
            ImmediateRegister(A),
        )
        .execute(&mut cpu);
        assert_eq!(cpu.read_of(A), 36);
        cpu = cpu.load_address(0x1234);
        assert_eq!(cpu.fetch_memory(&mut memory), 36);
        Load::new(B, ImmediateRegister(A)).execute(&mut cpu);
        Load::new(C, ImmediateRegister(A)).execute(&mut cpu);
        assert_eq!(cpu.read_of(B), 36);
        assert_eq!(cpu.read_of(C), 36);
        assert_eq!(cpu.read_of(BC), 36 * 256 + 36);
        Load::new(HL, I8080Addressing16Bit::ImmediateRegister(BC)).execute(&mut cpu);
        assert_eq!(cpu.read_of(HL), 36 * 256 + 36);
        println!("{:?}", cpu);
    }
}
