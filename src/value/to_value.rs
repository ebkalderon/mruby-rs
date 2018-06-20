use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

use mruby_sys::{mrb_float, mrb_int};

use super::{State, Value};

pub trait ToValue {
    fn to_value(&self, state: &mut State) -> Value;
}

impl ToValue for () {
    fn to_value(&self, state: &mut State) -> Value {
        state.serialize_undef()
    }
}

impl ToValue for bool {
    fn to_value(&self, state: &mut State) -> Value {
        state.serialize_bool(*self)
    }
}

impl ToValue for char {
    fn to_value(&self, state: &mut State) -> Value {
        state.serialize_char(*self)
    }
}

impl ToValue for f32 {
    fn to_value(&self, state: &mut State) -> Value {
        state.serialize_float(*self as mrb_float)
    }
}

impl ToValue for f64 {
    fn to_value(&self, state: &mut State) -> Value {
        state.serialize_float(*self as mrb_float)
    }
}

impl ToValue for i8 {
    fn to_value(&self, state: &mut State) -> Value {
        state.serialize_integer(*self as mrb_int)
    }
}

impl ToValue for i32 {
    fn to_value(&self, state: &mut State) -> Value {
        state.serialize_integer(*self as mrb_int)
    }
}

impl ToValue for i64 {
    fn to_value(&self, state: &mut State) -> Value {
        state.serialize_integer(*self as mrb_int)
    }
}

impl ToValue for u8 {
    fn to_value(&self, state: &mut State) -> Value {
        state.serialize_integer(*self as mrb_int)
    }
}

impl ToValue for u32 {
    fn to_value(&self, state: &mut State) -> Value {
        state.serialize_integer(*self as mrb_int)
    }
}

impl ToValue for u64 {
    fn to_value(&self, state: &mut State) -> Value {
        state.serialize_integer(*self as mrb_int)
    }
}

impl<'a> ToValue for &'a str {
    fn to_value(&self, state: &mut State) -> Value {
        state.serialize_string(self)
    }
}

impl ToValue for String {
    fn to_value(&self, state: &mut State) -> Value {
        state.serialize_string(self)
    }
}

impl<T: ToValue> ToValue for Option<T> {
    fn to_value(&self, state: &mut State) -> Value {
        if let Some(ref inner) = *self {
            inner.to_value(state)
        } else {
            state.serialize_nil()
        }
    }
}

impl<'a, T: ToValue> ToValue for &'a T {
    fn to_value(&self, state: &mut State) -> Value {
        (*self).to_value(state)
    }
}

impl<'a, T: ToValue> ToValue for &'a [T] {
    fn to_value(&self, state: &mut State) -> Value {
        state.serialize_array(self)
    }
}

impl<T: ToValue> ToValue for Vec<T> {
    fn to_value(&self, state: &mut State) -> Value {
        state.serialize_array(self.as_slice())
    }
}

macro_rules! impl_value_array {
    ( $($arity:tt)+ ) => (
        $(
            impl<T: ToValue> ToValue for [T; $arity] {
                #[allow(non_snake_case)]
                fn to_value(&self, state: &mut State) -> Value {
                    (&self[..]).to_value(state)
                }
            }
        )*
    );
}

impl_value_array! { 0 1 2 3 4 5 6 7 8 9 10 11 12 }

macro_rules! impl_value_tuple {
    ( $($field:ident)+ ) => (
        impl<$($field: ToValue,)*> ToValue for ($($field,)*) {
            #[allow(non_snake_case)]
            fn to_value(&self, state: &mut State) -> Value {
                use mruby_sys::{mrb_ary_new, mrb_ary_push};

                let ($(ref $field,)*) = *self;
                let State(inner) = *state;

                unsafe {
                    let array = mrb_ary_new(inner);

                    $(
                        let Value(val) = $field.to_value(state);
                        mrb_ary_push(inner, array, val);
                    )*

                    Value(array)
                }
            }
        }
    );
}

impl_value_tuple!(T0);
impl_value_tuple!(T0 T1);
impl_value_tuple!(T0 T1 T2);
impl_value_tuple!(T0 T1 T2 T3);
impl_value_tuple!(T0 T1 T2 T3 T4);
impl_value_tuple!(T0 T1 T2 T3 T4 T5);
impl_value_tuple!(T0 T1 T2 T3 T4 T5 T6);
impl_value_tuple!(T0 T1 T2 T3 T4 T5 T6 T7);
impl_value_tuple!(T0 T1 T2 T3 T4 T5 T6 T7 T8);
impl_value_tuple!(T0 T1 T2 T3 T4 T5 T6 T7 T8 T9);
impl_value_tuple!(T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10);
impl_value_tuple!(T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11);
impl_value_tuple!(T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12);

impl<K, V> ToValue for BTreeMap<K, V>
where
    K: ToValue + Eq + Hash,
    V: ToValue + Eq + Hash,
{
    fn to_value(&self, state: &mut State) -> Value {
        state.serialize_hash(self.iter())
    }
}

impl<K, V> ToValue for HashMap<K, V>
where
    K: ToValue + Eq + Hash,
    V: ToValue + Eq + Hash,
{
    fn to_value(&self, state: &mut State) -> Value {
        state.serialize_hash(self.iter())
    }
}
