//! Doc comment forms.

/** A block doc comment
 * over two lines. */
pub struct Block;

#[doc = "An attribute doc."]
pub struct Attribute;

/**
 * A starred block doc.
 *
 * With a paragraph.
 */
pub struct Starred;

/// Line one.
#[doc = "Line two from an attribute."]
pub struct Mixed;
