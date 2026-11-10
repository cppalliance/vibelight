//! The HTML outline: each declaration laid out as styled lines, the document
//! a crate's page is built from, the renderer that prints it, a facade's
//! page, the overview, and the one file that holds every page.

pub(crate) mod browse;
pub(crate) mod document;
pub(crate) mod facade_page;
mod glance;
pub(crate) mod layout;
mod legend;
pub(crate) mod order;
pub(crate) mod render_html;
mod render_prose;
mod render_spans;
mod render_style;
mod render_values;
pub(crate) mod root_page;
mod wrap;
