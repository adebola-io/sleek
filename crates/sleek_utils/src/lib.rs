#![allow(incomplete_features)]
#![feature(return_position_impl_trait_in_trait)]

mod high_order_iterator;
mod matrix_iterator;
mod node;
mod queue_iterator;

use std::{cell::RefCell, rc::Rc};

pub use high_order_iterator::HigherOrderIterator;
pub use matrix_iterator::MatrixIterator;
pub use node::Node;
pub use queue_iterator::QueueIterator;

pub type MutableCountRef<T> = Rc<RefCell<T>>;
