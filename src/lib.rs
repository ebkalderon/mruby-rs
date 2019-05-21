pub use crate::value::Value;
pub use mruby_macros::Symbol;

use std::ffi::CString;

use mruby_sys::{self, mrb_state};

use crate::de::{CastError, Deserializer, FromValue};
use crate::ser::{Serializer, ToValue};

#[macro_use]
mod macros;

pub mod de;
pub mod ser;
pub mod symbol;

mod class;
mod module;
mod object;
mod value;

#[derive(Debug)]
pub enum Error {
    Init,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Mruby {
    state: *mut mrb_state,
}

impl Mruby {
    pub fn new() -> Result<Self, Error> {
        let state = unsafe { mruby_sys::mrb_open() };
        if !state.is_null() {
            Ok(Mruby { state })
        } else {
            Err(Error::Init)
        }
    }

    pub fn register_global<V: ToValue>(&mut self, name: &str, global: V) {
        use mruby_sys::{mrb_gv_set, mrb_intern_cstr};

        let ser = Serializer::new(self.state);
        let value = global.to_value(ser).into_inner();
        let owned = CString::new(name).expect("String contains null byte");

        unsafe {
            let sym = mrb_intern_cstr(self.state, owned.as_ptr());
            mrb_gv_set(self.state, sym, value);
        }
    }

    pub fn get_global<V: FromValue>(&mut self, name: &str) -> Result<V, CastError> {
        use mruby_sys::{mrb_gv_get, mrb_intern_cstr};

        let owned = CString::new(name).expect("String contains null byte");
        let value = unsafe {
            let sym = mrb_intern_cstr(self.state, owned.as_ptr());
            Value(mrb_gv_get(self.state, sym))
        };

        let de = Deserializer::new(self.state, value);
        V::from_value(de)
    }
}

impl Drop for Mruby {
    fn drop(&mut self) {
        unsafe {
            mruby_sys::mrb_close(self.state);
        }
    }
}

unsafe impl Send for Mruby {}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::ffi::CString;

    use super::*;

    #[derive(Debug, PartialEq, Symbol)]
    #[symbol(rename_all = "snake_case")]
    pub enum AllowedSymbols {
        Foo,
        BarBaz,
    }

    #[test]
    fn round_trip() {
        let mut ruby = Mruby::new().unwrap();

        let mut map = BTreeMap::new();
        map.insert("first", 16);
        map.insert("second", 17);
        map.insert("third", 18);

        let sym = AllowedSymbols::BarBaz;
        let input = (42, Some(sym), [1, 2, 3], 64.5f32, map, true);
        ruby.register_global("$example", &input);
        println!("  serialized (rust): {:?}", input);

        unsafe {
            let owned = CString::new(r#"puts "native value (ruby): #{$example}""#).unwrap();
            mruby_sys::mrb_load_string(ruby.state, owned.as_ptr());
        }

        let output = ruby.get_global("$example").expect("Failed to deserialize");
        println!("deserialized (rust): {:?}", output);
        assert_eq!(input, output);
    }
}
