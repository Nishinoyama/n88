use crate::alu::{FlagSet, ALU};
use crate::memory::Memory;
use crate::register::typical::Register16In8Loader;
use crate::register::{
    Register, RegisterCode, RegisterDecrementable, RegisterIncrementable, RegisterLoader,
    RegisterReader,
};

pub enum CPURunningState {
    Running,
    Halted,
    Error,
}

pub trait CPU: Sized {
    type Data: Copy + Register;
    type Address: Copy + Register;
    fn data(&self) -> Self::Data;
    fn address(&self) -> Self::Address;
    fn load_data(self, data: Self::Data) -> Self;
    fn load_address(self, address: Self::Address) -> Self;
    fn cycle(self) -> Self;
    fn run(self) -> Option<Self>;
}

pub trait CPUMemory<M>: CPU
where
    M: Memory<Data = Self::Data, Address = Self::Address>,
{
    fn store_memory(self, memory: &mut M) -> Self {
        memory.store(self.address(), self.data());
        self
    }
    fn fetch_memory(self, memory: &M) -> Self {
        let address = self.address();
        self.load_data(memory.read(address))
    }
}

/// todo: ALUの素晴らしい設計を後で考える
pub trait CPUAlu: CPU {
    type ALU: ALU;
}

pub trait CPURegisters<C: RegisterCode<Register = Self::Register>>: CPU {
    type Register;
    fn read_of(&self, code: C) -> Self::Register;
    fn load_of(self, code: C, bits: Self::Register) -> Self;
    fn read_of_as_data(self, code: C) -> Self
    where
        Self: CPU<Data = Self::Register>,
    {
        let data = self.read_of(code);
        self.load_data(data)
    }
    fn read_of_as_address(self, code: C) -> Self
    where
        Self: CPU<Address = Self::Register>,
    {
        let address = self.read_of(code);
        self.load_address(address)
    }
}

pub trait CPUAccumulator: CPU {
    fn acc(&mut self) -> &mut Self::Data;
    fn acc_load(self) -> Self;
    fn acc_read(self) -> Self;
}

/// todo: ALUができたらやる
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

pub trait CPUProgramCounter: CPU {
    fn program_counter(&mut self) -> &mut Self::Address;
    fn program_counter_read(mut self) -> Self {
        let address = *self.program_counter();
        self.load_address(address)
    }
    /// fixme: u16 and u8 hardcode.
    fn program_counter_load_high(mut self) -> Self
    where
        Self: CPU<Address = u16, Data = u8>,
    {
        let data = self.data();
        Register16In8Loader::new(self.program_counter(), true).load(data);
        self
    }
    /// fixme: u16 and u8 hardcode.
    fn program_counter_load_low(mut self) -> Self
    where
        Self: CPU<Address = u16, Data = u8>,
    {
        let data = self.data();
        Register16In8Loader::new(self.program_counter(), false).load(data);
        self
    }
    fn program_fetch<M>(self, memory: &M) -> Self
    where
        Self: CPUMemory<M>,
        M: Memory<Data = Self::Data, Address = Self::Address>,
        Self::Address: RegisterIncrementable,
    {
        let mut temp = self.program_counter_read().fetch_memory(memory);
        temp.program_counter().increment();
        temp
    }
}

pub trait CPUStackPointer: CPU {
    fn stack_pointer(&mut self) -> &mut Self::Address;
    fn stack_pointer_read(mut self) -> Self {
        let address = *self.stack_pointer();
        self.load_address(address)
    }
    /// fixme: u16 and u8 hardcode.
    fn stack_pointer_load_high(mut self) -> Self
    where
        Self: CPU<Address = u16, Data = u8>,
    {
        let data = self.data();
        Register16In8Loader::new(self.stack_pointer(), true).load(data);
        self
    }
    /// fixme: u16 and u8 hardcode.
    fn stack_pointer_load_low(mut self) -> Self
    where
        Self: CPU<Address = u16, Data = u8>,
    {
        let data = self.data();
        Register16In8Loader::new(self.stack_pointer(), false).load(data);
        self
    }

