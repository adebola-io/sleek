use super::HigherOrderIterator;

/// A compound iterator that allows you to add elements in front of the sequence and give them priority.
/// # Examples
/// Basic usage:
/// ```
/// use sleek_utils::StackIterator;
///
/// let main_iter = [1, 2, 3].iter();
/// let mut stack_iter = StackIterator::new(main_iter);
///
/// assert_eq!(stack_iter.next(), Some(&1));
///
/// // Add an element in front.
/// stack_iter.push(&6);
/// assert_eq!(stack_iter.next(), Some(&6));
///
/// assert_eq!(stack_iter.next(), Some(&2));
/// ```
pub struct StackIterator<I>
where
    I: Iterator,
{
    front: Vec<I::Item>,
    input: I,
}

impl<I> StackIterator<I>
where
    I: Iterator,
{
    pub fn new(input: I) -> Self {
        StackIterator {
            input,
            front: vec![],
        }
    }
    pub fn push(&mut self, item: I::Item) {
        self.front.push(item)
    }
}

impl<I> Iterator for StackIterator<I>
where
    I: Iterator,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.front.is_empty() {
            self.front.pop()
        } else {
            self.input.next()
        }
    }
}

impl<I> HigherOrderIterator<I> for StackIterator<I>
where
    I: Iterator,
{
    fn inner(&self) -> &I {
        &self.input
    }

    fn inner_mut(&mut self) -> &mut I {
        &mut self.input
    }
}
