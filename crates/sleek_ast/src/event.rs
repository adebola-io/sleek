#![allow(unused)]
#[derive(Debug, Clone)]
pub enum HtmlEvent {
    Click,
    MouseOver,
    Scroll,
    Change,
    KeyDown,
    KeyUp,
}

#[derive(Debug, Clone)]
pub struct HtmlEventListener {
    event: HtmlEvent,
}
