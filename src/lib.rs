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
//! - [Documentation of `Nom` attribute](derive.Nom.html). This is the main
//!   documentation for this crate, with all possible options and many examples.
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

extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::*;

mod config;
mod enums;
mod meta;
mod parsertree;
mod structs;

use crate::meta::attr::{MetaAttr, MetaAttrType};
use enums::impl_nom_enums;
use structs::{get_pre_post_exec, parse_struct};

/// The `Nom` derive automatically generates a `parse` function for the structure
/// using [nom] parsers. It will try to infer parsers for primitive of known
/// types, but also allows you to specify parsers using custom attributes.
///
/// Deriving parsers supports `struct` and `enum` types.
///
/// Many examples are provided, and more can be found in the [project
/// tests](https://github.com/rust-bakery/nom-derive/tree/master/tests).
///
/// [nom]: https://github.com/Geal/nom
///
/// # Attributes
///
/// Derived parsers can be controlled using the `nom` attribute, with a sub-attribute.
/// For example, `#[nom(Value)]`.
///
/// Most combinators support using literal strings `#[nom(Count="4")]` or
/// parenthesized values `#[nom(Count(4))]`
///
/// To specify multiple attributes, use a comma-separated list: `#[nom(Debug, Count="4")]`.
///
/// The available attributes are:
///
/// | Attribute | Supports | Description
/// |-----------|------------------|------------
/// | [AlignAfter](#alignment-and-padding) | fields | skip bytes until aligned to a multiple of the provided value, after parsing value
/// | [AlignBefore](#alignment-and-padding) | fields | skip bytes until aligned to a multiple of the provided value, before parsing value
/// | [BigEndian](#byteorder) | all | Set the endianness to big endian
/// | [Cond](#conditional-values) | fields | Used on an `Option<T>` to read a value of type `T` only if the condition is met
/// | [Complete](#complete) | fields | Transforms Incomplete into Error
/// | [Count](#count) | fields | Set the expected number of items to parse
/// | [Debug](#debug) | all | Print error message and input if parser fails (at runtime)
/// | [DebugDerive](#debugderive) | top-level | Print the generated code to stderr during build
/// | [Default](#default) | fields | Do not parse, set a field to the default value for the type
/// | [ErrorIf](#verifications) | fields | Before parsing, check condition is true and return an error if false.
/// | [Exact](#exact) | top-level | Check that input was entirely consumed by parser
/// | [If](#conditional-values) | fields | Similar to `Cond`
/// | [Ignore](#default) | fields | An alias for `default`
/// | [InputName](#input-name) | top-level | Change the internal name of input
/// | [LittleEndian](#byteorder) | all | Set the endianness to little endian
/// | [Map](#map) | fields | Parse field, then apply a function
/// | [Move](#alignment-and-padding) | fields | add the specified offset to current position, before parsing
/// | [MoveAbs](#alignment-and-padding) | fields | go to the specified absoluted position, before parsing
/// | [Parse](#custom-parsers) | fields | Use a custom parser function for reading from a file
/// | [PreExec](#preexec) | all | Execute Rust code before parsing field or struct
/// | [PostExec](#postexec) | all | Execute Rust code after parsing field or struct
/// | [Selector](#deriving-parsers-for-enum) | all | Used to specify the value matching an enum variant
/// | [SetEndian](#byteorder) | all | Dynamically set the endianness
/// | [SkipAfter](#alignment-and-padding) | fields | skip the specified number of bytes, after parsing
/// | [SkipBefore](#alignment-and-padding) | fields | skip the specified number of bytes, before parsing
/// | [Tag](#tag) | fields | Parse a constant pattern
/// | [Take](#take) | fields | Take `n` bytes of input
/// | [Value](#value) | fields | Store result of evaluated expression in field
/// | [Verify](#verifications) | fields | After parsing, check that condition is true and return an error if false.
///
/// See below for examples.
///
/// # Deriving parsers for `Struct`
///
/// Import the `Nom` derive attribute:
///
/// ```rust
/// use nom_derive::Nom;
/// ```
/// and add it to structs or enums.
///
/// For simple structures, the parsers are automatically generated:
///
/// ```rust
/// # use nom_derive::Nom;
/// #
/// # #[derive(Debug,PartialEq)] // for assert_eq!
/// #[derive(Nom)]
/// struct S {
///   a: u32,
///   b: u16,
///   c: u16
/// }
///
/// # let input = b"\x00\x00\x00\x01\x12\x34\x56\x78";
/// # let res = S::parse(input);
/// # assert_eq!(res, Ok((&input[8..],S{a:1,b:0x1234,c:0x5678})));
/// ```
///
/// This also work for tuple structs:
///
/// ```rust
/// # use nom_derive::Nom;
/// #
/// # #[derive(Debug, PartialEq)] // for assert_eq!
/// #[derive(Nom)]
/// struct S(u32);
/// #
/// # let input = b"\x00\x00\x00\x01";
/// # let res = S::parse(input);
/// # assert_eq!(res, Ok((&input[4..],S(1))));
/// ```
///
/// ## Byteorder
///
/// By default, integers are parsed are big endian.
///
/// The `LittleEndian` attribute can be applied to a struct to change all integer parsers:
///
/// ```rust
/// # use nom_derive::Nom;
/// #
/// # #[derive(Debug, PartialEq)] // for assert_eq!
/// #[derive(Nom)]
/// #[nom(LittleEndian)]
/// struct LittleEndianStruct {
///   a: u32,
///   b: u16,
///   c: u16
/// }
///
/// let input = b"\x00\x00\x00\x01\x12\x34\x56\x78";
/// let res = LittleEndianStruct::parse(input);
/// assert_eq!(res, Ok((&input[8..],
///     LittleEndianStruct{a:0x0100_0000,b:0x3412,c:0x7856}))
/// );
/// ```
///
/// The `BigEndian` and `LittleEndian` attributes can be specified for struct fields.
/// If both per-struct and per-field attributes are present, the more specific wins.
///
/// For example, the all fields of the following struct will be parsed as big-endian,
/// except `b`:
///
/// ```rust
/// # use nom_derive::Nom;
/// #
/// # #[derive(Debug,PartialEq)] // for assert_eq!
/// #[derive(Nom)]
/// #[nom(BigEndian)]
/// struct MixedEndianStruct {
///   a: u32,
///   #[nom(LittleEndian)]
///   b: u16,
///   c: u16
/// }
///
/// # let input = b"\x00\x00\x00\x01\x12\x34\x56\x78";
/// # let res = MixedEndianStruct::parse(input);
/// # assert_eq!(res, Ok((&input[8..],
/// #     MixedEndianStruct{a:0x1,b:0x3412,c:0x5678}))
/// # );
/// ```
///
/// The `SetEndian` attribute changes the endianness of all following integer parsers to the
/// provided endianness (expected argument has type `nom::number::Endianness`). The expression
/// can be any expression or function returning an endianness, and will be evaluated once
/// at the location of the attribute.
///
/// Only the parsers after this attribute (including it) are affected: if `SetEndian` is applied to
/// the third field of a struct having 4 fields, only the fields 3 and 4 will have dynamic
/// endianness.
///
/// This allows dynamic (runtime) change of the endianness, at a small cost (a test is done before
/// every following integer parser).
/// However, if the argument is static or known at compilation, the compiler will remove the test
/// during optimization.
///
/// If a `BigEndian` or `LittleEndian` is applied to a field, its definition is used prior to
/// `SetEndian`.
///
/// For ex, to create a parse function having two arguments (`input`, and the endianness):
///
/// ```rust
/// # use nom_derive::Nom;
/// # use nom::number::Endianness;
/// #
/// # #[derive(Debug,PartialEq)] // for assert_eq!
/// #[derive(Nom)]
/// #[nom(ExtraArgs(endian: Endianness))]
/// #[nom(SetEndian(endian))] // Set dynamically the endianness
/// struct MixedEndianStruct {
///   a: u32,
///   b: u16,
///   #[nom(BigEndian)] // Field c will always be parsed as BigEndian
///   c: u16
/// }
///
/// # let input = b"\x00\x00\x00\x01\x12\x34\x56\x78";
/// let res = MixedEndianStruct::parse(input, Endianness::Big);
/// # assert_eq!(res, Ok((&input[8..],
/// #     MixedEndianStruct{a:0x1,b:0x1234,c:0x5678}))
/// # );
/// # let res = MixedEndianStruct::parse(input, Endianness::Little);
/// # assert_eq!(res, Ok((&input[8..],
/// #     MixedEndianStruct{a:0x0100_0000,b:0x3412,c:0x5678}))
/// # );
/// ```
///
/// # Deriving and Inferring Parsers
///
/// `nom-derive` is also able to infer parsers for some usual types: integers, `Option`, `Vec`, etc.
///
/// If the parser cannot be inferred, a default function will be called. It is also possible to
/// override this using the `Parse` attribute.
///
/// Following sections give more details.
///
/// ## Option types
///
/// If a field is an `Option<T>`, the generated parser is `opt(complete(T::parse))`
///
/// For ex:
/// ```rust
/// # use nom_derive::Nom;
/// #
/// # #[derive(Debug,PartialEq)] // for assert_eq!
/// #[derive(Nom)]
/// struct S {
///   a: Option<u32>
/// }
///
/// let input = b"\x00\x00\x00\x01";
/// let res = S::parse(input);
/// assert_eq!(res, Ok((&input[4..],S{a:Some(1)})));
/// ```
///
/// ## Vec types
///
/// If a field is an `Vec<T>`, the generated parser is `many0(complete(T::parse))`
///
/// For ex:
/// ```rust
/// # use nom_derive::Nom;
/// #
/// # #[derive(Debug,PartialEq)] // for assert_eq!
/// #[derive(Nom)]
/// struct S {
///   a: Vec<u16>
/// }
///
/// let input = b"\x00\x00\x00\x01";
/// let res = S::parse(input);
/// assert_eq!(res, Ok((&input[4..],S{a:vec![0,1]})));
/// ```
///
/// ## Count
///
/// The `Count(n)` attribute can be used to specify the number of items to parse.
///
/// Notes:
///   - the subparser is inferred as usual (item type must be `Vec< ... >`)
///   - the number of items (`n`) can be any expression, and will be cast to `usize`
///
/// For ex:
/// ```rust
/// # use nom_derive::Nom;
/// #
/// # #[derive(Debug,PartialEq)] // for assert_eq!
/// #[derive(Nom)]
/// struct S {
///   a: u16,
///   #[nom(Count="a")]
///   b: Vec<u16>
/// }
/// #
/// # let input = b"\x00\x01\x12\x34";
/// # let res = S::parse(input);
/// # assert_eq!(res, Ok((&input[4..],S{a:1, b:vec![0x1234]})));
/// ```
///
/// ## Tag
///
/// The `Tag(value)` attribute is used to parse a constant value (or "magic").
///
/// For ex:
/// ```rust
/// # use nom_derive::Nom;
/// #
/// # #[derive(Debug,PartialEq)] // for assert_eq!
/// #[derive(Nom)]
/// struct S<'a> {
///   #[nom(Tag(b"TAG"))]
///   tag: &'a[u8],
///   a: u16,
///   b: u16,
/// }
/// #
/// # let input = b"TAG\x00\x01\x12\x34";
/// # let res = S::parse(input);
/// # assert_eq!(res, Ok((&input[7..],S{tag: b"TAG", a:1, b:0x1234})));
/// ```
///
/// ## Take
///
/// The `Take="n"` attribute can be used to take `n` bytes of input.
///
/// Notes:
///   - the number of items (`n`) can be any expression, and will be cast to `usize`
///
/// For ex:
/// ```rust
/// # use nom_derive::Nom;
/// #
/// # #[derive(Debug,PartialEq)] // for assert_eq!
/// #[derive(Nom)]
/// struct S<'a> {
///   a: u16,
///   #[nom(Take="1")]
///   b: &'a [u8],
/// }
/// #
/// # let input = b"\x00\x01\x12\x34";
/// # let res = S::parse(input);
/// # assert_eq!(res, Ok((&input[3..],S{a:1, b:&[0x12]})));
/// ```
///
/// ## Default parsing function
///
/// If a field with type `T` is not a primitive or known type, the generated parser is
/// `T::parse(input)`.
///
/// This function can be automatically derived, or specified as a method for the struct.
/// In that case, the function must be a static method with the same API as a
/// [nom] combinator, returning the wrapped struct when parsing succeeds.
///
/// For example (using `Nom` derive):
/// ```rust
/// # use nom_derive::Nom;
/// #
/// # #[derive(Debug,PartialEq)] // for assert_eq!
/// #[derive(Nom)]
/// struct S2 {
///   c: u16
/// }
///
/// # #[derive(Debug,PartialEq)] // for assert_eq!
/// #[derive(Nom)]
/// struct S {
///   a: u16,
///   b: S2
/// }
/// #
/// # let input = b"\x00\x00\x00\x01";
/// # let res = S::parse(input);
/// # assert_eq!(res, Ok((&input[4..],S{a:0,b:S2{c:1}})));
/// ```
///
/// Example (defining `parse` method):
/// ```rust
/// # use nom_derive::Nom;
/// # use nom::{IResult,call,map};
/// # use nom::number::streaming::le_u16;
/// #
/// # #[derive(Debug,PartialEq)] // for assert_eq!
/// // no Nom derive
/// struct S2 {
///   c: u16
/// }
///
/// impl S2 {
///     fn parse(i:&[u8]) -> IResult<&[u8],S2> {
///         map!(
///             i,
///             le_u16, // little-endian
///             |c| S2{c} // return a struct S2
///         )
///     }
/// }
///
/// # #[derive(Debug,PartialEq)] // for assert_eq!
/// #[derive(Nom)]
/// struct S {
///   a: u16,
///   b: S2
/// }
/// #
/// # let input = b"\x00\x00\x00\x01";
/// # let res = S::parse(input);
/// # assert_eq!(res, Ok((&input[4..],S{a:0,b:S2{c:256}})));
/// ```
///
/// ## Custom parsers
///
/// Sometimes, the default parsers generated automatically are not those you
/// want.
///
/// The `Parse` custom attribute allows for specifying the parser that
/// will be inserted in the nom parser.
///
/// The parser is called with input as argument, so the signature of the parser
/// must be equivalent to:
///
/// ```rust,ignore
/// fn parser(i: &[u8]) -> IResult<T> {
/// // ...
/// }
/// ```
///
/// For example, to specify the parser of a field:
///
/// ```rust
/// # use nom_derive::Nom;
/// # use nom::number::streaming::le_u16;
/// #
/// # #[derive(Debug,PartialEq)] // for assert_eq!
/// #[derive(Nom)]
/// struct S{
///     #[nom(Parse="le_u16")]
///     a: u16
/// }
/// #
/// # let input = b"\x00\x01";
/// # let res = S::parse(input);
/// # assert_eq!(res, Ok((&input[2..],S{a:256})));
/// ```
///
/// The `Parse` argument can be a complex expression:
/// ```rust
/// # use nom_derive::Nom;
/// # use nom::combinator::cond;
/// # use nom::number::streaming::be_u16;
/// #
/// # #[derive(Debug,PartialEq)] // for assert_eq!
/// #[derive(Nom)]
/// struct S{
///     pub a: u8,
///     #[nom(Parse="cond(a > 0,be_u16)")]
///     pub b: Option<u16>,
/// }
/// #
/// # let input = b"\x01\x00\x01";
/// # let res = S::parse(input);
/// # assert_eq!(res, Ok((&input[3..],S{a:1,b:Some(1)})));
/// ```
/// Note that you are responsible from providing correct code.
///
/// ## Default
///
/// If a field is marked as `Ignore` (or `Default`), it will not be parsed.
/// Its value will be the default value for the field type.
///
/// This is convenient if the structured has more fields than the serialized value.
///
/// ```rust
/// # use nom_derive::Nom;
/// #
/// # #[derive(Debug,PartialEq)] // for assert_eq!
/// #[derive(Nom)]
/// struct S{
///     pub a: u8,
///     #[nom(Ignore)]
///     pub b: Option<u16>,
/// }
/// #
/// # let input = b"\x01\x00\x01";
/// # let res = S::parse(input);
/// # assert_eq!(res, Ok((&input[1..],S{a:1,b:None})));
/// ```
///
/// ## Complete
///
/// The `Complete` attribute transforms Incomplete into Error.
///
/// Default is to use streaming parsers.
///
/// ```rust
/// # use nom_derive::Nom;
/// # use nom::number::streaming::be_u8;
/// #
/// # #[derive(Debug,PartialEq)] // for assert_eq!
/// #[derive(Nom)]
/// struct S{
///     pub a: u8,
///     #[nom(Complete)]
///     pub b: u64,
/// }
/// #
/// # let input = b"\x01\x00\x01";
/// # let res = S::parse(input).expect_err("parse error");
/// # assert!(!res.is_incomplete());
/// ```
///
/// ## Map
///
/// The `Map` attribute can be used to apply a function to the result
/// of the parser.
/// It is often used combined with the `Parse` attribute.
///
/// ```rust
/// # use nom_derive::Nom;
/// # use nom::number::streaming::be_u8;
/// #
/// # #[derive(Debug,PartialEq)] // for assert_eq!
/// #[derive(Nom)]
/// struct S{
///     pub a: u8,
///     #[nom(Parse="be_u8", Map = "|x: u8| x.to_string()")]
///     pub b: String,
/// }
/// #
/// # let input = b"\x01\x00\x01";
/// # let res = S::parse(input);
/// # assert_eq!(res, Ok((&input[2..],S{a:1,b:"0".to_string()})));
/// ```
///
/// ## Conditional Values
///
/// The `Cond` custom attribute allows for specifying a condition.
/// The generated parser will use the `cond!` combinator, which calls the
/// child parser only if the condition is met.
/// The type with this attribute must be an `Option` type.
///
/// ```rust
/// # use nom_derive::Nom;
/// #
/// # #[derive(Debug,PartialEq)] // for assert_eq!
/// #[derive(Nom)]
/// struct S{
///     pub a: u8,
///     #[nom(Cond="a == 1")]
///     pub b: Option<u16>,
/// }
/// #
/// # let input = b"\x01\x00\x01";
/// # let res = S::parse(input);
/// # assert_eq!(res, Ok((&input[3..],S{a:1,b:Some(1)})));
/// ```
///
/// ## Value
///
/// The `Value` attribute does not parse data. It is used to store the result
/// of the evaluated expression in the variable.
///
/// Previous fields can be used in the expression.
///
/// ```rust
/// # use nom_derive::Nom;
/// # use nom::number::streaming::be_u8;
/// #
/// # #[derive(Debug,PartialEq)] // for assert_eq!
/// #[derive(Nom)]
/// struct S{
///     pub a: u8,
///     #[nom(Value = "a.to_string()")]
///     pub b: String,
/// }
/// #
/// # let input = b"\x01\x00\x01";
/// # let res = S::parse(input);
/// # assert_eq!(res, Ok((&input[1..],S{a:1,b:"1".to_string()})));
/// ```
///
/// ## Verifications
///
/// The `Verify` custom attribute allows for specifying a verifying function.
/// The generated parser will use the `verify` combinator, which calls the
/// child parser only if is verifies a condition (and otherwise raises an error).
///
/// The argument used in verify function is passed as a reference.
///
/// ```rust
/// # use nom_derive::Nom;
/// #
/// # #[derive(Debug,PartialEq)] // for assert_eq!
/// #[derive(Nom)]
/// struct S{
///     #[nom(Verify="*a == 1")]
///     pub a: u8,
/// }
/// #
/// # let input = b"\x01";
/// # let res = S::parse(input);
/// # assert_eq!(res, Ok((&input[1..],S{a:1})));
/// ```
///
/// The `ErrorIf` checks the provided condition, and return an error if the
/// test returns false.
/// The condition is tested before any parsing occurs for this field, and does not
/// change the input pointer.
///
/// Error has type `ErrorKind::Verify` (nom).
///
/// The argument used in verify function is passed as a reference.
///
/// ```rust
/// # use nom_derive::Nom;
/// #
/// # #[derive(Debug,PartialEq)] // for assert_eq!
/// #[derive(Nom)]
/// struct S{
///     pub a: u8,
///     #[nom(ErrorIf(a != 1))]
///     pub b: u8,
/// }
/// #
/// # let input = b"\x01\x02";
/// # let res = S::parse(input);
/// # assert_eq!(res, Ok((&input[2..],S{a:1, b:2})));
/// ```
///
/// ## Exact
///
/// The `Exact` custom attribute adds a verification after parsing the entire element.
/// It succeeds if the input has been entirely consumed by the parser.
///
/// ```rust
/// # use nom_derive::Nom;
/// #
/// # #[derive(Debug,PartialEq)] // for assert_eq!
/// #[derive(Nom)]
/// #[nom(Exact)]
/// struct S{
///     pub a: u8,
/// }
/// #
/// # let input = b"\x01\x01";
/// # let res = S::parse(&input[1..]);
/// # assert!(res.is_ok());
/// # let res = S::parse(input);
/// # assert!(res.is_err());
/// ```
///
/// ## PreExec
///
/// The `PreExec` custom attribute executes the provided code before parsing
/// the field or structure.
///
/// This attribute can be specified multiple times. Statements will be executed in order.
///
/// Note that the current input can be accessed, as a regular variable (see [InputName](#input-name)).
/// If you create a new variable with the same name, it will be used as input (resulting in
/// side-effects).
///
/// Expected value: a valid Rust statement
///
/// ```rust
/// # use nom_derive::Nom;
/// #
/// # #[derive(Debug,PartialEq)] // for assert_eq!
/// #[derive(Nom)]
/// struct S{
///     #[nom(PreExec="let sz = i.len();")]
///     pub a: u8,
///     #[nom(Value(sz))]
///     pub sz: usize,
/// }
/// #
/// # let input = b"\x01";
/// # let res = S::parse(input);
/// # assert_eq!(res, Ok((&input[1..],S{a:1, sz:1})));
/// ```
///
/// ## PostExec
///
/// The `PostExec` custom attribute executes the provided code after parsing
/// the field or structure.
///
/// This attribute can be specified multiple times. Statements will be executed in order.
///
/// Note that the current input can be accessed, as a regular variable (see [InputName](#input-name)).
/// If you create a new variable with the same name, it will be used as input (resulting in
/// side-effects).
///
/// Expected value: a valid Rust statement
///
/// ```rust
/// # use nom_derive::Nom;
/// #
/// # #[derive(Debug,PartialEq)] // for assert_eq!
/// #[derive(Nom)]
/// struct S{
///     #[nom(PostExec="let b = a + 1;")]
///     pub a: u8,
///     #[nom(Value(b))]
///     pub b: u8,
/// }
/// #
/// # let input = b"\x01";
/// # let res = S::parse(input);
/// # assert_eq!(res, Ok((&input[1..],S{a:1, b:2})));
/// ```
///
/// If applied to the top-level element, the statement is executing after the entire element
/// is parsed.
///
/// If parsing a structure, the built structure is available in the `struct_def` variable.
///
/// If parsing an enum, the built structure is available in the `enum_def` variable.
///
/// ```rust
/// # use nom_derive::Nom;
/// #
/// # #[derive(PartialEq)] // for assert_eq!
/// #[derive(Debug)]
/// #[derive(Nom)]
/// #[nom(PostExec(println!("parsing done: {:?}", struct_def);))]
/// struct S{
///     pub a: u8,
///     pub b: u8,
/// }
/// #
/// # let input = b"\x01\x02";
/// # let res = S::parse(input);
/// # assert_eq!(res, Ok((&input[2..],S{a:1, b:2})));
/// ```
///
/// ## Alignment and Padding
///
///  - `AlignAfter`/`AlignBefore`: skip bytes until aligned to a multiple of the provided value
///    Alignment is calculated to the start of the original parser input
///  - `SkipAfter`/`SkipBefore`: skip the specified number of bytes
///  - `Move`: add the speficied offset to current position, before parsing. Offset can be negative.
///  - `MoveAbs`: go to specified absolute position (relative to the start of original parser
///     input), before parsing
///
///  If multiple directives are provided, they are applied in order of appearance of the
///  attribute.
///
///  If the new position would be before the start of the slice or beyond its end,
///  an error is raised (`TooLarge` or `Incomplete`, depending on the case).
///
/// Expected value: a valid Rust value (immediate value, or expression)
///
/// ```rust
/// # use nom_derive::Nom;
/// #
/// # #[derive(Debug,PartialEq)] // for assert_eq!
/// #[derive(Nom)]
/// struct S{
///     pub a: u8,
///     #[nom(AlignBefore(4))]
///     pub b: u8,
/// }
/// #
/// # let input = b"\x01\x00\x00\x00\x02";
/// # let res = S::parse(input);
/// # assert_eq!(res, Ok((&input[5..],S{a:1, b:2})));
/// ```
///
/// # Deriving parsers for `Enum`
///
/// The `Nom` attribute can also used to generate parser for `Enum` types.
/// The generated parser will used a value (called *selector*) to determine
/// which attribute variant is parsed.
/// Named and unnamed enums are supported.
///
/// In addition of `derive(Nom)`, a `Selector` attribute must be used:
///   - on the structure, to specify the type of selector to match
///   - on each variant, to specify the value associated with this variant.
///
/// Expected values:
///   - top-level: a valid Rust type
///   - fields: a valid Rust match arm expression (for ex: `0`). *Note*: this expression can
///     contain a pattern guard (for ex: `x if x > 2`)
///
/// ```rust
/// # use nom_derive::Nom;
/// #
/// # #[derive(Debug,PartialEq)] // for assert_eq!
/// #[derive(Nom)]
/// #[nom(Selector="u8")]
/// pub enum U1{
///     #[nom(Selector="0")] Field1(u32),
///     #[nom(Selector="1")] Field2(Option<u32>),
/// }
/// #
/// # let input = b"\x00\x00\x00\x02";
/// # let res = U1::parse(input, 0);
/// # assert_eq!(res, Ok((&input[4..],U1::Field1(2))));
/// ```
///
/// The generated function will look like:
///
/// <pre>
/// impl U1{
///     pub fn parse(i:&[u8], selector: u8) -> IResult<&[u8],U1> {
///         match selector {
///             ...
///         }
///     }
/// }
/// </pre>
///
/// It can be called either directly (`U1::parse(n)`) or using nom
/// (`call!(U1::parse,n)`).
///
/// The selector can be a primitive type (`u8`), or any other type implementing the `PartialEq`
/// trait.
///
/// ```rust
/// # use nom_derive::Nom;
/// #
/// #[derive(Debug,PartialEq,Eq,Clone,Copy,Nom)]
/// pub struct MessageType(pub u8);
///
/// # #[derive(Debug,PartialEq)] // for assert_eq!
/// #[derive(Nom)]
/// #[nom(Selector="MessageType")]
/// pub enum U1{
///     #[nom(Selector="MessageType(0)")] Field1(u32),
///     #[nom(Selector="MessageType(1)")] Field2(Option<u32>),
/// }
///
/// // Example of call from a struct:
/// #[derive(Nom)]
/// pub struct S1{
///     pub msg_type: MessageType,
///     #[nom(Parse="{ |i| U1::parse(i, msg_type) }")]
///     pub msg_value: U1
/// }
/// #
/// # let input = b"\x00\x00\x00\x02";
/// # let res = U1::parse(input, MessageType(0));
/// # assert_eq!(res, Ok((&input[4..],U1::Field1(2))));
/// ```
///
/// ## Default case
///
/// By default, if no value of the selector matches the input value, a nom error
/// `ErrorKind::Switch` is raised. This can be changed by using `_` as selector
/// value for one the variants.
///
/// ```rust
/// # use nom_derive::Nom;
/// #
/// # #[derive(Debug,PartialEq)] // for assert_eq!
/// #[derive(Nom)]
/// #[nom(Selector="u8")]
/// pub enum U2{
///     #[nom(Selector="0")] Field1(u32),
///     #[nom(Selector="_")] Field2(u32),
/// }
/// #
/// # let input = b"\x00\x00\x00\x02";
/// # let res = U2::parse(input, 123);
/// # assert_eq!(res, Ok((&input[4..],U2::Field2(2))));
/// ```
///
/// If the `_` selector is not the last variant, the generated code will use it
/// as the last match to avoid unreachable code.
///
/// ## Special case: specifying parsers for fields
///
/// Sometimes, an unnamed field requires a custom parser. In that case, the
/// *field* (not the variant) must be annotated with attribute `Parse`.
///
/// Named fields:
///
/// ```rust
/// # use nom_derive::Nom;
/// # use nom::bytes::streaming::take;
/// #
/// # #[derive(Debug,PartialEq,Eq,Clone,Copy,Nom)]
/// # pub struct MessageType(pub u8);
/// #
/// #[derive(Nom)]
/// #[nom(Selector="MessageType")]
/// pub enum U3<'a>{
///     #[nom(Selector="MessageType(0)")] Field1{a:u32},
///     #[nom(Selector="MessageType(1)")] Field2{
///         #[nom(Parse="take(4 as usize)")]
///         a: &'a[u8]
///     },
/// }
/// ```
///
/// Unnamed fields:
///
/// ```rust
/// # use nom_derive::Nom;
/// # use nom::bytes::streaming::take;
/// #
/// # #[derive(Debug,PartialEq,Eq,Clone,Copy,Nom)]
/// # pub struct MessageType(pub u8);
/// #
/// #[derive(Nom)]
/// #[nom(Selector="MessageType")]
/// pub enum U3<'a>{
///     #[nom(Selector="MessageType(0)")] Field1(u32),
///     #[nom(Selector="MessageType(1)")] Field2(
///         #[nom(Parse="take(4 as usize)")] &'a[u8]
///     ),
/// }
/// ```
///
/// ## Special case: fieldless enums
///
/// If the entire enum is fieldless (a list of constant integer values), a
/// parser can be derived if
///   - the `Enum` has a `repr(ty)` attribute, with `ty` an integer type
///   - the `Enum` implements the `Eq` trait
///
/// In that case, the `Selector` attribute must *not* be specified.
///
/// ```rust
/// # use nom_derive::Nom;
/// # use nom::*;
/// # use nom::number::streaming::be_u8;
/// #
/// # #[derive(Debug,PartialEq)] // for assert_eq!
/// #[repr(u8)]
/// #[derive(Eq,Nom)]
/// pub enum U3{
///     A,
///     B = 2,
///     C
/// }
/// #
/// # let empty : &[u8] = b"";
/// # assert_eq!(
/// #     U3::parse(b"\x00"),
/// #     Ok((empty,U3::A))
/// # );
/// # assert!(
/// #     U3::parse(b"\x01").is_err()
/// # );
/// # assert_eq!(
/// #     U3::parse(b"\x02"),
/// #     Ok((empty,U3::B))
/// # );
/// ```
///
/// The generated parser will parse an element of type `ty` (as Big Endian), try
/// to match to enum values, and return an instance of `Enum` if it succeeds
/// (wrapped in an `IResult`).
///
/// For ex, `U3::parse(b"\x02")` will return `Ok((&b""[..],U3::B))`.
///
/// ## Input Name
///
/// Internally, the parser will use a variable to follow the input.
/// By default, this variable is named `i`.
///
/// This can cause problems, for example, if one field of the structure has the same name
///
/// The internal variable name can be renamed using the `InputName` top-level attribute.
///
/// ```rust
/// # use nom_derive::Nom;
/// #
/// # #[derive(Debug,PartialEq)] // for assert_eq!
/// #[derive(Nom)]
/// #[nom(InputName(aaa))]
/// pub struct S {
///     pub i: u8,
/// }
/// #
/// # let empty : &[u8] = b"";
/// # assert_eq!(
/// #     S::parse(b"\x00"),
/// #     Ok((empty, S{i:0}))
/// # );
/// ```
///
/// Note that this variable can be used as usual, for ex. to peek data
/// without advancing in the current stream, determining the length of
/// remaining bytes, etc.
///
/// ```rust
/// # use nom_derive::Nom;
/// #
/// # #[derive(Debug,PartialEq)] // for assert_eq!
/// #[derive(Nom)]
/// #[nom(InputName(i))]
/// pub struct S {
///     pub a: u8,
///     #[nom(Value(i.len()))]
///     pub remaining_len: usize,
/// }
/// #
/// # let empty : &[u8] = b"";
/// # assert_eq!(
/// #     S::parse(b"\x00"),
/// #     Ok((empty, S{a:0, remaining_len:0}))
/// # );
/// ```
///
/// **This can create side-effects**: if you create a variable with the same name
/// as the input, it will shadow it. While this will is generally an error, it can
/// sometimes be useful.
///
/// For example, to skip 2 bytes of input:
///
/// ```rust
/// # use nom_derive::Nom;
/// #
/// # #[derive(Debug,PartialEq)] // for assert_eq!
/// #[derive(Nom)]
/// #[nom(InputName(i))]
/// pub struct S {
///     pub a: u8,
///     // skip 2 bytes
///     // XXX this will panic if input is smaller than 2 bytes at this points
///     #[nom(PreExec(let i = &i[2..];))]
///     pub b: u8,
/// }
/// #
/// # let empty : &[u8] = b"";
/// # assert_eq!(
/// #     S::parse(b"\x00\x01\x02\x03"),
/// #     Ok((empty, S{a:0, b:3}))
/// # );
/// ```
///
/// ## Debug
///
/// Errors in generated parsers may be hard to understand and debug.
///
/// The `Debug` attribute insert calls to nom's `dbg_dmp` function, which will print
/// an error message and the input if the parser fails. This attribute can be applied to either
/// fields, or at top-level (all sub-parsers will be wrapped).
///
/// This helps resolving parse errors (at runtime).
///
/// ```rust
/// # use nom_derive::Nom;
/// #
/// #[derive(Nom)]
/// pub struct S {
///     pub a: u32,
///     #[nom(Debug)]
///     pub b: u64,
/// }
/// ```
///
/// ## DebugDerive
///
/// The `DebugDerive` attribute, if applied to top-level, makes the generator print the
/// generated code to `stderr`.
///
/// This helps resolving compiler errors.
///
/// ```rust
/// # use nom_derive::Nom;
/// #
/// #[derive(Nom)]
/// #[nom(DebugDerive)]
/// pub struct S {
///     pub a: u32,
/// }
/// ```
#[proc_macro_derive(Nom, attributes(nom))]
pub fn nom(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let ast = parse_macro_input!(input as DeriveInput);

    // Build and return the generated impl
    impl_nom(&ast, false)
}

