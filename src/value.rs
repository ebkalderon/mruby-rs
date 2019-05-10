pub use self::to_value::ToValue;

use std::ffi::CString;

use mruby_sys::{self, mrb_bool, mrb_float, mrb_int, mrb_state, mrb_value};

use crate::class::Class;

pub mod to_value;

#[derive(Clone, Debug)]
pub struct CastError;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Kind {
    Array,
    Boolean,
    Data,
    Float,
    Fiber,
    File,
    Function,
    Hash,
    Integer,
    Module,
    Nil,
    Object,
    Proc,
    Range,
    String,
    Symbol,
    Undefined,
}

#[derive(Clone, Debug)]
pub struct Value(pub(crate) mrb_value);

impl Value {
    pub fn kind(&self) -> Kind {
        let Value(ref val) = *self;
        match val.tt {
            mruby_sys::mrb_vtype_MRB_TT_FALSE => Kind::Boolean,
            mruby_sys::mrb_vtype_MRB_TT_FREE => Kind::Function,
            mruby_sys::mrb_vtype_MRB_TT_TRUE => Kind::Boolean,
            mruby_sys::mrb_vtype_MRB_TT_FIXNUM => Kind::Integer,
            mruby_sys::mrb_vtype_MRB_TT_SYMBOL => Kind::Symbol,
            mruby_sys::mrb_vtype_MRB_TT_UNDEF => Kind::Undefined,
            mruby_sys::mrb_vtype_MRB_TT_FLOAT => Kind::Float,
            mruby_sys::mrb_vtype_MRB_TT_CPTR => panic!(),
            mruby_sys::mrb_vtype_MRB_TT_OBJECT => Kind::Object,
            mruby_sys::mrb_vtype_MRB_TT_CLASS => panic!(),
            mruby_sys::mrb_vtype_MRB_TT_MODULE => Kind::Module,
            mruby_sys::mrb_vtype_MRB_TT_ICLASS => panic!(),
            mruby_sys::mrb_vtype_MRB_TT_SCLASS => panic!(),
            mruby_sys::mrb_vtype_MRB_TT_PROC => Kind::Proc,
            mruby_sys::mrb_vtype_MRB_TT_ARRAY => Kind::Array,
            mruby_sys::mrb_vtype_MRB_TT_HASH => Kind::Hash,
            mruby_sys::mrb_vtype_MRB_TT_STRING => Kind::String,
            mruby_sys::mrb_vtype_MRB_TT_RANGE => Kind::Range,
            mruby_sys::mrb_vtype_MRB_TT_EXCEPTION => panic!(),
            mruby_sys::mrb_vtype_MRB_TT_FILE => Kind::File,
            mruby_sys::mrb_vtype_MRB_TT_ENV => panic!(),
            mruby_sys::mrb_vtype_MRB_TT_DATA => Kind::Data,
            mruby_sys::mrb_vtype_MRB_TT_FIBER => Kind::Fiber,
            mruby_sys::mrb_vtype_MRB_TT_MAXDEFINE => panic!(),
            tt => panic!(format!("Unknown `mrb_vtype` specified: {:?}", tt)),
        }
    }

    pub(crate) fn into_inner(self) -> mrb_value {
        self.0
    }
}

#[derive(Debug)]
pub struct Deserializer(*mut mrb_state);

impl Deserializer {
    pub(crate) fn new(state: *mut mrb_state) -> Self {
        Deserializer(state)
    }
}

#[derive(Debug)]
pub struct Serializer(*mut mrb_state);

impl Serializer {
    pub(crate) fn new(state: *mut mrb_state) -> Self {
        Serializer(state)
    }

    pub fn serialize_array<T: ToValue>(&mut self, val: &[T]) -> Value {
        use mruby_sys::mrb_ary_new_from_values;

        let Serializer(state) = *self;
        let array: Vec<mrb_value> = val
            .iter()
            .map(|v| v.to_value(self))
            .map(|v| v.into_inner())
            .collect();

        unsafe {
            let array = mrb_ary_new_from_values(state, array.len() as mrb_int, array.as_ptr());
            Value(array)
        }
    }

    #[inline]
    pub fn serialize_bool(&mut self, val: bool) -> Value {
        use mruby_sys::mrb_ext_bool_value;
        unsafe { Value(mrb_ext_bool_value(val as mrb_bool)) }
    }

    #[inline]
    pub fn serialize_char(&mut self, val: char) -> Value {
        use std::str::from_utf8_unchecked;

        let buf = [val as u8];
        let s = unsafe { from_utf8_unchecked(&buf) };
        s.to_value(self)
    }

    #[inline]
    pub fn serialize_integer(&mut self, val: mrb_int) -> Value {
        use mruby_sys::mrb_ext_fixnum_value;
        unsafe { Value(mrb_ext_fixnum_value(val)) }
    }

    #[inline]
    #[cfg(not(feature = "disable-floats"))]
    pub fn serialize_float(&mut self, val: mrb_float) -> Value {
        use mruby_sys::mrb_ext_float_value;

        let Serializer(state) = *self;
        unsafe { Value(mrb_ext_float_value(state, val)) }
    }

    pub fn serialize_hash<'a, M, K, V>(&mut self, map: M) -> Value
    where
        M: IntoIterator<Item = (&'a K, &'a V)>,
        K: ToValue + 'a,
        V: ToValue + 'a,
    {
        use mruby_sys::{mrb_hash_new, mrb_hash_new_capa, mrb_hash_set};

        let Serializer(state) = *self;
        let iter = map.into_iter();

        let hash = unsafe {
            if let (_, Some(size)) = iter.size_hint() {
                mrb_hash_new_capa(state, size as mrb_int)
            } else {
                mrb_hash_new(state)
            }
        };

        for (key, value) in iter {
            let k = key.to_value(self).into_inner();
            let v = value.to_value(self).into_inner();
            unsafe {
                mrb_hash_set(state, hash, k, v);
            }
        }

        Value(hash)
    }

    #[inline]
    pub fn serialize_nil(&mut self) -> Value {
        use mruby_sys::mrb_ext_nil_value;
        unsafe { Value(mrb_ext_nil_value()) }
    }

    #[inline]
    pub fn serialize_object<T: Class>(&mut self, _obj: T) -> Value {
        unimplemented!()
    }

    #[inline]
    pub fn serialize_string<S: AsRef<str>>(&mut self, val: S) -> Value {
        use mruby_sys::mrb_str_new_cstr;

        let Serializer(state) = *self;
        unsafe {
            let s = CString::new(val.as_ref()).expect("String contains null byte");
            Value(mrb_str_new_cstr(state, s.as_ptr()))
        }
    }

    #[inline]
    pub fn serialize_undef(&mut self) -> Value {
        use mruby_sys::mrb_ext_undef_value;
        unsafe { Value(mrb_ext_undef_value()) }
    }
}
