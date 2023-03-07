use std::{collections::VecDeque, rc::Rc};

use super::HigherOrderIterator;

/// A compound iterator that allows you to add elements in front of the sequence and give them priority.
/// # Examples
/// Basic usage:
/// ```
/// use sleek_utils::QueueIterator;
///
/// let main_iter = [1, 2, 3].iter();
/// let mut iter = QueueIterator::new(main_iter);
///
/// assert_eq!(iter.next(), Some(&1));
///
/// // Add an element in front.
/// iter.push(&6);
/// assert_eq!(iter.next(), Some(&6));
///
/// assert_eq!(iter.next(), Some(&2));
/// ```
pub struct QueueIterator<I: Iterator>
where
    I: Iterator,
{
    front: VecDeque<I::Item>,
    input: I,
    _f: Option<Rc<dyn Fn(&mut I)>>,
}

impl<'a, I> QueueIterator<I>
where
    I: Iterator,
{
    pub fn new(input: I) -> Self {
        QueueIterator {
            input,
            front: VecDeque::new(),
            _f: None,
        }
    }
    /// Put an item in front of the iterator.
    /// ```
    /// use sleek_utils::QueueIterator;
    ///
    /// let main_iter = [4, 7].iter();
    /// let mut iter = QueueIterator::new(main_iter);
    ///
    /// assert_eq!(iter.next(), Some(&4));
    ///
    /// // Add an element in front.
    /// iter.push(&5);
    /// assert_eq!(iter.next(), Some(&5));
    ///
    /// // Add another element in front.
    /// iter.push(&6);
    /// iter.push(&7);

    /// assert_eq!(iter.next(), Some(&6));
    /// assert_eq!(iter.next(), Some(&7));
    /// ```
    pub fn push(&mut self, item: I::Item) {
        self.front.push_back(item);

        if let Some(listener) = &self._f {
            listener(&mut self.input)
        }
    }
    pub fn on_push(&mut self, f: Rc<dyn Fn(&mut I)>) {
        self._f = Some(f);
    }
    /// Advances the iterator until it reaches the first item that matches a predicate, or it reaches the end of the iterator.
    ///
    /// The difference between this and [`Iterator::find`] is that find returns the element found,
    /// while [`next_until`](QueueIterator::until) stops right before it.
    /// # Examples
    /// ```
    /// use sleek_utils::QueueIterator;
    ///
    /// let mut iter = QueueIterator::new([4, 3, 1, 6, 7].iter());
    ///
    /// iter.next_until(|x| x > &&5);
    ///
    /// assert_eq!(iter.next(), Some(&6));
    /// assert_eq!(iter.next(), Some(&7));
    ///
    /// ```
    pub fn next_until<F>(&mut self, f: F)
    where
        F: Fn(&I::Item) -> bool,
    {
        loop {
            if let Some(item) = self.next() {
                if f(&item) {
                    self.push(item);
                    break;
                }
            } else {
                break;
            }
        }
    }
    /// Advances the iterator until it reaches the first item that does not matches a predicate.
    ///
    /// # Examples
    /// ```
    /// use sleek_utils::QueueIterator;
    ///
    /// let mut iter = QueueIterator::new([1, 2, 4, 5, 10].iter());
    ///
    /// iter.next_while(|x| x < &&5);
    ///
    /// assert_eq!(iter.next(), Some(&5));
    /// assert_eq!(iter.next(), Some(&10));
    ///
    /// ```
    pub fn next_while<F>(&mut self, f: F)
    where
        F: Fn(&I::Item) -> bool,
    {
        loop {
            if let Some(item) = self.next() {
                if !f(&item) {
                    self.push(item);
                    break;
                }
            } else {
                break;
            }
        }
    }
    /// Gather the succeeding items into the collection until there is an item that matches the predicate.
    ///
    /// # Examples
    /// ```
    /// use sleek_utils::QueueIterator;
    ///
    /// let mut iter = QueueIterator::new("The number is 1234".chars());
    ///
    /// let x: String = iter.collect_until(|ch| ch.is_numeric());
    ///
    /// assert_eq!(x, "The number is ");
    ///
    /// assert_eq!(iter.next(), Some('1'));
    /// assert_eq!(iter.next(), Some('2'));
    ///
    /// ```
    pub fn collect_until<B, F>(&mut self, f: F) -> B
    where
        B: FromIterator<I::Item>,
        F: Fn(&I::Item) -> bool,
    {
        let mut collect = vec![];
        loop {
            if let Some(item) = self.next() {
                if f(&item) {
                    self.push(item);
                    break;
                } else {
                    collect.push(item);
                }
            } else {
                break;
            }
        }
        B::from_iter(collect)
    }
}

impl<I> Iterator for QueueIterator<I>
where
    I: Iterator,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.front.is_empty() {
            self.front.pop_front()
        } else {
            self.input.next()
        }
    }
}

impl<I> HigherOrderIterator<I> for QueueIterator<I>
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