fn get_extra_args(meta_list: &[MetaAttr]) -> Option<&proc_macro2::TokenStream> {
    meta_list
        .iter()
        .find(|m| m.attr_type == MetaAttrType::ExtraArgs)
        .and_then(|m| m.arg())
}

fn impl_nom(ast: &syn::DeriveInput, debug_derive: bool) -> TokenStream {
    use crate::config::Config;
    // eprintln!("ast: {:#?}", ast);
    let struct_name = ast.ident.to_string();
    // parse top-level attributes and prepare tokens for each field parser
    let meta = meta::parse_nom_top_level_attribute(&ast.attrs)
        .expect("Parsing the 'nom' top level attribute failed");
    // eprintln!("top-level meta: {:?}", meta);
    let mut config = Config::from_meta_list(struct_name, &meta).expect("Could not build config");
    config.debug_derive |= debug_derive;
    let (tl_pre, tl_post) = get_pre_post_exec(&meta, &config);
    // is endianness dynamic ?
    if meta.iter().any(|m| m.attr_type == MetaAttrType::SetEndian) {
        config.test_endian = true;
    }
    // enums are handled differently
    let s = match &ast.data {
        syn::Data::Enum(_) => {
            return impl_nom_enums(ast, &mut config);
        }
        syn::Data::Struct(ref s) => parse_struct(s, &mut config),
        syn::Data::Union(_) => panic!("Unions not supported"),
    };
    // prepare tokens
    let generics = &ast.generics;
    let name = &ast.ident;
    let (idents, parser_tokens): (Vec<_>, Vec<_>) = s
        .parsers
        .iter()
        .map(|sp| {
            let id = syn::Ident::new(&sp.name, Span::call_site());
            (id, &sp.parser)
        })
        .unzip();
    let (pre, post): (Vec<_>, Vec<_>) = s
        .parsers
        .iter()
        .map(|sp| (sp.pre_exec.as_ref(), sp.post_exec.as_ref()))
        .unzip();
    let idents2 = idents.clone();
    // Code generation
    let struct_def = match (s.empty, s.unnamed) {
        (true, _) => quote! { ( #name ) },
        (_, true) => quote! { ( #name ( #(#idents2),* ) ) },
        (_, false) => quote! { ( #name { #(#idents2),* } ) },
    };
    let input_name = syn::Ident::new(&config.input_name, Span::call_site());
    let orig_input_name = syn::Ident::new(
        &("orig_".to_string() + &config.input_name),
        Span::call_site(),
    );
    let extra_args = get_extra_args(&meta);
    let tokens = quote! {
        impl#generics #name#generics {
            pub fn parse(#orig_input_name: &[u8] #extra_args) -> nom::IResult<&[u8],#name> {
                let #input_name = #orig_input_name;
                #tl_pre
                #(#pre let (#input_name, #idents) = #parser_tokens (#input_name) ?; #post)*
                let struct_def = #struct_def;
                #tl_post
                Ok((#input_name, struct_def))
            }
        }
    };
    if config.debug_derive {
        eprintln!("tokens:\n{}", tokens);
    }
    tokens.into()
}

/// This derive macro behaves exactly like [Nom derive](derive.Nom.html), except it
/// prints the generated parser on stderr.
/// This is helpful for debugging generated parsers.
#[deprecated(
    since = "0.6.0",
    note = "Please use the nom(DebugDerive) attribute instead"
)]
#[proc_macro_derive(NomDeriveDebug, attributes(nom))]
pub fn nom_derive_debug(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let ast = parse_macro_input!(input as DeriveInput);

    // Build and return the generated impl
    impl_nom(&ast, true)
}
