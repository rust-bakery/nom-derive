//! # nom-derive
//!
//! [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE-MIT)
//! [![Apache License 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](./LICENSE-APACHE)
//! [![docs.rs](https://docs.rs/nom-derive/badge.svg)](https://docs.rs/nom-derive)
//! [![Build Status](https://travis-ci.org/chifflier/nom-derive.svg?branch=master)](https://travis-ci.org/chifflier/nom-derive)
//! [![Crates.io Version](https://img.shields.io/crates/v/nom-derive.svg)](https://crates.io/crates/nom-derive)
//!
//! ## Overview
//!
//! nom-derive is a custom derive attribute, to derive [nom] parsers automatically from the structure definition.
//!
//! It is not meant to replace [nom], but to provide a quick and easy way to generate parsers for
//! structures, especially for simple structures. This crate aims at simplifying common cases.
//! In some cases, writing the parser manually will remain more efficient.
//!
//! - [API documentation](https://docs.rs/nom-derive)
//! - The [docs::Nom] pseudo-module. This is the main
//!   documentation for the `Nom` attribute, with all possible options and many examples.
//!
//! *Feedback welcome !*
//!
//! ## `#[derive(Nom)]`
//!
//! This crate exposes a single custom-derive macro `Nom` which
//! implements `parse` for the struct it is applied to.
//!
//! The goal of this project is that:
//!
//! * `derive(Nom)` should be enough for you to derive [nom] parsers for simple
//!   structures easily, without having to write it manually
//! * it allows overriding any parsing method by your own
//! * it allows using generated parsing functions along with handwritten parsers and
//!   combining them without efforts
//! * it remains as fast as nom
//!
//! `nom-derive` adds declarative parsing to `nom`. It also allows mixing with
//! procedural parsing easily, making writing parsers for byte-encoded formats
//! very easy.
//!
//! For example:
//!
//! ```rust
//! use nom_derive::*;
//!
//! #[derive(Nom)]
//! struct S {
//!   a: u32,
//!   b: u16,
//!   c: u16
//! }
//! ```
//!
//! This generates an implementation of the [`Parse`] trait to `S`. The generated code looks
//! like (code simplified):
//! ```rust,ignore
//! impl<'a> Parse<&'a> for S {
//!     pub fn parse_be(i: &'a [u8]) -> nom::IResult(&'a [u8], S) {
//!         let (i, a) = be_u32(i)?;
//!         let (i, b) = be_u16(i)?;
//!         let (i, c) = be_u16(i)?;
//!         Ok((i, S{ a, b, c }))
//!     }
//!     pub fn parse_le(i: &'a [u8]) -> nom::IResult(&'a [u8], S) {
//!         let (i, a) = le_u32(i)?;
//!         let (i, b) = le_u16(i)?;
//!         let (i, c) = le_u16(i)?;
//!         Ok((i, S{ a, b, c }))
//!     }
//!     pub fn parse(i: &'a [u8]) -> nom::IResult(&'a [u8], S) {
//!         S::parse_be(i)
//!     }
//! }
//! ```
//!
//! To parse input, just call `let res = S::parse_be(input);`.
//!
//! If the endianness of the struct is fixed (for ex. using the top-level `BigEndian` or
//! `LittleEndian` attributes, or the `NomBE` and `NomLE` custom derive), then the implementation
//! always uses this endianness, and all 3 functions are equivalent.
//!
//! For extensive documentation of all attributes and examples, see the documentation of [docs::Nom]
//! custom derive attribute.
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

pub mod docs;
mod helpers;
mod traits;

pub use helpers::*;
pub use traits::*;

pub use nom;
pub use nom_derive_impl::*;
