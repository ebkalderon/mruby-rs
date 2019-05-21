use std::ffi::CString;

use mruby_sys::{mrb_ary_new_from_values, mrb_bool, mrb_float, mrb_int, mrb_state, mrb_value};

use super::ToValue;
use crate::class::Class;
use crate::symbol::ToSymbol;
use crate::value::Value;

#[derive(Debug)]
pub struct Serializer(*mut mrb_state);

impl Serializer {
    pub(crate) const fn new(state: *mut mrb_state) -> Self {
        Serializer(state)
    }

    pub fn serialize_array<T, U>(self, val: T) -> Value
    where
        T: IntoIterator<Item = U>,
        U: ToValue,
    {
        let Serializer(state) = self;
        let array: Vec<mrb_value> = val
            .into_iter()
            .map(|v| v.to_value(Serializer(state)))
            .map(|v| v.into_inner())
            .collect();

        unsafe {
            let array = mrb_ary_new_from_values(state, array.len() as mrb_int, array.as_ptr());
            Value(array)
        }
    }

    #[inline]
    pub fn serialize_array_hetero(self) -> ArraySerializer {
        let Serializer(state) = self;
        ArraySerializer::new(state)
    }

    #[inline]
    pub fn serialize_bool(self, val: bool) -> Value {
        use mruby_sys::mrb_ext_bool_value;
        unsafe { Value(mrb_ext_bool_value(val as mrb_bool)) }
    }

    pub fn serialize_char(self, val: char) -> Value {
        let mut buf = [0u8; 2];
        let s = val.encode_utf8(&mut buf);
        s.to_value(self)
    }

    #[inline]
    pub fn serialize_integer(self, val: mrb_int) -> Value {
        use mruby_sys::mrb_ext_fixnum_value;
        unsafe { Value(mrb_ext_fixnum_value(val)) }
    }

    #[inline]
    #[cfg(not(feature = "disable-floats"))]
    pub fn serialize_float(self, val: mrb_float) -> Value {
        use mruby_sys::mrb_ext_float_value;

        let Serializer(state) = self;
        unsafe { Value(mrb_ext_float_value(state, val)) }
    }

    pub fn serialize_hash<M, K, V>(self, map: M) -> Value
    where
        M: IntoIterator<Item = (K, V)>,
        K: ToValue,
        V: ToValue,
    {
        use mruby_sys::{mrb_hash_new, mrb_hash_new_capa, mrb_hash_set};

        let Serializer(state) = self;
        let iter = map.into_iter();

        let hash = unsafe {
            if let (_, Some(size)) = iter.size_hint() {
                mrb_hash_new_capa(state, size as mrb_int)
            } else {
                mrb_hash_new(state)
            }
        };

        for (key, value) in iter {
            let k = key.to_value(Serializer(state)).into_inner();
            let v = value.to_value(Serializer(state)).into_inner();
            unsafe {
                mrb_hash_set(state, hash, k, v);
            }
        }

        Value(hash)
    }

    #[inline]
    pub fn serialize_nil(self) -> Value {
        use mruby_sys::mrb_ext_nil_value;
        unsafe { Value(mrb_ext_nil_value()) }
    }

    pub fn serialize_object<T: Class>(self, _obj: T) -> Value {
        unimplemented!()
    }

    pub fn serialize_string<S: AsRef<str>>(self, val: S) -> Value {
        use mruby_sys::mrb_str_new_cstr;

        let Serializer(state) = self;
        let cstr = CString::new(val.as_ref()).expect("String contains null byte");

        unsafe { Value(mrb_str_new_cstr(state, cstr.as_ptr())) }
    }

    pub fn serialize_symbol<T: ToSymbol>(self, sym: T) -> Value {
        use mruby_sys::{mrb_ext_symbol_value, mrb_intern_cstr};

        let Serializer(state) = self;
        let name = CString::new(sym.as_str()).expect("String contains null byte");
        let symbol = unsafe { mrb_intern_cstr(state, name.as_ptr()) };

        unsafe { Value(mrb_ext_symbol_value(symbol)) }
    }

    #[inline]
    pub fn serialize_undef(self) -> Value {
        use mruby_sys::mrb_ext_undef_value;
        unsafe { Value(mrb_ext_undef_value()) }
    }
}

#[derive(Debug)]
pub struct ArraySerializer {
    state: *mut mrb_state,
    array: Value,
}

impl ArraySerializer {
    fn new(state: *mut mrb_state) -> Self {
        use mruby_sys::mrb_ary_new;
        let array = unsafe { Value(mrb_ary_new(state)) };
        ArraySerializer { state, array }
    }

    pub fn next_element<T: ToValue>(self, elem: T) -> Self {
        use mruby_sys::mrb_ary_push;

        let ser = Serializer(self.state);
        let value = elem.to_value(ser).into_inner();
        unsafe {
            mrb_ary_push(self.state, *self.array.get_ref(), value);
            self
        }
    }

    #[inline]
    pub fn finish(self) -> Value {
        self.array
    }
}
