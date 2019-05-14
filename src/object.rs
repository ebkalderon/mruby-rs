/// Type containing a dynamic mruby class instance with runtime reflection.
///
/// This type is useful for interacting with mruby classes that don't have an equivalent Rust class
/// to cast into. It implements the `Class`, `ToValue`, and `FromValue` traits, so it should be
/// pretty easy to work with. If you're trying to deserialize something and you're not sure what
/// static type to specify, just make it "Object" and it will allow you to query the name, parent
/// class name, methods, constants, instance variables, etc. at runtime.
#[derive(Debug)]
pub struct Object;
