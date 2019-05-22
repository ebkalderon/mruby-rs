#![forbid(unsafe_code)]
#![recursion_limit = "128"]

extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod symbol;

/// Custom `#[derive]` macro for defining strongly-typed Ruby symbols.
///
/// This derive macro takes the enum variants (which must not have payloads nor type parameters)
/// and implements `FromValue` and `ToValue` on them, along with two utility traits called
/// `FromSymbol` and `ToSymbol`.
///
/// # Example
///
///
/// ```rust
/// #[derive(Symbol)]
/// pub enum Hello {
///     Foo,
///     BarBaz,
/// }
/// ```
///
/// # Attributes
///
/// The `Symbol` procedural macro provides an optional attribute called `rename` which customizes
/// how the symbols should be serialized and deserialized:
///
/// ```rust
/// #[derive(Symbol)]
/// #[symbol(rename_all = "snake_case")]
/// pub enum Hello {
///     Foo,    // :foo
///     BarBaz, // :bar_baz
/// }
/// ```
///
/// Possible values for `rename_all` are identical to [`#[serde(rename_all = "...")]`][rename_all],
/// with the addition of two extra (weird) values:
///
///  * `"lowercase spaced"`
///  * `"UPPERCASE SPACED"`
///
/// Although heavily discouraged, it is technically possible to create symbols in Ruby which
/// contain spaces by surrounding them with quotes. These two extra values allow symbols like these
/// to be expressed, if desired.
///
/// If the `rename` attribute is not present, `"snake_case"` is selected by default, adhering to
/// idiomatic Ruby style.
///
/// [rename_all]: https://serde.rs/container-attrs.html#rename_all
///
/// Additionally, there is a per-field `#[symbol(rename = "...")]` attribute which allows you to
/// manually rename the given field to an arbitrary string.
#[proc_macro_derive(Symbol, attributes(symbol))]
pub fn derive_symbol(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    symbol::derive_symbol(&ast)
        .unwrap_or_else(|e| e.write_errors())
        .into()
}

/// Defines a new Ruby class with the given PascalCase name and fields.
///
/// # Design
///
/// ```rust
/// /// All fields must implement `ToValue`/`FromValue` in order to work, unless skipped,
/// /// serde-style.
/// #[ruby::class]
/// pub struct Person {
///     name: String,
/// }
///
/// #[ruby::methods]
/// impl Person {
///     /// Defines the `initialize()` method for the class.
///     ///
///     /// 1. All parameters, if any, must implement `FromValue`, or must be at least `T: Into<U>`
///     ///    where `U: FromValue`.
///     /// 2. The return value must be `Self` or `Result<Self, T: Error>`. In the case of the
///     ///    constructor returning an `Err`, the error will be converted into a Ruby
///     ///    `RuntimeError` and the message will contain the Error's `Display` implementation. If
///     ///    a more specific Ruby exception type is desired, you can always select one of the
///     ///    built-in Ruby exceptions included with the `mruby` crate and make that be your `Err`.
///     ///
///     /// The Ruby constructor doesn't have to be `pub`. This is because the restrictions above
///     /// might not be desirable for use in a Rust API. Therefore, it might be beneficial to have
///     /// a `pub` constructor used within Rust and a private `initialize` constructor which is
///     /// more ergonomic to call from Ruby.
///     #[initialize]
///     pub fn new<T>(first_name: String, last_name: T) -> Self
///     where
///         T: Into<Option<String>>,
///     {
///         Person {
///             name: format!("{} {}", first_name, last_name.into().unwrap_or_default()),
///         }
///     }
///
///     /// Defines a Ruby method for the class.
///     ///
///     /// 1. Methods must require `&self` or `&mut self` or other variation.
///     /// 2. All parameters, if any, must implement `FromValue`, or must be at least `T: Into<U>`
///     ///    where `U: FromValue`.
///     /// 3. The return value must implement `ToValue`, or must at least be `T: Into<U>` where
///     ///    `U: ToValue`.
///     pub fn say_hello(&self) -> io::Result<()> {
///         io::stdout().write(b"Hello/")?;
///         Ok(())
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn mruby_class(_: TokenStream, _: TokenStream) -> TokenStream {
    unimplemented!()
}
