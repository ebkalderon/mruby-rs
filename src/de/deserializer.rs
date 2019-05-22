use std::ffi::CStr;
use std::iter::FusedIterator;
use std::marker::PhantomData;

use mruby_sys::{mrb_float, mrb_int, mrb_state, mrb_value};

use super::{CastError, FromValue};
use crate::class::Class;
use crate::symbol::FromSymbol;
use crate::value::Value;

#[derive(Debug)]
pub struct Deserializer {
    state: *mut mrb_state,
    pub(super) value: Value,
}

impl Deserializer {
    pub(crate) const fn new(state: *mut mrb_state, value: Value) -> Self {
        Deserializer { state, value }
    }

    // TODO: Switch to `const fn` once `match` statements are stabilized. See:
    // https://github.com/rust-lang/rust/issues/49146
    #[inline]
    pub fn deserialize_bool(self) -> Result<bool, CastError> {
        use mruby_sys::{MRB_TT_FALSE, MRB_TT_TRUE};

        match self.value.get_ref().tt {
            MRB_TT_TRUE => Ok(true),
            MRB_TT_FALSE => Ok(false),
            _ => Err(CastError::unexpected_type("value is not a boolean")),
        }
    }

    pub fn deserialize_char(self) -> Result<char, CastError> {
        let text = self.deserialize_str()?;
        let mut chars = text.chars();
        let first_char = chars
            .next()
            .ok_or_else(|| CastError::unexpected_type("string value is empty"))?;

        if chars.count() == 0 {
            Ok(first_char)
        } else {
            Err(CastError::unexpected_type("value contains multiple chars"))
        }
    }

    #[inline]
    pub fn deserialize_float(self) -> Result<mrb_float, CastError> {
        use mruby_sys::{mrb_ext_float_to_cfloat, MRB_TT_FLOAT};

        let value = self.value.into_inner();
        if value.tt == MRB_TT_FLOAT {
            unsafe { Ok(mrb_ext_float_to_cfloat(value)) }
        } else {
            Err(CastError::unexpected_type("value is not a float"))
        }
    }

    #[inline]
    pub fn deserialize_integer(self) -> Result<mrb_int, CastError> {
        use mruby_sys::{mrb_ext_fixnum_to_cint, MRB_TT_FIXNUM};

        let value = self.value.into_inner();
        if value.tt == MRB_TT_FIXNUM {
            unsafe { Ok(mrb_ext_fixnum_to_cint(value)) }
        } else {
            Err(CastError::unexpected_type("value is not an integer"))
        }
    }

    #[inline]
    pub fn deserialize_map<K: FromValue, V: FromValue>(self) -> MapIter<K, V> {
        MapIter::new(self.state, self.value)
    }

    pub fn deserialize_object<T: Class>(self) -> Result<T, CastError> {
        unimplemented!()
    }

    pub fn deserialize_option<T: FromValue>(self) -> Result<Option<T>, CastError> {
        use mruby_sys::mrb_ext_is_value_nil;

        unsafe {
            if mrb_ext_is_value_nil(*self.value.get_ref()) == 1 {
                Ok(None)
            } else {
                let de = Deserializer::new(self.state, self.value);
                T::from_value(de).map(Some)
            }
        }
    }

    pub fn deserialize_str<'a>(self) -> Result<&'a str, CastError> {
        use mruby_sys::{mrb_str_to_cstr, MRB_TT_STRING, MRB_TT_SYMBOL};

        unsafe {
            let value = self.value.get_ref();
            let s = match value.tt {
                MRB_TT_STRING => mrb_str_to_cstr(self.state, self.value.into_inner()),
                MRB_TT_SYMBOL => return self.deserialize_symbol(),
                _ => return Err(CastError::unexpected_type("value is not a string")),
            };

            CStr::from_ptr(s).to_str().map_err(CastError::from)
        }
    }

    #[inline]
    pub fn deserialize_seq<T: FromValue>(self) -> SeqIter<T> {
        SeqIter::new(self.state, self.value)
    }

    pub fn deserialize_symbol<'a, T: FromSymbol<'a> + 'a>(self) -> Result<T, CastError> {
        use mruby_sys::{mrb_ext_symbol_to_sym, mrb_str_to_cstr, mrb_sym2str, MRB_TT_SYMBOL};

        let value = self.value.into_inner();
        if value.tt != MRB_TT_SYMBOL {
            return Err(CastError::unexpected_type("value is not a symbol"));
        }

