//! procedural parsing easily, making writing parsers for byte-encoded formats
//! very easy.
//!
//! For example:
//!
//! ```rust
//! use nom_derive::Nom;
//!
//! #[derive(Nom)]
//! struct S {
//!   a: u32,
//!   b: u16,
//!   c: u16
//! }
//! ```
//!
//! This adds a static method `parse` to `S`, with the following signature:
//! ```rust,ignore
//! impl S {
//!     pub fn parse(i: &[u8]) -> nom::IResult(&[u8], S);
//! }
//! ```
//!
//! To parse input, just call `let res = S::parse(input);`.
//!
//! For extensive documentation of all attributes and examples, see the [Nom derive
//! attribute](derive.Nom.html) documentation.
//!
//! Many examples are provided, and more can be found in the [project
//! tests](https://github.com/rust-bakery/nom-derive/tree/master/tests).
//!
//! ## Combinators visibility
//!
//! All inferred parsers will generate code with absolute type path, so there is no need
//! to add `use` statements for them. However, if you use any combinator directly (or in a `Parse`
//! statement, for ex.), it has to be imported as usual.
//!
//! That is probably not going to change, since
//! * a proc_macro cannot export items other than functions tagged with `#[proc_macro_derive]`
//! * there are variants of combinators with the same names (complete/streaming, bits/bytes), so
//!   re-exporting them would create side-effects.
//!
//! ## Debug tips
//!
//! * If the generated parser does not compile, add `#[nom(DebugDerive)]` to the structure.
//!   It will dump the generated parser to `stderr`.
//! * If the generated parser fails at runtime, try adding `#[nom(Debug)]` to the structure or
//!   to fields. It wraps subparsers in `dbg_dmp` and will print the field name and input to
//!   `stderr` if the parser fails.
//!
//! [nom]: https://github.com/geal/nom

mod traits;

pub use traits::*;

pub use nom::IResult;
pub use nom_derive_impl::*;