    fn push<M>(mut self, memory: &mut M) -> Self
    where
        Self: CPUMemory<M>,
        M: Memory<Data = Self::Data, Address = Self::Address>,
        Self::Address: RegisterDecrementable,
    {
        self.stack_pointer().decrement();
        self.stack_pointer_read().store_memory(memory)
    }
    fn pop<M>(self, memory: &M) -> Self
    where
        Self: CPUMemory<M>,
        M: Memory<Data = Self::Data, Address = Self::Address>,
        Self::Address: RegisterIncrementable,
    {
        let mut temp = self.stack_pointer_read().fetch_memory(memory);
        temp.stack_pointer().increment();
        temp
    }
}

pub trait CPUJump: CPU + CPUProgramCounter {
    fn jump(mut self, address: Self::Address) -> Self {
        self.load_address(address)
    }
    /// fixme: u16 and u8 hardcode.
    fn jump_high(mut self) -> Self
    where
        Self: CPU<Address = u16, Data = u8>,
    {
        let data = self.data();
        Register16In8Loader::new(self.program_counter(), true).load(data);
        self
    }
    /// fixme: u16 and u8 hardcode.
    fn jump_low(mut self) -> Self
    where
        Self: CPU<Address = u16, Data = u8>,
    {
        let data = self.data();
        Register16In8Loader::new(self.program_counter(), false).load(data);
        self
    }
    /// todo: ALUが実装できたら
    fn jump_on(&mut self, index: Self::Address, flag: <Self::ALU as ALU>::Flag)
    where
        Self: CPUFlagRegister,
        <Self::ALU as ALU>::FlagSet: From<Self::FlagRegisterSize>;
}

#[cfg(test)]
mod tests {
    use crate::cpu::{CPUMemory, CPUProgramCounter, CPUStackPointer, CPU};
    use crate::memory::typical::Memory8Bit64KB;
    use crate::memory::Memory;

    #[derive(Debug, Default, Copy, Clone)]
    struct CPU8 {
        data: u8,
        af: u16,
        sp: u16,
        pc: u16,
        address: u16,
    }

    impl CPU for CPU8 {
        type Data = u8;
        type Address = u16;

        fn data(&self) -> Self::Data {
            self.data
        }

        fn address(&self) -> Self::Address {
            self.address
        }

        fn load_data(mut self, data: Self::Data) -> Self {
            self.data = data;
            self
        }

        fn load_address(mut self, address: Self::Address) -> Self {
            self.address = address;
            self
        }

        fn cycle(self) -> Self {
            unimplemented!()
        }

        fn run(self) -> Option<Self> {
            unimplemented!()
        }
    }

    impl CPUMemory<Memory8Bit64KB> for CPU8 {}

    impl CPUProgramCounter for CPU8 {
        fn program_counter(&mut self) -> &mut Self::Address {
            &mut self.pc
        }
    }

    impl CPUStackPointer for CPU8 {
        fn stack_pointer(&mut self) -> &mut Self::Address {
            &mut self.sp
        }
    }

    #[test]
    fn pc() {
        let mut memory = Memory8Bit64KB::default();
        for i in 0..256 {
            memory.store(i, i as u8);
        }
        let cpu = (0..=3).fold(CPU8::default(), |cpu, _| cpu.program_fetch(&memory));
        println!("{:?}", cpu);
        assert_eq!(cpu.data(), 3);
        let cpu = (0..=123).fold(CPU8::default(), |cpu, _| cpu.program_fetch(&memory));
        assert_eq!(cpu.data(), 123);
    }

    #[test]
    fn sp() {
        let mut cpu = CPU8::default();
        let mut memory = Memory8Bit64KB::default();
        *cpu.stack_pointer() = 256;
        let cpu = cpu
            .load_data(3)
            .push(&mut memory)
            .load_data(1)
            .push(&mut memory)
            .load_data(4)
            .push(&mut memory)
            .load_data(1)
            .push(&mut memory)
            .load_data(5)
            .push(&mut memory);
        let cpu = cpu.pop(&mut memory);
        assert_eq!(cpu.data(), 5);
        let cpu = cpu.pop(&mut memory);
        assert_eq!(cpu.data(), 1);
        let cpu = cpu.pop(&mut memory);
        assert_eq!(cpu.data(), 4);
        let cpu = cpu.pop(&mut memory);
        assert_eq!(cpu.data(), 1);
        let cpu = cpu.pop(&mut memory);
        assert_eq!(cpu.data(), 3);
    }
}