        unsafe {
            let sym = mrb_ext_symbol_to_sym(value);
            let name = mrb_sym2str(self.state, sym);
            let cstr = mrb_str_to_cstr(self.state, name);
            let s = CStr::from_ptr(cstr).to_str()?;
            T::from_name(s).map_err(CastError::from)
        }
    }

    #[inline]
    pub fn deserialize_tuple(self) -> SeqDeserializer {
        SeqDeserializer::new(self.state, self.value)
    }

    pub fn deserialize_unit(self) -> Result<(), CastError> {
        use mruby_sys::{mrb_ext_is_value_nil, MRB_TT_UNDEF};

        let value = self.value.into_inner();
        let is_undefined = value.tt == MRB_TT_UNDEF;

        if is_undefined || unsafe { mrb_ext_is_value_nil(value) == 1 } {
            Ok(())
        } else {
            Err(CastError::unexpected_type("value is not undefined"))
        }
    }
}

#[derive(Debug)]
pub struct SeqDeserializer {
    state: *mut mrb_state,
    value: Option<Value>,
    index: usize,
    len: usize,
}

impl SeqDeserializer {
    fn new(state: *mut mrb_state, value: Value) -> Self {
        use mruby_sys::{mrb_ext_ary_len, MRB_TT_ARRAY};

        let (value, len) = if value.get_ref().tt == MRB_TT_ARRAY {
            let length = unsafe { mrb_ext_ary_len(*value.get_ref()) as usize };
            (Some(value), length)
        } else {
            (None, 0)
        };

        SeqDeserializer {
            state,
            value,
            index: 0,
            len,
        }
    }

    pub fn next_element<T: FromValue>(&mut self) -> Option<Result<T, CastError>> {
        use mruby_sys::mrb_ary_ref;

        if self.index >= self.len {
            return None;
        }

        let value = self.value.take()?;
        let elem = unsafe { mrb_ary_ref(self.state, *value.get_ref(), self.index as mrb_int) };
        let de = Deserializer::new(self.state, Value(elem));
        let result = T::from_value(de);

        if result.is_err() {
            return Some(result);
        }

        self.value = Some(value);
        self.index += 1;

        Some(result)
    }

    pub const fn len(&self) -> usize {
        self.len
    }
}

#[derive(Debug)]
pub struct MapIter<K, V> {
    state: *mut mrb_state,
    entries: Option<(mrb_value, mrb_value)>,
    index: usize,
    len: usize,
    _marker: PhantomData<(K, V)>,
}

impl<K: FromValue, V: FromValue> MapIter<K, V> {
    fn new(state: *mut mrb_state, value: Value) -> Self {
        use mruby_sys::{mrb_ext_ary_len, mrb_hash_keys, mrb_hash_values, MRB_TT_HASH};

        let (entries, len) = unsafe {
            let inner = value.into_inner();
            if inner.tt == MRB_TT_HASH {
                let keys = mrb_hash_keys(state, inner);
                let values = mrb_hash_values(state, inner);
                let entries = (keys, values);
                let len = mrb_ext_ary_len(keys) as usize;
                (Some(entries), len)
            } else {
                (None, 0)
            }
        };

        MapIter {
            state,
            entries,
            index: 0,
            len,
            _marker: PhantomData,
        }
    }
}

impl<K: FromValue, V: FromValue> Iterator for MapIter<K, V> {
    type Item = Result<(K, V), CastError>;

    fn next(&mut self) -> Option<Self::Item> {
        use mruby_sys::mrb_ary_ref;

        if self.index >= self.len {
            return None;
        }

        let (keys, values) = self.entries.take()?;

        let key = unsafe { mrb_ary_ref(self.state, keys, self.index as mrb_int) };
        let key_de = Deserializer::new(self.state, Value(key));
        let key_result = K::from_value(key_de);

        if let Err(e) = key_result {
            return Some(Err(e));
        }

        let val = unsafe { mrb_ary_ref(self.state, values, self.index as mrb_int) };
        let val_de = Deserializer::new(self.state, Value(val));
        let val_result = V::from_value(val_de);

        if let Err(e) = val_result {
            return Some(Err(e));
        }

        self.entries = Some((keys, values));
        self.index += 1;

        Some(key_result.and_then(|k| val_result.map(|v| (k, v))))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<K: FromValue, V: FromValue> ExactSizeIterator for MapIter<K, V> {
    #[inline]
    fn len(&self) -> usize {
        self.len
    }
}

impl<K: FromValue, V: FromValue> FusedIterator for MapIter<K, V> {}

#[derive(Debug)]
pub struct SeqIter<T> {
    sequence: SeqDeserializer,
    _marker: PhantomData<T>,
}

impl<T: FromValue> SeqIter<T> {
    fn new(state: *mut mrb_state, value: Value) -> Self {
        SeqIter {
            sequence: SeqDeserializer::new(state, value),
            _marker: PhantomData,
        }
    }
}

impl<T: FromValue> Iterator for SeqIter<T> {
    type Item = Result<T, CastError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.sequence.next_element()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.sequence.len();
        (len, Some(len))
    }
}

impl<T: FromValue> ExactSizeIterator for SeqIter<T> {
    #[inline]
    fn len(&self) -> usize {
        self.sequence.len()
    }
}

impl<T: FromValue> FusedIterator for SeqIter<T> {}
