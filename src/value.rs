pub use self::from_value::FromValue;
pub use self::to_value::ToValue;

use std::ffi::{CStr, CString};
use std::fmt::{Debug, Formatter, Result as FmtResult};

use mruby_sys::{self, mrb_bool, mrb_float, mrb_int, mrb_state, mrb_value};

use crate::class::Class;

mod from_value;
mod to_value;

#[derive(Clone, Debug)]
pub struct CastError;

#[derive(Clone)]
pub struct Value(pub(crate) mrb_value);

impl Value {
    pub fn get_ref(&self) -> &mrb_value {
        &self.0
    }

    pub fn into_inner(self) -> mrb_value {
        self.0
    }
}

impl Debug for Value {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        use mruby_sys::*;

        let Value(ref inner) = *self;

        let mut debug = fmt.debug_struct(stringify!(Value));
        debug.field("type", &inner.tt);

        let value: &dyn Debug = match inner.tt {
            MRB_TT_FALSE => &false,
            MRB_TT_FIXNUM => unsafe { &inner.value.i },
            MRB_TT_FLOAT => unsafe { &inner.value.f },
            MRB_TT_TRUE => &true,
            MRB_TT_UNDEF => &(),
            _ => &"<unknown>",
        };

        debug.field("value", value).finish()
    }
}

#[derive(Debug)]
pub struct Deserializer {
    state: *mut mrb_state,
    value: Value,
}

impl Deserializer {
    pub(crate) fn new(state: *mut mrb_state, value: Value) -> Self {
        Deserializer { state, value }
    }

    #[inline]
    pub fn deserialize_bool(self) -> Result<bool, CastError> {
        use mruby_sys::{MRB_TT_FALSE, MRB_TT_TRUE};

        match self.value.into_inner().tt {
            MRB_TT_TRUE => Ok(true),
            MRB_TT_FALSE => Ok(false),
            _ => Err(CastError),
        }
    }

    pub fn deserialize_char(self) -> Result<char, CastError> {
        let text = self.deserialize_str()?;
        let mut chars = text.chars();
        let first_char = chars.next().ok_or(CastError)?;

        if chars.count() == 0 {
            Ok(first_char)
        } else {
            Err(CastError)
        }
    }

    #[inline]
    pub fn deserialize_float(self) -> Result<mrb_float, CastError> {
        use mruby_sys::{mrb_ext_float_to_cfloat, MRB_TT_FLOAT};

        let value = self.value.into_inner();
        if value.tt == MRB_TT_FLOAT {
            unsafe { Ok(mrb_ext_float_to_cfloat(value)) }
        } else {
            Err(CastError)
        }
    }

    #[inline]
    pub fn deserialize_integer(self) -> Result<mrb_int, CastError> {
        use mruby_sys::{mrb_ext_fixnum_to_cint, MRB_TT_FIXNUM};

        let value = self.value.into_inner();
        if value.tt == MRB_TT_FIXNUM {
            unsafe { Ok(mrb_ext_fixnum_to_cint(value)) }
        } else {
            Err(CastError)
        }
    }

    pub fn deserialize_map<K, V>(self) -> Result<impl Iterator<Item = (K, V)>, CastError>
    where
        K: FromValue,
        V: FromValue,
    {
        use mruby_sys::{
            mrb_ary_ref, mrb_ext_ary_len, mrb_hash_keys, mrb_hash_values, MRB_TT_HASH,
        };

        let value = self.value.into_inner();
        if value.tt == MRB_TT_HASH {
            unsafe {
                let keys = mrb_hash_keys(self.state, value);
                let values = mrb_hash_values(self.state, value);
                let len = mrb_ext_ary_len(keys) as usize;
                let mut map = Vec::with_capacity(len);

                for i in 0..len {
                    let key = Value(mrb_ary_ref(self.state, keys, i as mrb_int));
                    let key_de = Deserializer::new(self.state, key);
                    let k = K::from_value(key_de)?;

                    let val = Value(mrb_ary_ref(self.state, values, i as mrb_int));
                    let val_de = Deserializer::new(self.state, val);
                    let v = V::from_value(val_de)?;
                    map.push((k, v));
                }

                Ok(map.into_iter())
            }
        } else {
            Err(CastError)
        }
    }

