use mruby_sys::mrb_state;

use crate::value::{Deserializer, Serializer, Value};

#[derive(Debug)]
pub struct State(*mut mrb_state);

impl State {
    pub(crate) fn new(state: *mut mrb_state) -> Self {
        State(state)
    }

    pub(crate) fn as_mut_ptr(&mut self) -> *mut mrb_state {
        self.0
    }

    pub fn deserialize(&mut self, value: Value) -> Deserializer {
        let State(state) = *self;
        Deserializer::new(state, value)
    }

    pub fn serialize(&mut self) -> Serializer {
        let State(state) = *self;
        Serializer::new(state)
    }
}
