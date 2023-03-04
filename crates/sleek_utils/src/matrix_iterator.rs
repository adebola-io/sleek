use super::HigherOrderIterator;

/// Compound iterator that can track rows and columns.
/// Useful for keeping track of rows and columns when iterating through two-dimensional data.
pub struct MatrixIterator<I>
where
    I: Iterator,
    I::Item: PartialEq,
{
    row: usize,
    column: usize,
    input: I,
    delineator: I::Item,
}

impl<I> MatrixIterator<I>
where
    I: Iterator,
    I::Item: PartialEq,
{
    /// Creates a new matrix iterator.
    /// The delineator is the item match that marks the end of the row.
    pub fn new(input: I, delineator: I::Item) -> Self {
        MatrixIterator {
            input,
            row: 1,
            column: 1,
            delineator,
        }
    }
    /// Get current location in the matrix.
    /// # Examples
    /// ```
    /// use sleek_utils::MatrixIterator;
    ///
    /// let main_iter = "\
    ///     This is the first line.
    ///     This is the second.
    ///     This is the third.
    /// ".chars();
    ///
    /// let mut matrix = MatrixIterator::new(main_iter, '\n');
    /// matrix.nth(45);
    ///
    /// assert_eq!(matrix.get_position(), [2, 23]);
    /// ```
    pub fn get_position(&self) -> [usize; 2] {
        [self.row, self.column]
    }

    /// Gets an array range from a start position to the current position.
    pub fn get_range(&self, start: [usize; 2]) -> [usize; 4] {
        [start[0], start[1], self.row, self.column]
    }

    /// Consumes the entire iterator and return the number of rows in the matrix.
    /// # Examples
    /// ```
    /// use sleek_utils::MatrixIterator;
    ///
    /// let main_iter =
    ///     [1, 2, 3, 0,
    ///      4, 5, 6, 0,
    ///      7, 8, 9, 0 ].iter();
    ///
    /// let mut matrix = MatrixIterator::new(main_iter, &0);
    ///
    /// assert_eq!(matrix.rows(), 3);
    ///
    /// ```
    pub fn rows(&mut self) -> usize {
        match self.last() {
            Some(value) => {
                if value == self.delineator {
                    self.row - 1
                } else {
                    self.row
                }
            }
            None => self.row,
        }
    }
}

impl<I> Iterator for MatrixIterator<I>
where
    I: Iterator,
    I::Item: PartialEq,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.input.next().map(|item| {
            if item == self.delineator {
                self.row += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            item
        })
    }
}

impl<I> HigherOrderIterator<I> for MatrixIterator<I>
where
    I: Iterator,
    I::Item: PartialEq,
{
    fn inner(&self) -> &I {
        &self.input
    }

    fn inner_mut(&mut self) -> &mut I {
        &mut self.input
    }
}
