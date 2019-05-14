//! Raw FFI bindings to [mruby](https://mruby.org/). For higher-level mruby bindings, see
//! [mruby-rs].
//!
//! FIXME: Need to switch to `std::ffi::VaList` once
//! [rust-lang/rust#44930](https://github.com/rust-lang/rust/issues/44930) is stabilized.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::os::raw::{c_char, c_void};

#[cfg(feature = "stdio")]
use libc::FILE;

#[cfg(not(feature = "use-f32"))]
#[cfg(not(feature = "debug"))]
#[cfg(feature = "stdio")]
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/double_nodebug_stdio.rs"
));

#[cfg(not(feature = "use-f32"))]
#[cfg(not(feature = "debug"))]
#[cfg(not(feature = "stdio"))]
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/double_nodebug_nostdio.rs"
));

#[cfg(not(feature = "use-f32"))]
#[cfg(feature = "debug")]
#[cfg(feature = "stdio")]
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/double_debug_stdio.rs"
));

#[cfg(not(feature = "use-f32"))]
#[cfg(feature = "debug")]
#[cfg(not(feature = "stdio"))]
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/double_debug_nostdio.rs"
));

#[cfg(feature = "use-f32")]
#[cfg(not(feature = "debug"))]
#[cfg(feature = "stdio")]
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/float_nodebug_stdio.rs"
));

#[cfg(feature = "use-f32")]
#[cfg(not(feature = "debug"))]
#[cfg(not(feature = "stdio"))]
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/float_nodebug_nostdio.rs"
));

#[cfg(feature = "use-f32")]
#[cfg(feature = "debug")]
#[cfg(feature = "stdio")]
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/float_debug_stdio.rs"
));

#[cfg(feature = "use-f32")]
#[cfg(feature = "debug")]
#[cfg(not(feature = "stdio"))]
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/float_debug_nostdio.rs"
));

#[cfg(feature = "disable-floats")]
#[cfg(not(feature = "debug"))]
#[cfg(feature = "stdio")]
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/nofloat_nodebug_stdio.rs"
));

#[cfg(feature = "disable-floats")]
#[cfg(not(feature = "debug"))]
#[cfg(not(feature = "stdio"))]
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/nofloat_nodebug_nostdio.rs"
));

#[cfg(feature = "disable-floats")]
#[cfg(feature = "debug")]
#[cfg(feature = "stdio")]
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/nofloat_debug_stdio.rs"
));

#[cfg(feature = "disable-floats")]
#[cfg(feature = "debug")]
#[cfg(not(feature = "stdio"))]
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/nofloat_debug_nostdio.rs"
));

extern "C" {
    #[inline]
    pub fn mrb_ext_ary_len(array: mrb_value) -> mrb_int;

    #[inline]
    pub fn mrb_ext_bool_value(boolean: mrb_bool) -> mrb_value;

    #[inline]
    pub fn mrb_ext_class_value(c: *mut RClass) -> mrb_value;

    #[inline]
    pub fn mrb_ext_cptr_value(mrb: *mut mrb_state, p: *mut c_void) -> mrb_value;

    #[inline]
    pub fn mrb_ext_fixnum_to_cint(num: mrb_value) -> mrb_int;

    #[inline]
    pub fn mrb_ext_fixnum_value(i: mrb_int) -> mrb_value;

    #[cfg(not(feature = "disable-floats"))]
    #[inline]
    pub fn mrb_ext_float_to_cfloat(flt: mrb_value) -> mrb_float;

    #[cfg(not(feature = "disable-floats"))]
    #[inline]
    pub fn mrb_ext_float_value(mrb: *mut mrb_state, f: mrb_float) -> mrb_value;

    #[inline]
    pub fn mrb_ext_is_value_nil(mrb: *mut mrb_state, v: mrb_value) -> mrb_bool;

    #[inline]
    pub fn mrb_ext_nil_value() -> mrb_value;

    #[inline]
    pub fn mrb_ext_raise(mrb: *mut mrb_state, err: *const c_char, msg: *const c_char) -> c_void;

    #[inline]
    pub fn mrb_ext_symbol_value(i: mrb_sym) -> mrb_value;

    #[inline]
    pub fn mrb_ext_undef_value() -> mrb_value;
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;
    use std::ptr;

    use super::*;

    #[test]
    fn mrb_open_close() {
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
    fn ext_fixnum_to_cint() {
        let input = 123;
        let fixnum = unsafe { mrb_ext_fixnum_value(input as mrb_int) };
        let output = unsafe { mrb_ext_fixnum_to_cint(fixnum) };
        assert_eq!(input, output);
    }

    #[test]
    fn ext_fixnum_value() {
        unsafe {
            let _val = mrb_ext_fixnum_value(42 as mrb_int);
        }
    }

    #[cfg(not(feature = "disable-floats"))]
    #[test]
    fn ext_float_to_cfloat() {
        unsafe {
            let state = mrb_open();

            let input = 1.0;
            let float = mrb_ext_float_value(state, input as mrb_float);
            let output = mrb_ext_float_to_cfloat(float);
            assert_eq!(input, output);

            mrb_close(state);
        }
    }

    #[cfg(not(feature = "disable-floats"))]
    #[test]
    fn ext_float_value() {
        unsafe {
            let state = mrb_open();
            let _val = mrb_ext_float_value(state, 3.14159f32 as mrb_float);
            mrb_close(state);
        }
    }

    #[test]
    fn ext_is_value_nil() {
        unsafe {
            let state = mrb_open();

            let val = mrb_ext_nil_value();
            let is_nil = mrb_ext_is_value_nil(state, val) == 1;
            assert!(is_nil);

            let val = mrb_ext_fixnum_value(5);
            let is_not_nil = mrb_ext_is_value_nil(state, val) == 0;
            assert!(is_not_nil);

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
    #[ignore]
    fn ext_raise_success() {
        unsafe {
            let state = mrb_open();
            let error = CString::new("RuntimeError").unwrap();
            let message = CString::new("hello world").unwrap();
            mrb_ext_raise(state, error.as_ptr(), message.as_ptr());
            mrb_close(state);
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
