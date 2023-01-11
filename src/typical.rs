pub mod i8080 {
    use crate::addressing::Addressing;
    use crate::alu::typical::*;
    use crate::alu::*;
    use crate::cpu::*;
    use crate::memory::typical::*;
    use crate::memory::Memory;
    use crate::register::typical::*;
    use crate::register::{RegisterReader, RegisterSet};

    #[derive(Debug, Default)]
    pub struct I8080 {
        psw: Register16,
        b: Register16,
        d: Register16,
        h: Register16,
        sp: Register16,
        pc: Register16,
        memory: Memory8Bit64KB,
    }

    impl CPUMemory for I8080 {
        type MemoryAddress = u16;
        type MemoryData = u8;

        fn memory_store(&mut self, index: Self::MemoryAddress, data: Self::MemoryData) {
            self.memory.store(index, data);
        }

        fn memory_read(&self, index: Self::MemoryAddress) -> Self::MemoryData {
            self.memory.read(index)
        }
    }

    impl RegisterSet<I8080RegisterCode8Bit> for I8080 {
        type Size = u8;
        type Loader<'a> = Register16In8Loader<'a> where Self: 'a;
        type Reader<'a> = Register16In8Reader<'a> where Self: 'a;

        fn loader_of<'a>(&'a mut self, code: I8080RegisterCode8Bit) -> Self::Loader<'a> {
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
            Register16In8Loader::new(register, low)
        }

        fn reader_of<'a>(&'a self, code: I8080RegisterCode8Bit) -> Self::Reader<'a> {
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
            Register16In8Reader::new(register, low)
        }
    }

    impl RegisterSet<I8080RegisterCode16Bit> for I8080 {
        type Size = u16;
        type Loader<'a> = Register16Loader<'a> where Self: 'a;
        type Reader<'a> = Register16Reader<'a> where Self: 'a;

        fn loader_of<'a>(&'a mut self, code: I8080RegisterCode16Bit) -> Self::Loader<'a> {
            Register16Loader::new(match code {
                I8080RegisterCode16Bit::PSW => &mut self.psw,
                I8080RegisterCode16Bit::BC => &mut self.b,
                I8080RegisterCode16Bit::DE => &mut self.d,
                I8080RegisterCode16Bit::HL => &mut self.h,
            })
        }

        fn reader_of<'a>(&'a self, code: I8080RegisterCode16Bit) -> Self::Reader<'a> {
            Register16Reader::new(match code {
                I8080RegisterCode16Bit::PSW => &self.psw,
                I8080RegisterCode16Bit::BC => &self.b,
                I8080RegisterCode16Bit::DE => &self.d,
                I8080RegisterCode16Bit::HL => &self.h,
            })
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

        fn value(self, cpu: &I8080) -> Self::Size {
            match self {
                I8080Addressing8Bit::ImmediateValue(v) => v,
                I8080Addressing8Bit::ImmediateRegister(reg) => cpu.reader_of(reg).read(),
                I8080Addressing8Bit::DirectValue(addr) => cpu.memory_read(addr),
                I8080Addressing8Bit::DirectRegister(reg) => {
                    I8080Addressing8Bit::DirectValue(cpu.reader_of(reg).read()).value(cpu)
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

        fn value(self, cpu: &I8080) -> Self::Size {
            match self {
                I8080Addressing16Bit::ImmediateValue(v) => v,
                I8080Addressing16Bit::ImmediateRegister(reg) => cpu.reader_of(reg).read(),
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

    impl ALU for I8080ALU {
        type Data = u8;
        type Flag = I8080ALUFlag;
        type FlagSet = FlagSetBits<u8>;
        type Control = I8080ALUControl;

        fn op(
            &self,
            code: Self::Control,
            a: Self::Data,
            b: Self::Data,
        ) -> (Self::Data, Self::FlagSet) {
            todo!()
        }
    }

    #[derive(Debug)]
    pub enum I8080ALUFlag {
        Sign,
        Zero,
        AuxiliaryCarry,
        Parity,
        Carry,
        // flagset.change(Sign, acc >= 0x80);
        // flagset.change(Zero, acc == 0x00);
        // flagset.change(Parity, acc.count_ones() % 2 == 0);
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
            Load::new(A, ImmediateValue(36)).execute(&mut cpu);
            Store::new(
                I8080Addressing16Bit::ImmediateValue(0x1234),
                ImmediateRegister(A),
            )
            .execute(&mut cpu);
            assert_eq!(cpu.reader_of(A).read(), 36);
            assert_eq!(cpu.memory_read(0x1234), 36);
            Load::new(B, ImmediateRegister(A)).execute(&mut cpu);
            Load::new(C, ImmediateRegister(A)).execute(&mut cpu);
            assert_eq!(cpu.reader_of(B).read(), 36);
            assert_eq!(cpu.reader_of(C).read(), 36);
            assert_eq!(cpu.reader_of(BC).read(), 36 * 256 + 36);
            Load::new(HL, I8080Addressing16Bit::ImmediateRegister(BC)).execute(&mut cpu);
            assert_eq!(cpu.reader_of(HL).read(), 36 * 256 + 36);
            println!("{:?}", cpu);
        }
    }
}
