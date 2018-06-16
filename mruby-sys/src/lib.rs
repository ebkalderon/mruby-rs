#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub use self::ffi::*;

use std::os::raw::c_void;

mod ffi;

extern "C" {
    #[link_name = "\u{1}_mrb_ext_bool_value"]
    pub fn mrb_ext_bool_value(boolean: mrb_bool) -> mrb_value;

    #[link_name = "\u{1}_mrb_ext_cptr_value"]
    pub fn mrb_ext_cptr_value(mrb: *mut mrb_state, p: *mut c_void) -> mrb_value;

    #[link_name = "\u{1}_mrb_ext_fixnum_value"]
    pub fn mrb_ext_fixnum_value(i: mrb_int) -> mrb_value;

    #[cfg(feature = "use-floats")]
    #[link_name = "\u{1}_mrb_ext_float_value"]
    pub fn mrb_ext_float_value(mrb: *mut mrb_state, f: mrb_float) -> mrb_value;

    #[link_name = "\u{1}_mrb_ext_nil_value"]
    pub fn mrb_ext_nil_value() -> mrb_value;

    #[link_name = "\u{1}_mrb_ext_symbol_value"]
    pub fn mrb_ext_symbol_value(i: mrb_sym) -> mrb_value;

    #[link_name = "\u{1}_mrb_ext_undef_value"]
    pub fn mrb_ext_undef_value() -> mrb_value;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_close() {
        unsafe {
            let state = mrb_open();
            mrb_close(state);
        }
    }
}
