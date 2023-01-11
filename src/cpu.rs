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
    type MemoryAddress;
    type MemoryData;
    fn memory_store(&mut self, index: Self::MemoryAddress, data: Self::MemoryData);
    fn memory_read(&self, index: Self::MemoryAddress) -> Self::MemoryData;
}

pub trait CPUAlu {
    type ALU: ALU;
    fn alu(&self) -> Self::ALU;
    fn alu_acc_op<'a>(
        &'a mut self,
        control: <Self::ALU as ALU>::Control,
        rhs: <Self::ALU as ALU>::Data,
    ) -> (<Self::ALU as ALU>::Data, <Self::ALU as ALU>::FlagSet)
    where
        Self: 'a + CPUAccumulator<AccSize = <Self::ALU as ALU>::Data>,
    {
        let lhs = self.acc_reader().read();
        let res = self.alu().op(control, lhs, rhs);
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
        flag_masks: &[<Self::ALU as ALU>::Flag],
    ) -> Self::FlagRegisterLoader<'a>
    where
        <Self::ALU as ALU>::Flag: Copy,
    {
        self.flag_loader_masked(<Self::ALU as ALU>::FlagSet::from_slice(flag_masks))
    }
    fn flag_reader<'a>(&'a self) -> Self::FlagRegisterReader<'a>;
    fn flag_on(&self, flag: <Self::ALU as ALU>::Flag) -> bool {
        let flags: <Self::ALU as ALU>::FlagSet = self.flag_reader().read().into();
        flags.is_set(flag)
    }
}

pub trait CPUProgramCounter: CPUMemory {
    type ProgramCounterLoader<'a>: RegisterLoader<Size = Self::MemoryAddress>
    where
        Self: 'a;
    type ProgramCounterReader<'a>: RegisterReader<Size = Self::MemoryAddress>
    where
        Self: 'a;
    fn program_counter_loader<'a>(&'a mut self) -> Self::ProgramCounterLoader<'a>;
    fn program_counter_reader<'a>(&'a self) -> Self::ProgramCounterReader<'a>;
    fn program_fetch(&mut self) -> Self::MemoryData;
}

pub trait CPUStackPointer: CPUMemory {
    type StackPointerLoader<'a>: RegisterLoader<Size = Self::MemoryAddress>
    where
        Self: 'a;
    type StackPointerReader<'a>: RegisterReader<Size = Self::MemoryAddress>
    where
        Self: 'a;
    fn stack_pointer_loader<'a>(&'a mut self) -> Self::StackPointerLoader<'a>;
    fn stack_pointer_reader<'a>(&'a self) -> Self::StackPointerReader<'a>;
    fn push(&mut self, bits: Self::MemoryData);
    fn pop(&mut self) -> Self::MemoryData;
}

pub trait CPUJump: CPUMemory + CPUProgramCounter {
    fn jump(&mut self, index: Self::MemoryAddress) {
        self.program_counter_loader().load(index);
    }
    fn jump_on<'a>(&'a mut self, index: Self::MemoryAddress, flag: <Self::ALU as ALU>::Flag)
    where
        Self: CPUFlagRegister,
        <Self::ALU as ALU>::FlagSet: From<<Self::FlagRegisterReader<'a> as RegisterReader>::Size>,
    {
        if self.flag_on(flag) {
            self.jump(index)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::cpu::{CPUMemory, CPUProgramCounter, CPUStackPointer};
    use crate::memory::typical::Memory8Bit64KB;
    use crate::memory::Memory;
    use crate::register::typical::{Register16, Register16Loader, Register16Reader};
    use crate::register::{Register, RegisterLoader, RegisterReader};
    use crate::instruction::Instruction;
    use crate::instruction::typical::*;

    #[derive(Debug, Default)]
    struct CPU8 {
        af: Register16,
        sp: Register16,
        pc: Register16,
        memory: Memory8Bit64KB,
    }

    impl CPU8 {
        fn new() -> Self {
            let mut sp = Register16::default();
            sp.load(0xfffe);
            let mut memory = Memory8Bit64KB::default();
            for i in 0..256 {
                memory.store(i, i as u8);
            }
            Self {
                sp,
                memory,
                ..Default::default()
            }
        }
    }

    impl CPUMemory for CPU8 {
        type MemoryAddress = <Memory8Bit64KB as Memory>::Address;
        type MemoryData = <Memory8Bit64KB as Memory>::Data;

        fn memory_store(&mut self, index: Self::MemoryAddress, data: Self::MemoryData) {
            self.memory.store(index, data)
        }

        fn memory_read(&self, index: Self::MemoryAddress) -> Self::MemoryData {
            self.memory.read(index)
        }
    }

    impl CPUProgramCounter for CPU8 {
        type ProgramCounterLoader<'a> = Register16Loader<'a> where Self: 'a;
        type ProgramCounterReader<'a> = Register16Reader<'a> where Self: 'a;

        fn program_counter_loader<'a>(&'a mut self) -> Self::ProgramCounterLoader<'a> {
            Register16Loader::new(&mut self.pc)
        }

        fn program_counter_reader<'a>(&'a self) -> Self::ProgramCounterReader<'a> {
            Register16Reader::new(&self.pc)
        }

        fn program_fetch(&mut self) -> Self::MemoryData {
            let index = self.program_counter_reader().read();
            let res = self.memory_read(index);
            self.program_counter_loader().load(index + 1);
            res
        }
    }

    impl CPUStackPointer for CPU8 {
        type StackPointerLoader<'a>  = Register16Loader<'a> where Self: 'a;
        type StackPointerReader<'a>  = Register16Reader<'a>  where Self: 'a;

        fn stack_pointer_loader<'a>(&'a mut self) -> Self::StackPointerLoader<'a> {
            Register16Loader::new(&mut self.sp)
        }

        fn stack_pointer_reader<'a>(&'a self) -> Self::StackPointerReader<'a> {
            Register16Reader::new(&self.sp)
        }

        fn push(&mut self, bits: Self::MemoryData) {
            let sp = self.stack_pointer_reader().read().wrapping_sub(1);
            self.stack_pointer_loader().load(sp);
            self.memory_store(sp, bits);
        }

        fn pop(&mut self) -> Self::MemoryData {
            let sp = self.stack_pointer_reader().read();
            self.stack_pointer_loader().load(sp.wrapping_add(1));
            self.memory_read(sp)
        }
    }

    #[test]
    fn pc() {
        let mut cpu = CPU8::new();
        assert_eq!(cpu.program_fetch(), 0);
        assert_eq!(cpu.program_fetch(), 1);
        assert_eq!(cpu.program_fetch(), 2);
        assert_eq!(cpu.program_fetch(), 3);
        Jump::new(31).execute(&mut cpu);
        assert_eq!(cpu.program_fetch(), 31);
        assert_eq!(cpu.program_fetch(), 32);
        assert_eq!(cpu.program_fetch(), 33);
    }

    #[test]
    fn sp() {
        let mut cpu = CPU8::new();
        cpu.push(3);
        cpu.push(1);
        cpu.push(4);
        cpu.push(1);
        Push::new(5).execute(&mut cpu);
        assert_eq!(cpu.pop(), 5);
        assert_eq!(cpu.pop(), 1);
        cpu.push(5);
        assert_eq!(cpu.pop(), 5);
        assert_eq!(cpu.pop(), 4);
        assert_eq!(cpu.pop(), 1);
        cpu.push(5);
        assert_eq!(cpu.pop(), 5);
        assert_eq!(cpu.pop(), 3);
    }
}
