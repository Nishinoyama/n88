pub trait Instruction<CPU> {
    fn execute(self, cpu: &mut CPU);
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
