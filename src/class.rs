use mruby_sys::RClass;

use value::State;

#[derive(Debug)]
pub struct Class(*mut RClass);

pub trait ToClass {
    fn to_class(&self, _state: &mut State) -> Class {
        unimplemented!()
    }
}
