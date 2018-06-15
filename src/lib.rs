extern crate mruby_sys;

use mruby_sys::mrb_state;

#[derive(Debug)]
pub enum RuntimeError {
    Init,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Mruby {
    state: *mut mrb_state,
}

impl Mruby {
    pub fn new() -> Result<Self, RuntimeError> {
        let state = unsafe {
            let state = mruby_sys::mrb_open();
            if state.is_null() {
                return Err(RuntimeError::Init);
            } else {
                state
            }
        };

        Ok(Mruby { state })
    }
}

impl Drop for Mruby {
    fn drop(&mut self) {
        unsafe {
            mruby_sys::mrb_close(self.state);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_world() {
        let mut mrb = Mruby::new().unwrap();

//         unsafe {
//             mruby_sys::mrb_load_string(mrb.state, "puts \"Hello world\"!".as_ptr() as *const i8);
//         }
    }
}
