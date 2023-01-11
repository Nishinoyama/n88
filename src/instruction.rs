pub trait Instruction<CPU> {
    fn execute(self, cpu: &mut CPU);
}

pub mod typical {
    use super::*;
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
            cpu.program_counter_loader().load(self.address);
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
        dst: C
    }

    impl<CPU, C, B> Instruction<CPU> for Pop<C>
        where
            CPU: CPUStackPointer<MemoryData = B> + RegisterSet<C, Size = B>,
    {
    {
        fn execute(self, cpu: &mut CPU) {
            let src = cpu.pop();
            cpu.loader_of(self.dst).load(src);
        }
    }

    pub struct Condition<F, I> {
        cond: F,
        then: I,
    }

    impl<CPU, F, I> Instruction<CPU> for Condition<F, I>
        where
            F: Fn(&CPU) -> bool,
            I: Instruction<CPU>,
    {
        fn execute(self, cpu: &mut CPU) {
            if (self.cond)(cpu) {
                self.then.execute(cpu);
            }
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
        Etc(Box<dyn Fn(&mut CPU)>),
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
