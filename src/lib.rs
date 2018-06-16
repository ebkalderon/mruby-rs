extern crate mruby_sys;

use mruby_sys::mrb_state;

use value::{ToValue, Value};

mod class;
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
        let state = unsafe {
            let state = mruby_sys::mrb_open();
            if state.is_null() {
                return Err(Error::Init);
            } else {
                state
            }
        };

        Ok(Mruby { state })
    }

    pub fn register_global<V: ToValue>(&mut self, name: &str, global: V) {
        use std::ffi::CString;
        use mruby_sys::{mrb_gv_set, mrb_intern_cstr};

        let mut state = value::State::new(self.state);
        let Value(val) = global.to_value(&mut state);

        unsafe {
            let owned = CString::new(name).expect("Unterminated string");
            let sym = mrb_intern_cstr(self.state, owned.as_ptr());
            mrb_gv_set(self.state, sym, val);
        }
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
    use std::ffi::CString;

    use super::*;

    #[test]
    fn hello_world() {
        let mut mrb = Mruby::new().unwrap();
        mrb.register_global("$example", (42, Some("hello"), [1, 2, 3], 64.5f32, true));

        unsafe {
            let owned = CString::new("puts $example").expect("Unterminated string");
            mruby_sys::mrb_load_string(mrb.state, owned.as_ptr() as *mut _);
        }
    }
}
