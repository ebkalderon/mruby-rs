//! ```rust
//! /// Defines a new Ruby class with the given PascalCase name and fields.
//! ///
//! /// All fields must implement `ToValue`/`FromValue` in order to work, unless skipped,
//! /// serde-style.
//! #[ruby::class]
//! pub struct Person {
//!     name: String,
//! }
//!
//! #[ruby::methods]
//! impl Person {
//!     /// Defines the `initialize()` method for the class.
//!     ///
//!     /// 1. All parameters, if any, must implement `FromValue`, or must be at least `T: Into<U>`
//!     ///    where `U: FromValue`.
//!     /// 2. The return value must be `Self` or `Result<Self, T: Error>`. In the case of the
//!     ///    constructor returning an `Err`, the error will be converted into a Ruby
//!     ///    `RuntimeError` and the message will contain the Error's `Display` implementation. If
//!     ///    a more specific Ruby exception type is desired, you can always select one of the
//!     ///    built-in Ruby exceptions included with the `mruby` crate and make that be your `Err`.
//!     ///
//!     /// The Ruby constructor doesn't have to be `pub`. This is because the restrictions above
//!     /// might not be desirable for use in a Rust API. Therefore, it might be beneficial to have
//!     /// a `pub` constructor used within Rust and a private `initialize` constructor which is
//!     /// more ergonomic to call from Ruby.
//!     #[initialize]
//!     pub fn new<T>(first_name: String, last_name: T) -> Self
//!     where
//!         T: Into<Option<String>>,
//!     {
//!         Person {
//!             name: format!("{} {}", first_name, last_name.into().unwrap_or_default()),
//!         }
//!     }
//!
//!     /// Defines a Ruby method for the class.
//!     ///
//!     /// 1. Methods must require `&self` or `&mut self` or other variation.
//!     /// 2. All parameters, if any, must implement `FromValue`, or must be at least `T: Into<U>`
//!     ///    where `U: FromValue`.
//!     /// 3. The return value must implement `ToValue`, or must at least be `T: Into<U>` where
//!     ///    `U: ToValue`.
//!     pub fn say_hello(&self) -> io::Result<()> {
//!         io::stdout().write(b"Hello!")?;
//!         Ok(())
//!     }
//! }
//! ```
