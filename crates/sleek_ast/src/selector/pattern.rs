use crate::HtmlTag;

use super::Selector;

#[derive(PartialEq, Debug)]
pub enum PseudoClass {
    Root,
    Empty,
}

#[derive(PartialEq, Debug)]
pub enum SelectorPattern {
    Universal,
    Tag(HtmlTag),
    Class(String),
    Id(String),
    Attribute(String, Option<String>),
    Descendant([Selector; 2]),
    Child([Selector; 2]),
    // Parent([Selector; 2]),
    AdjacentSibling([Selector; 2]),
    GeneralSibling([Selector; 2]),
    Group(Vec<Selector>),
    PseudoClass(PseudoClass),
}
