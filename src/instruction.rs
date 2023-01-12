pub trait Instruction<CPU> {
    fn execute(self, cpu: &mut CPU);
}

pub mod typical {
    use super::*;
    use crate::addressing::Addressing;
    use crate::alu::ALU;
    use crate::cpu::*;
    use crate::register::*;

    pub struct Jump<A> {
        address: A,
    }

    impl<CPU, A> Instruction<CPU> for Jump<A>
    where
        CPU: CPUProgramCounter<MemoryAddress = A>,
        A: Copy,
    {
        fn execute(self, cpu: &mut CPU) {
            cpu.program_counter_load(self.address);
        }
    }

    impl<A> Jump<A> {
        pub fn new(address: A) -> Self {
            Self { address }
        }
    }

    pub struct Push<B> {
        data: B,
    }

    impl<CPU, B> Instruction<CPU> for Push<B>
    where
        CPU: CPUStackPointer<MemoryData = B>,
        B: Copy,
    {
        fn execute(self, cpu: &mut CPU) {
            cpu.push(self.data)
        }
    }

    impl<B> Push<B> {
        pub fn new(data: B) -> Self {
            Self { data }
        }
    }

    pub struct Pop<C> {
        dst: C,
    }

    impl<CPU, C, B> Instruction<CPU> for Pop<C>
    where
        CPU: CPUStackPointer<MemoryData = B> + RegisterSet<C, Size = B>,
    {
        fn execute(self, cpu: &mut CPU) {
            let src = cpu.pop();
            cpu.load_of(self.dst, src)
        }
    }

    pub struct Condition<F, I> {
        cond: F,
        then: I,
    }

    impl<CPU, F, I> Instruction<CPU> for Condition<F, I>
    where
        F: FnOnce(&CPU) -> bool,
        I: Instruction<CPU>,
    {
        fn execute(self, cpu: &mut CPU) {
            if (self.cond)(cpu) {
                self.then.execute(cpu);
            }
        }
    }

    pub struct Load<C, A> {
        dst: C,
        src: A,
    }

    impl<C, A> Load<C, A> {
        pub fn new(dst: C, src: A) -> Self {
            Self { dst, src }
        }
    }

    impl<CPU, C, B, A> Instruction<CPU> for Load<C, A>
    where
        CPU: RegisterSet<C, Size = B>,
        C: Copy,
        A: Addressing<CPU, Size = B> + Copy,
    {
        fn execute(self, cpu: &mut CPU) {
            let bits = self.src.value(cpu);
            cpu.load_of(self.dst, bits);
        }
    }

    pub struct Store<D, S> {
        dst: D,
        src: S,
    }

    impl<D, S> Store<D, S> {
        pub fn new(dst: D, src: S) -> Self {
            Self { dst, src }
        }
    }

    impl<CPU, D, S, A, B> Instruction<CPU> for Store<D, S>
    where
        CPU: CPUMemory<MemoryAddress = A, MemoryData = B>,
        D: Addressing<CPU, Size = A> + Copy,
        S: Addressing<CPU, Size = B> + Copy,
    {
        fn execute(self, cpu: &mut CPU) {
            let dst = self.dst.value(cpu);
            let src = self.src.value(cpu);
            cpu.memory_store(dst, src);
        }
    }

    pub struct Arithmetic<C, F, D, L> {
        control: C,
        flags: Vec<F>,
        dst: D,
        rhs: L,
    }

    impl<C, F, D, L> Arithmetic<C, F, D, L> {
        pub fn new(control: C, flags: Vec<F>, dst: D, rhs: L) -> Self {
            Self {
                control,
                flags,
                dst,
                rhs,
            }
        }
    }

    impl<CPU, A, C, F, D, L, B, G> Instruction<CPU> for Arithmetic<C, F, D, L>
    where
        CPU: CPUAccumulator<AccSize = B>
            + CPUFlagRegister<ALU = A, FlagRegisterSize = G>
            + RegisterSet<D, Size = B>,
        A: ALU<Data = B, Control = C, Flag = F>,
        C: Copy,
        F: Copy,
        L: Addressing<CPU, Size = B>,
        G: From<A::FlagSet>,
    {
        fn execute(self, cpu: &mut CPU) {
            let rhs = self.rhs.value(cpu);
            let (res, flag) = cpu.alu_acc_op(self.control, rhs);
            cpu.flag_load_mask_slice(&self.flags, flag.into());
            cpu.load_of(self.dst, res);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::instruction::Instruction;

    #[derive(Debug, Default)]
    struct CPU8 {
        a: u8,
        b: u8,
    }

    enum Instructions<CPU> {
        LoadA(u8),
        LoadB(u8),
        Add,
        Etc(Box<dyn FnOnce(&mut CPU)>),
    }

    impl Instruction<CPU8> for Instructions<CPU8> {
        fn execute(self, cpu: &mut CPU8) {
            use Instructions::*;
            match self {
                LoadA(a) => cpu.a = a,
                LoadB(b) => cpu.b = b,
                Add => cpu.a = cpu.a.wrapping_add(cpu.b),
                Etc(f) => f(cpu),
            }
        }
    }

    #[test]
    fn instruction() {
        use Instructions::*;
        let mut cpu = CPU8::default();
        LoadA(36).execute(&mut cpu);
        LoadB(17).execute(&mut cpu);
        Add.execute(&mut cpu);
        assert_eq!(cpu.a, 53);
        let inc = Etc(Box::new(|cpu: &mut CPU8| cpu.a += 1));
        inc.execute(&mut cpu);
        assert_eq!(cpu.a, 54);
        let left_shift = |i| Box::new(move |cpu: &mut CPU8| cpu.a <<= i);
        Etc(left_shift(3)).execute(&mut cpu);
        assert_eq!(cpu.a, 176);
    }
}
