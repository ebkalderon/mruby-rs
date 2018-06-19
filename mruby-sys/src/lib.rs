#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub use self::ffi::*;

use std::os::raw::c_void;

mod ffi;

extern "C" {
    pub fn mrb_ext_bool_value(boolean: mrb_bool) -> mrb_value;

    pub fn mrb_ext_cptr_value(mrb: *mut mrb_state, p: *mut c_void) -> mrb_value;

    pub fn mrb_ext_fixnum_value(i: mrb_int) -> mrb_value;

    #[cfg(feature = "use-floats")]
    pub fn mrb_ext_float_value(mrb: *mut mrb_state, f: mrb_float) -> mrb_value;

    pub fn mrb_ext_nil_value() -> mrb_value;

    pub fn mrb_ext_symbol_value(i: mrb_sym) -> mrb_value;

    pub fn mrb_ext_undef_value() -> mrb_value;
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;
    use std::ptr;

    use super::*;

    #[test]
    fn open_close() {
        unsafe {
            let state = mrb_open();
            mrb_close(state);
        }
    }

    #[test]
    fn ext_bool_value() {
        unsafe {
            let _true = mrb_ext_bool_value(true as mrb_bool);
            let _false = mrb_ext_bool_value(false as mrb_bool);
        }
    }

    #[test]
    fn ext_cptr_value() {
        unsafe {
            let state = mrb_open();
            let _val = mrb_ext_cptr_value(state, ptr::null_mut());
            mrb_close(state);
        }
    }

    #[test]
    fn ext_fixnum_value() {
        unsafe {
            let _val = mrb_ext_fixnum_value(42 as mrb_int);
        }
    }

    #[cfg(feature = "use-floats")]
    #[test]
    fn ext_float_value() {
        unsafe {
            let state = mrb_open();
            let _val = mrb_ext_float_value(state, 3.14159f32 as mrb_float);
            mrb_close(state);
        }
    }

    #[test]
    fn ext_nil_value() {
        unsafe {
            let _val = mrb_ext_nil_value();
        }
    }

    #[test]
    fn ext_symbol_value() {
        unsafe {
            let state = mrb_open();
            let sym = mrb_intern_cstr(state, CString::new("example").unwrap().as_ptr());
            let _val = mrb_ext_symbol_value(sym);
        }
    }

    #[test]
    fn ext_undef_value() {
        unsafe {
            let _val = mrb_ext_undef_value();
        }
    }
}
