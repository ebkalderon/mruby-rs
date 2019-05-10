use std::ffi::CString;

use mruby_sys::{mrb_bool, mrb_float, mrb_int, mrb_state, mrb_value};

use crate::class::Class;
use crate::value::{Serializer, ToValue, Value};

#[derive(Debug)]
pub struct State(*mut mrb_state);

impl State {
    pub(crate) fn new(state: *mut mrb_state) -> Self {
        State(state)
    }

    pub(crate) fn as_mut_ptr(&self) -> *mut mrb_state {
        self.0
    }

    pub fn serialize(&mut self) -> Serializer {
        Serializer::new(self.0)
    }
}
