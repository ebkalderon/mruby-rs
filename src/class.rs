use std::ffi::CString;

use mruby_sys::{mrb_class_get, mrb_define_class, mrb_intern, mrb_state, RClass};

use crate::ser::{Serializer, ToValue};

pub trait Class {
    const NAME: &'static str;
    const PARENT: Option<&'static str>;

    fn define(builder: &mut Builder);
}

#[derive(Debug)]
pub struct Builder {
    state: *mut mrb_state,
    class: *mut RClass,
}

impl Builder {
    pub(crate) unsafe fn new(state: *mut mrb_state, name: &str, parent: Option<&str>) -> Self {
        let name = CString::new(name).expect("String contains null byte");
        let parent = match parent {
            None => (*state).object_class,
            Some("Array") => (*state).array_class,
            Some("Object") => (*state).object_class,
            Some("Hash") => (*state).hash_class,
            Some("Float") => (*state).float_class,
            Some("Proc") => (*state).proc_class,
            Some(other) => {
                let name = CString::new(other).unwrap();
                mrb_class_get(state, name.as_ptr())
            }
        };

        let class = mrb_define_class(state, name.as_ptr(), parent);
        Builder { state, class }
    }

    pub fn def_const<N, V>(&mut self, name: N, value: V) -> &mut Self
    where
        N: AsRef<str>,
        V: ToValue,
    {
        use mruby_sys::mrb_define_const;

        let name = CString::new(name.as_ref()).expect("String contains null byte");
        let value = value.to_value(Serializer::new(self.state)).into_inner();

        unsafe {
            mrb_define_const(self.state, self.class, name.as_ptr(), value);
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
        let _value = value.to_value(Serializer::new(self.state)).into_inner();

        unsafe {
            let _sym = mrb_intern(self.state, name.as_ptr(), s.len());
            unimplemented!()
        }
    }
}
