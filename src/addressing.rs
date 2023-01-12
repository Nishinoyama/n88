pub trait Addressing<C> {
    type Size;
    fn value(&self, cpu: &C) -> Self::Size;
}
