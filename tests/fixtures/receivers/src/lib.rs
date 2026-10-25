//! Receivers.

use std::pin::Pin;
use std::rc::Rc;

/// Every receiver form.
pub struct Node;

impl Node {
    pub fn shared(&self) -> u8 {
        0
    }

    pub fn exclusive(&mut self) {}

    pub fn consume(self) {}

    pub fn consume_mut(mut self) {}

    pub fn boxed(self: Box<Self>) {}

    pub fn counted(self: Rc<Self>) {}

    pub fn pinned(self: Pin<&mut Self>) {}

    pub fn explicit(self: &Self) -> u8 {
        0
    }

    pub fn explicit_mut(self: &mut Self) {}

    pub fn lifetime<'a>(&'a self) -> &'a str {
        ""
    }

    pub fn none() {}
}

/// A free function never gets static.
pub fn free(node: &Node) {}
