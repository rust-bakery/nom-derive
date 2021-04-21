## Upgrading to 0.8

### The Parse trait

The trait `Parse` has been introduced in 0.8.0. This trait is used to provide a common interface, and default implementations for primitive types.

As a consequence, it must be imported as well as the custom derive attribute:
`use nom_derive::Nom` now often becomes `use nom_derive::{Nom, Parse}` or `use nom_derive::*`.

### Order of attributes

`nom-derive` will now apply attributes in order of appearance to build a parse tree.

For example, if specifying both `Count` and `Cond` attributes:
- `Count="4", Cond(a > 0)` is built as `Count(4, Cond(a> 0, T::Parse))`, which means a type `Vec<Option<T>>` is expected
- `Cond(a > 0), Count="4"` is built as `Cond(a> 0, Count(4, T::Parse))`, which means a type `Option<Vec<T>>` is expected