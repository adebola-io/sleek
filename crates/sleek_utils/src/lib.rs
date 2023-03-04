mod high_order_iterator;
mod matrix_iterator;
mod node;
mod stack_iterator;

use std::{cell::RefCell, rc::Rc};

pub use high_order_iterator::HigherOrderIterator;
pub use matrix_iterator::MatrixIterator;
pub use node::Node;
pub use stack_iterator::StackIterator;

pub type MutableCountRef<T> = Rc<RefCell<T>>;
