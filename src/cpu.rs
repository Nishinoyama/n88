use crate::alu::{FlagSet, ALU};
use crate::memory::Memory;
use crate::register::{RegisterLoader, RegisterReader};

pub enum CPURunningState {
    Running,
    Halted,
    Error,
}

pub trait CPU {
    fn cycle(&mut self) -> CPURunningState;
    fn run(&mut self) {
        while let CPURunningState::Running = self.cycle() {}
    }
}

pub trait CPUMemory {
    type Memory: Memory;
    fn mut_memory(&mut self) -> &mut Self::Memory;
    fn memory(&self) -> &Self::Memory;
}

pub trait CPUAlu {
    type ALU: ALU;
    fn alu(&self) -> &Self::ALU;
    fn alu_acc_op<'a, R, L>(
        &'a mut self,
        control: <Self::ALU as ALU>::Control,
        lhs: <Self::ALU as ALU>::Data,
    ) -> (<Self::ALU as ALU>::Data, <Self::ALU as ALU>::FlagSet)
    where
        Self: 'a + CPUAccumulator<AccReader<'a> = R, AccLoader<'a> = L>,
        R: RegisterReader<Size = <Self::ALU as ALU>::Data>,
    {
        let rhs = self.acc_reader().read();
        let res = self.alu().op(control, rhs, lhs);
        res
    }
}

pub trait CPUAccumulator {
    type AccSize;
    type AccLoader<'a>: RegisterLoader<Size = Self::AccSize>
    where
        Self: 'a;
    type AccReader<'a>: RegisterReader<Size = Self::AccSize>
    where
        Self: 'a;
    fn acc_loader<'a>(&'a mut self) -> Self::AccLoader<'a>;
    fn acc_reader<'a>(&'a self) -> Self::AccReader<'a>;
}

pub trait CPUFlagRegister: CPUAlu {
    type FlagRegisterSize: Into<<Self::ALU as ALU>::FlagSet>;
    type FlagRegisterLoader<'a>: RegisterLoader<Size = Self::FlagRegisterSize>
    where
        Self: 'a;
    type FlagRegisterReader<'a>: RegisterReader<Size = Self::FlagRegisterSize>
    where
        Self: 'a;
    fn flag_loader<'a>(&'a mut self) -> Self::FlagRegisterLoader<'a> {
        self.flag_loader_masked(<Self::ALU as ALU>::FlagSet::all_on())
    }
    fn flag_loader_masked<'a>(
        &'a mut self,
        flag_mask: <Self::ALU as ALU>::FlagSet,
    ) -> Self::FlagRegisterLoader<'a>;
    fn flag_loader_mask_slice<'a>(
        &'a mut self,
        flag_masks: &[<Self::ALU as ALU>::FlagSet],
    ) -> Self::FlagRegisterLoader<'a> {
        self.flag_loader_masked(<Self::ALU as ALU>::FlagSet::all_on())
    }
    fn flag_reader<'a>(&'a self) -> Self::FlagRegisterReader<'a>;
    fn flag_on(&mut self, flag: <Self::ALU as ALU>::Flag) -> bool {
        let flags: <Self::ALU as ALU>::FlagSet = self.flag_reader().read().into();
        flags.get_flag(flag)
    }
}

