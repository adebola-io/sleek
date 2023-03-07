/// Defines an interface for iterators that wrap around other iterators.
pub trait HigherOrderIterator<T: Iterator>: Iterator {
    fn inner(&self) -> &T;
    fn inner_mut(&mut self) -> &mut T;
    /// Collect the next n values in the iteration.
    fn collect_next<B>(&mut self, index: usize) -> B
    where
        B: FromIterator<Self::Item>,
        Self: Sized,
    {
        let mut collection = vec![];
        for _ in 0..index {
            if let Some(value) = self.next() {
                collection.push(value)
            }
        }
        B::from_iter(collection)
    }
}
