#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub use self::ffi::*;

mod ffi;

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
