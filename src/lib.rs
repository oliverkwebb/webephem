#[warn(missing_docs)]
/// The generation and managment of the star and planet data referenced
pub mod catalog;
pub mod parse;
/// pracstro provides a way to do this, but that isn't functional in a lot of contexts
///
/// Used in ephemeris generation and date reading
pub mod query;
pub mod value;
pub mod wasm;
