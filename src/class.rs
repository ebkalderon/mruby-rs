use mruby_sys::RClass;

use crate::value::State;

#[derive(Debug)]
pub struct ClassData(*mut RClass);

pub trait Class {
    const NAME: &'static str;

    type Parent: Class;

    fn define(state: &mut State) -> ClassData;
}

#[derive(Debug)]
pub enum Object {}

impl Class for Object {
    const NAME: &'static str = "Object";

    type Parent = Self;

    fn define(_state: &mut State) -> ClassData {
        unimplemented!()
    }
}