    #[inline]
    pub fn deserialize_object<T: Class>(self) -> Result<T, CastError> {
        unimplemented!()
    }

    pub fn deserialize_option<T: FromValue>(self) -> Result<Option<T>, CastError> {
        use mruby_sys::mrb_ext_is_value_nil;

        unsafe {
            let value = self.value.into_inner();
            if mrb_ext_is_value_nil(self.state, value) == 1 {
                Ok(None)
            } else {
                let de = Deserializer::new(self.state, Value(value));
                T::from_value(de).map(Some)
            }
        }
    }

    pub fn deserialize_str<'a>(self) -> Result<&'a str, CastError> {
        use mruby_sys::{mrb_str_to_cstr, MRB_TT_STRING, MRB_TT_SYMBOL};

        unsafe {
            let value = self.value.into_inner();
            let s = match value.tt {
                MRB_TT_STRING => mrb_str_to_cstr(self.state, value),
                MRB_TT_SYMBOL => unimplemented!(),
                _ => return Err(CastError),
            };

            CStr::from_ptr(s).to_str().map_err(|_| CastError)
        }
    }

    #[inline]
    pub fn deserialize_unit(self) -> Result<(), CastError> {
        use mruby_sys::MRB_TT_UNDEF;

        if self.value.into_inner().tt == MRB_TT_UNDEF {
            Ok(())
        } else {
            Err(CastError)
        }
    }

    pub fn deserialize_vec<T: FromValue>(self) -> Result<Vec<T>, CastError> {
        use mruby_sys::{mrb_ary_ref, mrb_ext_ary_len, MRB_TT_ARRAY};

        let value = self.value.into_inner();
        if value.tt == MRB_TT_ARRAY {
            unsafe {
                let len = mrb_ext_ary_len(value) as usize;
                let mut vec = Vec::with_capacity(len);

                for i in 0..len {
                    let elem = Value(mrb_ary_ref(self.state, value, i as mrb_int));
                    let de = Deserializer::new(self.state, elem);
                    vec.push(T::from_value(de)?);
                }

                Ok(vec)
            }
        } else {
            Err(CastError)
        }
    }
}

#[derive(Debug)]
pub struct Serializer(*mut mrb_state);

impl Serializer {
    pub(crate) fn new(state: *mut mrb_state) -> Self {
        Serializer(state)
    }

    pub fn serialize_array<T, U>(self, val: T) -> Value
    where
        T: IntoIterator<Item = U>,
        U: ToValue,
    {
        use mruby_sys::mrb_ary_new_from_values;

        let Serializer(state) = self;
        let array: Vec<mrb_value> = val
            .into_iter()
            .map(|v| v.to_value(Serializer(state)))
            .map(|Value(v)| v)
            .collect();

        unsafe {
            let array = mrb_ary_new_from_values(state, array.len() as mrb_int, array.as_ptr());
            Value(array)
        }
    }

    #[inline]
    pub fn serialize_bool(self, val: bool) -> Value {
        use mruby_sys::mrb_ext_bool_value;
        unsafe { Value(mrb_ext_bool_value(val as mrb_bool)) }
    }

    #[inline]
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

    pub fn serialize_hash<'a, M, K, V>(self, map: M) -> Value
    where
        M: IntoIterator<Item = (&'a K, &'a V)>,
        K: ToValue + 'a,
        V: ToValue + 'a,
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

    #[inline]
    pub fn serialize_object<T: Class>(self, _obj: T) -> Value {
        unimplemented!()
    }

    #[inline]
    pub fn serialize_string<S: AsRef<str>>(self, val: S) -> Value {
        use mruby_sys::mrb_str_new_cstr;

        let Serializer(state) = self;
        unsafe {
            let s = CString::new(val.as_ref()).expect("String contains null byte");
            Value(mrb_str_new_cstr(state, s.as_ptr()))
        }
    }

    #[inline]
    pub fn serialize_undef(self) -> Value {
        use mruby_sys::mrb_ext_undef_value;
        unsafe { Value(mrb_ext_undef_value()) }
    }
}
