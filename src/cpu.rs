use crate::alu::{FlagSet, ALU};

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
        let lhs = self.acc_read();
        self.alu().op(control, lhs, rhs)
    }
}

pub trait CPUAccumulator {
    type AccSize;
    fn acc_load(&mut self, bits: Self::AccSize);
    fn acc_read(&self) -> Self::AccSize;
}

pub trait CPUFlagRegister: CPUAlu {
    type FlagRegisterSize: Into<<Self::ALU as ALU>::FlagSet>;
    fn flag_load(&mut self, bits: Self::FlagRegisterSize) {
        self.flag_load_masked(<Self::ALU as ALU>::FlagSet::all_on(), bits);
    }
    fn flag_load_masked(
        &mut self,
        flag_mask: <Self::ALU as ALU>::FlagSet,
        bits: Self::FlagRegisterSize,
    );
    fn flag_load_mask_slice(
        &mut self,
        flag_masks: &[<Self::ALU as ALU>::Flag],
        bits: Self::FlagRegisterSize,
    ) where
        <Self::ALU as ALU>::Flag: Copy,
    {
        self.flag_load_masked(<Self::ALU as ALU>::FlagSet::from_slice(flag_masks), bits);
    }
    fn flag_read(&self) -> Self::FlagRegisterSize;
    fn flag_on(&self, flag: <Self::ALU as ALU>::Flag) -> bool {
        let flags: <Self::ALU as ALU>::FlagSet = self.flag_read().into();
        flags.is_set(flag)
    }
}

pub trait CPUProgramCounter: CPUMemory {
    fn program_counter_load(&mut self, bits: Self::MemoryAddress);
    fn program_counter_read(&self) -> Self::MemoryAddress;
    fn program_fetch(&mut self) -> Self::MemoryData;
}

pub trait CPUStackPointer: CPUMemory {
    fn stack_pointer_load(&mut self, bits: Self::MemoryAddress);
    fn stack_pointer_read(&self) -> Self::MemoryAddress;
    fn push(&mut self, bits: Self::MemoryData);
    fn pop(&mut self) -> Self::MemoryData;
}

pub trait CPUJump: CPUMemory + CPUProgramCounter {
    fn jump(&mut self, index: Self::MemoryAddress) {
        self.program_counter_load(index);
    }
    fn jump_on(&mut self, index: Self::MemoryAddress, flag: <Self::ALU as ALU>::Flag)
    where
        Self: CPUFlagRegister,
        <Self::ALU as ALU>::FlagSet: From<Self::FlagRegisterSize>,
    {
        if self.flag_on(flag) {
            self.jump(index)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::cpu::{CPUMemory, CPUProgramCounter, CPUStackPointer};
    use crate::instruction::typical::*;
    use crate::instruction::Instruction;
    use crate::memory::typical::Memory8Bit64KB;
    use crate::memory::Memory;
    use crate::register::typical::{Register16, Register16Loader, Register16Reader};
    use crate::register::{Register, RegisterLoader, RegisterReader};

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
        fn program_counter_load(&mut self, bits: Self::MemoryAddress) {
            Register16Loader::new(&mut self.pc).load(bits)
        }

        fn program_counter_read(&self) -> Self::MemoryAddress {
            Register16Reader::new(&self.pc).read()
        }

        fn program_fetch(&mut self) -> Self::MemoryData {
            let index = self.program_counter_read();
            let res = self.memory_read(index);
            self.program_counter_load(index.wrapping_add(1));
            res
        }
    }

    impl CPUStackPointer for CPU8 {
        fn stack_pointer_load(&mut self, bits: Self::MemoryAddress) {
            Register16Loader::new(&mut self.sp).load(bits)
        }

        fn stack_pointer_read(&self) -> Self::MemoryAddress {
            Register16Reader::new(&self.sp).read()
        }

        fn push(&mut self, bits: Self::MemoryData) {
            let sp = self.stack_pointer_read().wrapping_sub(1);
            self.stack_pointer_load(sp);
            self.memory_store(sp, bits);
        }

        fn pop(&mut self) -> Self::MemoryData {
            let sp = self.stack_pointer_read();
            self.stack_pointer_load(sp.wrapping_add(1));
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
