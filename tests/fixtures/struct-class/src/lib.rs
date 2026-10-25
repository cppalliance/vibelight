//! Struct versus class.

/// Public and private members.
pub struct Mixed {
    pub open: u8,
    closed: u8,
}

/// Holds the text the lexer is still reading.
pub struct Cursor {
    text: String,
    pos: usize,
}

impl Cursor {
    fn new(text: String) -> Cursor {
        Cursor { text, pos: 0 }
    }

    fn peek(&self) -> Option<char> {
        None
    }

    fn bump(&mut self) {}

    fn into_text(self) -> String {
        self.text
    }
}

/// No members at all.
pub struct Bare {}

/// An enum with only private methods.
pub enum Toggle {
    On,
    Off,
}

impl Toggle {
    fn flip(&mut self) {}
}

/// A tuple struct with only private fields.
pub struct Wrapper(u8);
