use std::ffi::CString;

use mruby_sys::{mrb_class_get, mrb_define_class, mrb_define_const, mrb_intern, RClass};

use crate::state::State;
use crate::value::ToValue;

pub trait Class {
    const NAME: &'static str;
    const PARENT: Option<&'static str>;

    fn define(builder: &mut Builder);
}

#[derive(Debug)]
pub struct Builder {
    state: State,
    class: *mut RClass,
}

impl Builder {
    pub(crate) unsafe fn new(state: State, name: &str, parent: Option<&str>) -> Self {
        let name = CString::new(name).expect("String contains null byte");
        let parent = match parent {
            None => (*state.as_mut_ptr()).object_class,
            Some("Array") => (*state.as_mut_ptr()).array_class,
            Some("Object") => (*state.as_mut_ptr()).object_class,
            Some("Hash") => (*state.as_mut_ptr()).hash_class,
            Some("Float") => (*state.as_mut_ptr()).float_class,
            Some("Proc") => (*state.as_mut_ptr()).proc_class,
            Some(other) => {
                let name = CString::new(other).unwrap();
                mrb_class_get(state.as_mut_ptr(), name.as_ptr())
            }
        };

        let class = mrb_define_class(state.as_mut_ptr(), name.as_ptr(), parent);
        Builder { state, class }
    }

    pub fn def_const<N, V>(&mut self, name: N, value: V) -> &mut Self
    where
        N: AsRef<str>,
        V: ToValue,
    {
        let name = CString::new(name.as_ref()).expect("String contains null byte");
        let value = value.to_value(&mut self.state.serialize()).into_inner();

        unsafe {
            mrb_define_const(self.state.as_mut_ptr(), self.class, name.as_ptr(), value);
            self
        }
    }

    pub fn def_var<N, V>(&mut self, name: N, value: V) -> &mut Self
    where
        N: AsRef<str>,
        V: ToValue,
    {
        let s = name.as_ref();
        let name = CString::new(s).expect("String contains null byte");
        let value = value.to_value(&mut self.state.serialize()).into_inner();

        unsafe {
            let sym = mrb_intern(self.state.as_mut_ptr(), name.as_ptr(), s.len());
            unimplemented!()
        }
    }
}
