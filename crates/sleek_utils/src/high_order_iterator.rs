/// Defines an interface for iterators that wrap around other iterators.
pub trait HigherOrderIterator<T: Iterator> {
    fn inner(&self) -> &T;
    fn inner_mut(&mut self) -> &mut T;
}
