<!-- cargo-sync-readme start -->

# nom-derive

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE-MIT)
[![Apache License 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](./LICENSE-APACHE)
[![docs.rs](https://docs.rs/nom-derive/badge.svg)](https://docs.rs/nom-derive)
[![Build Status](https://travis-ci.org/chifflier/nom-derive.svg?branch=master)](https://travis-ci.org/chifflier/nom-derive)
[![Crates.io Version](https://img.shields.io/crates/v/nom-derive.svg)](https://crates.io/crates/nom-derive)

## Overview

nom-derive is a custom derive attribute, to derive [nom] parsers automatically from the structure definition.

It is not meant to replace [nom], but to provide a quick and easy way to generate parsers for
structures, especially for simple structures. This crate aims at simplifying common cases.
In some cases, writing the parser manually will remain more efficient.

- [API documentation](https://docs.rs/nom-derive)
- [Documentation of `Nom` attribute](https://docs.rs/nom-derive/latest/nom_derive/derive.Nom.html). This is the main
  documentation for this crate, with all possible options and many examples.

*Feedback welcome !*

## `#[derive(Nom)]`

This crate exposes a single custom-derive macro `Nom` which
implements `parse` for the struct it is applied to.

The goal of this project is that:

* `derive(Nom)` should be enough for you to derive [nom] parsers for simple
  structures easily, without having to write it manually
* it allows overriding any parsing method by your own
* it allows using generated parsing functions along with handwritten parsers and
  combining them without efforts
* it remains as fast as nom

`nom-derive` adds declarative parsing to `nom`. It also allows mixing with
procedural parsing easily, making writing parsers for byte-encoded formats
very easy.

For example:

```rust
use nom_derive::Nom;

#[derive(Nom)]
struct S {
  a: u32,
  b: u16,
  c: u16
}
```

This adds a static method `parse` to `S`, with the following signature:
```rust,ignore
impl S {
    pub fn parse(i: &[u8]) -> nom::IResult(&[u8], S);
}
```

To parse input, just call `let res = S::parse(input);`.

For extensive documentation of all attributes and examples, see the [Nom derive
attribute](https://docs.rs/nom-derive/latest/nom_derive/derive.Nom.html) documentation.

Many examples are provided, and more can be found in the [project
tests](https://github.com/rust-bakery/nom-derive/tree/master/tests).

## Debug tips

* If the generated parser does not compile, add `#[nom(DebugDerive)]` to the structure.
  It will dump the generated parser to `stderr`.
* If the generated parser fails at runtime, try adding `#[nom(Debug)]` to the structure or
  to fields. It wraps subparsers in `dbg_dmp` and will print the field name and input to
  `stderr` if the parser fails.

[nom]: https://github.com/geal/nom
<!-- cargo-sync-readme end -->

## Changes

### <unreleased>

- Allow dynamic configuration of endianness (SetEndian attribute)
- Add support for `u128`/`i128` (#3)

### 0.6.1

- Add Tag attribute
- Fix type verification with Cond when using multiple attributes

### 0.6.0

- Switch to nom parsing functions, do not generate macros
- Use qualified paths, caller do not have to import nom macros
- Move all attributes under the 'nom' namespace
- Add many attributes (LittleEndian, BigEndian, Map, Debug, Value, Take,
  AlignAfter/AlignBefore, SkipAfter/SkipBefore, ErrorIf, etc.)
- Deprecate the `NomDeriveDebug` derive (replaced by `DebugDerive` attribute)
- Improve documentation, add many examples
- Rewrite attribute parser, now accepting a more flexible syntax

### 0.5.0

- Upgrade to nom 5.0
- The `parse` method is now public
- Upgrade dependencies (syn, quote, proc-macro2)

### 0.4.0

- Add support for `Enum` parser generator
  - Enums require a selector to choose the variant
  - Fieldless enums (list of constants) are handled as a special case
- Add `NomDeriveDebug` attribute to display generated parser on stderr during build

### 0.3.0

- Move crate to rust-bakery github project
- Add `Count` attribute

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
