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
    offset: [usize; 2],
    row_lengths: Vec<usize>,
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
            offset: [1, 1],
            row_lengths: vec![],
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
    /// assert_eq!(matrix.locus(), [2, 23]);
    /// ```
    pub fn locus(&self) -> [usize; 2] {
        self.offset
    }

    /// Gets an array range from a start position to the current position.
    pub fn get_range(&self, start: [usize; 2]) -> [usize; 4] {
        let position = self.locus();
        [start[0], start[1], position[0], position[1]]
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
    /// Shift the position backwards by 1 without calling [`next`].
    /// For the rest of the iteration the position returned will be behind the actual iterator position.
    /// # Panics
    /// It panics if the cuurent position is [1, 1], i.e. there is no left to rewind to.
    /// # Examples
    /// Basic usage:
    /// ```
    /// use sleek_utils::MatrixIterator;
    ///
    /// let main_iter = [
    ///     01, 02, 04, -1,
    ///     05, 07, 24, -1
    /// ].iter();
    ///
    /// let mut matrix = MatrixIterator::new(main_iter, &-1);
    ///
    /// matrix.next(); // Position is [1, 2]
    /// matrix.next(); // Position is [1, 3]
    ///
    /// matrix.left();  // Shifts position back. Position is now [1, 2]
    /// assert_eq!(matrix.locus(), [1, 2]);
    /// ```
    pub fn left(&mut self) {
        if self.offset[1] == 1 {
            if self.row_lengths.is_empty() {
                panic!("Cannot move left out of matrix bounds")
            }
            self.offset[0] = self.offset[0] - 1;
            self.offset[1] = self.row_lengths[self.offset[0] - 1];
        } else {
            self.offset[1] -= 1;
        }
    }
    pub fn right(&mut self) {}
    /// Set the matrix position to the position of the iterator.
    /// Undoes all effects of positional methods.
    pub fn cohere(&mut self) {
        self.offset = [0, 0]
    }
    fn move_offset(&mut self) {
        // No positional effect.
        if self.offset[0] < self.row {
            // Max row length, move to next.
            let offsets_row_length = self.row_lengths[self.offset[0] - 1];
            if self.offset[1] + 1 > offsets_row_length {
                self.offset[0] += 1;
                self.offset[1] = 1;
            } else {
                self.offset[1] += 1;
            }
        } else {
            self.offset[1] += 1;
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
                self.row_lengths.push(self.column);
                if self.offset == [self.row, self.column] {
                    self.offset[0] += 1;
                    self.offset[1] = 1;
                } else {
                    self.move_offset();
                }
                self.row += 1;
                self.column = 1;
            } else {
                self.move_offset();
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

#[cfg(test)]
mod tests {
    use crate::MatrixIterator;

    #[test]
    fn it_moves_matrix() {
        let mut matrix = MatrixIterator::new([1, 2, 0, 5, 7, 0, 8, 9, 0].iter(), &0);
        assert_eq!(matrix.locus(), [1, 1]);

        matrix.next();

        assert_eq!(matrix.locus(), [1, 2]);

        matrix.left();

        assert_eq!(matrix.locus(), [1, 1]);

        matrix.next();

        assert_eq!(matrix.locus(), [1, 2]);

        matrix.next();

        assert_eq!(matrix.locus(), [1, 3]);

        matrix.next();

        assert_eq!(matrix.locus(), [2, 1]);

        matrix.left();

        assert_eq!(matrix.locus(), [1, 3]);

        matrix.next(); // [2, 1]

        matrix.next(); // [2, 2]

        matrix.next(); // [2, 3];

        matrix.next(); // [3, 1]

        assert_eq!(matrix.locus(), [3, 1]);

        println!("{:?}", matrix.locus());
    }
}