// pub trait CPUProgramCounter: CPUMemory {
//     type ProgramCounterLoader<'a>: RegisterLoader<Size = <Self::Memory as Memory>::Address>
//     where
//         Self: 'a;
//     type ProgramCounterReader<'a>: RegisterReader<Size = <Self::Memory as Memory>::Address>
//     where
//         Self: 'a;
//     fn program_counter_loader<'a>(&mut self) -> Self::ProgramCounterLoader<'a>;
//     fn program_counter_reader<'a>(&self) -> Self::ProgramCounterReader<'a>;
//     fn program_fetch(&mut self) -> <Self::Memory as Memory>::Data {
//         let index = self.program_counter_reader().read();
//         self.memory().read(index)
//     }
// }
//
// pub trait CPUStackPointer: CPUMemory {
//     type StackNodeSize;
//     type StackPointerLoader<'a>: RegisterLoader<Size = <Self::Memory as Memory>::Address>
//     where
//         Self: 'a;
//     type StackPointerReader<'a>: RegisterReader<Size = <Self::Memory as Memory>::Address>
//     where
//         Self: 'a;
//     fn stack_pointer_loader<'a>(&mut self) -> Self::StackPointerLoader<'a>;
//     fn stack_pointer_reader<'a>(&self) -> Self::StackPointerReader<'a>;
//     fn push(&mut self, bits: Self::StackNodeSize);
//     fn pop(&mut self) -> Self::StackNodeSize;
// }
//
// pub trait CPUJump: CPUMemory + CPUProgramCounter {
//     fn jump(&mut self, index: <Self::Memory as Memory>::Address) {
//         self.program_counter_loader().load(index);
//     }
//     fn jump_on<'a>(
//         &'a mut self,
//         index: <Self::Memory as Memory>::Address,
//         flag: <Self::ALU as ALU>::Flag,
//     ) where
//         Self: CPUFlagRegister,
//         <Self::ALU as ALU>::FlagSet: From<<Self::FlagRegisterReader<'a> as RegisterReader>::Size>,
//     {
//         if self.flag_on(flag) {
//             self.jump(index)
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alu::typical::FlagSetBits;
    use crate::alu::FlagSet;
    use crate::instruction::Instruction;
    use crate::memory::typical::Memory8Bit64KB;
    use crate::register::typical::*;
    use crate::register::*;

    #[derive(Default, Debug)]
    struct CPU8 {
        af: Register16,
        bc: Register16,
        hl: Register16,
        sp: Register16,
        memory: Memory8Bit64KB,
    }

    impl CPUMemory for CPU8 {
        type Memory = Memory8Bit64KB;
        fn mut_memory(&mut self) -> &mut Self::Memory {
            &mut self.memory
        }
        fn memory(&self) -> &Self::Memory {
            &self.memory
        }
    }

    impl CPUAccumulator for CPU8 {
        type AccSize = u8;
        type AccLoader<'a> = Register16In8Loader<'a> where Self: 'a;
        type AccReader<'a> = Register16In8Reader<'a> where Self: 'a;

        fn acc_loader<'a>(&'a mut self) -> Self::AccLoader<'a> {
            Register16In8Loader::new(&mut self.af, false)
        }

        fn acc_reader<'a>(&'a self) -> Self::AccReader<'a> {
            Register16In8Reader::new(&self.af, false)
        }
    }

    impl CPUFlagRegister for CPU8 {
        type FlagRegisterSize = u8;
        type FlagRegisterLoader<'a> = MaskedRegisterLoader<Self::FlagRegisterSize, Register16In8Loader<'a>> where Self: 'a;
        type FlagRegisterReader<'a> = Register16In8Reader<'a> where Self: 'a;

        fn flag_loader_masked<'a>(
            &'a mut self,
            flag_mask: <Self::ALU as ALU>::FlagSet,
        ) -> Self::FlagRegisterLoader<'a> {
            MaskedRegisterLoader::new(
                Register16In8Loader::new(&mut self.af, true),
                flag_mask.into(),
            )
        }

        fn flag_reader<'a>(&'a self) -> Self::FlagRegisterReader<'a> {
            Register16In8Reader::new(&self.af, true)
        }
    }

    impl CPUAlu for CPU8 {
        type ALU = Adder;

        fn alu(&self) -> &Self::ALU {
            &Adder {}
        }
    }

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

    #[derive(Copy, Clone)]
    enum Register16Code {
        AF,
        BC,
        HL,
    }

    #[derive(Copy, Clone)]
    enum Register8Code {
        A,
        B,
        C,
        H,
        L,
    }

    #[derive(Copy, Clone)]
    enum Inst {
        Load(Register8Code, u8),
        Mov(Register8Code, Register8Code),
        LoadX(Register16Code, u16),
        MovX(Register16Code, Register16Code),
        Store,
        StoreX,
        Add(Register8Code),
        Sub(Register8Code),
        Nop,
    }

    impl Instruction<CPU8> for Inst {
        fn execute(&self, cpu: &mut CPU8) {
            use Register16Code::*;
            match self {
                Inst::Load(dst, t) => cpu.loader_of(*dst).load(*t),
                Inst::Mov(dst, src) => {
                    let bits = cpu.reader_of(*src).read();
                    cpu.loader_of(*dst).load(bits);
                }
                Inst::LoadX(dst, x) => cpu.loader_of(*dst).load(*x),
                Inst::MovX(dst, src) => {
                    let bits = cpu.reader_of(*src).read();
                    cpu.loader_of(*dst).load(bits);
                }
                Inst::Store => {
                    let bits = cpu.acc_reader().read();
                    let index = cpu.reader_of(HL).read();
                    cpu.mut_memory().store(index, bits);
                }
                Inst::StoreX => {
                    let [b, c] = cpu.reader_of(BC).read().to_le_bytes();
                    let index = cpu.reader_of(HL).read();
                    cpu.mut_memory().store(index, b);
                    cpu.mut_memory().store(index + 1, c);
                }
                Inst::Add(lhs) => {
                    let lhs = cpu.reader_of(*lhs).read();
                    let (res, flag) = cpu.alu_acc_op(false, lhs);
                    cpu.flag_loader().load(flag.into());
                    cpu.acc_loader().load(res);
                }
                Inst::Sub(lhs) => {
                    let lhs = cpu.reader_of(*lhs).read();
                    let (res, flag) = cpu.alu_acc_op(false, lhs);
                    cpu.flag_loader().load(flag.into());
                    cpu.acc_loader().load(res);
                }
                Inst::Nop => {}
            }
        }
    }

    #[allow(clippy::needless_lifetimes)]
    impl RegisterSet<Register16Code> for CPU8 {
        type Size = u16;
        type Loader<'a> = Register16Loader<'a>;
        type Reader<'a> = Register16Reader<'a>;

        fn loader_of<'a>(&'a mut self, code: Register16Code) -> Self::Loader<'a> {
            Register16Loader::new(match code {
                Register16Code::AF => &mut self.af,
                Register16Code::HL => &mut self.hl,
                Register16Code::BC => &mut self.bc,
            })
        }
        fn reader_of<'a>(&'a self, code: Register16Code) -> Self::Reader<'a> {
            Register16Reader::new(match code {
                Register16Code::AF => &self.af,
                Register16Code::HL => &self.hl,
                Register16Code::BC => &self.bc,
            })
        }
    }

    #[allow(clippy::needless_lifetimes)]
    impl RegisterSet<Register8Code> for CPU8 {
        type Size = u8;
        type Loader<'a> = Register16In8Loader<'a>;
        type Reader<'a> = Register16In8Reader<'a>;

        fn loader_of<'a>(&'a mut self, code: Register8Code) -> Self::Loader<'a> {
            let register = match code {
                Register8Code::A => &mut self.af,
                Register8Code::H | Register8Code::L => &mut self.hl,
                Register8Code::B | Register8Code::C => &mut self.bc,
            };
            Register16In8Loader::new(register, code.is_low())
        }

        fn reader_of<'a>(&'a self, code: Register8Code) -> Self::Reader<'a> {
            let register = match code {
                Register8Code::A => &self.af,
                Register8Code::H | Register8Code::L => &self.hl,
                Register8Code::B | Register8Code::C => &self.bc,
            };
            Register16In8Reader::new(register, code.is_low())
        }
    }

    impl Register8Code {
        fn is_low(&self) -> bool {
            use Register8Code::*;
            match self {
                A | H | B => false,
                L | C => true,
            }
        }
    }

    #[test]
    fn instruction() {
        use Inst::*;
        use Register16Code::*;
        use Register8Code::*;
        let mut cpu = CPU8::default();
        let m = &mut cpu;
        let instructions = [
            Load(A, 10),
            Load(B, 32),
            Add(B),
            LoadX(HL, 0x3141),
            Store,
            Load(C, 94),
            Mov(B, A),
            LoadX(HL, 0x1729),
            StoreX,
        ];
        instructions.into_iter().for_each(|i| i.execute(m));
        assert_eq!(cpu.memory().read(0x3141), 42);
        assert_eq!(cpu.memory().read(0x1729), 94);
        assert_eq!(cpu.memory().read(0x172a), 42);
    }
}
