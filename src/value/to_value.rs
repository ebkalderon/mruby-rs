use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hash::Hash;

use mruby_sys::{mrb_float, mrb_int};

use super::{Serializer, Value};

pub trait ToValue {
    fn to_value(&self, ser: Serializer) -> Value;
}

impl ToValue for Value {
    fn to_value(&self, _: Serializer) -> Value {
        self.clone()
    }
}

impl ToValue for () {
    fn to_value(&self, ser: Serializer) -> Value {
        ser.serialize_undef()
    }
}

impl ToValue for bool {
    fn to_value(&self, ser: Serializer) -> Value {
        ser.serialize_bool(*self)
    }
}

impl ToValue for char {
    fn to_value(&self, ser: Serializer) -> Value {
        ser.serialize_char(*self)
    }
}

#[cfg(not(feature = "disable-floats"))]
impl ToValue for f32 {
    fn to_value(&self, ser: Serializer) -> Value {
        ser.serialize_float(*self as mrb_float)
    }
}

#[cfg(all(not(feature = "disable-floats"), not(feature = "use-f32")))]
impl ToValue for f64 {
    fn to_value(&self, ser: Serializer) -> Value {
        ser.serialize_float(*self as mrb_float)
    }
}

impl ToValue for i8 {
    fn to_value(&self, ser: Serializer) -> Value {
        ser.serialize_integer(*self as mrb_int)
    }
}

impl ToValue for i32 {
    fn to_value(&self, ser: Serializer) -> Value {
        ser.serialize_integer(*self as mrb_int)
    }
}

impl ToValue for i64 {
    fn to_value(&self, ser: Serializer) -> Value {
        ser.serialize_integer(*self as mrb_int)
    }
}

impl ToValue for isize {
    fn to_value(&self, ser: Serializer) -> Value {
        ser.serialize_integer(*self as mrb_int)
    }
}

impl ToValue for u8 {
    fn to_value(&self, ser: Serializer) -> Value {
        ser.serialize_integer(*self as mrb_int)
    }
}

impl ToValue for u32 {
    fn to_value(&self, ser: Serializer) -> Value {
        ser.serialize_integer(*self as mrb_int)
    }
}

impl ToValue for u64 {
    fn to_value(&self, ser: Serializer) -> Value {
        ser.serialize_integer(*self as mrb_int)
    }
}

impl ToValue for usize {
    fn to_value(&self, ser: Serializer) -> Value {
        ser.serialize_integer(*self as mrb_int)
    }
}

impl ToValue for str {
    fn to_value(&self, ser: Serializer) -> Value {
        ser.serialize_string(&self)
    }
}

impl ToValue for String {
    fn to_value(&self, ser: Serializer) -> Value {
        ser.serialize_string(self)
    }
}

impl<'a, T> ToValue for Cow<'a, T>
where
    T: ToOwned + ToValue + 'a,
{
    fn to_value(&self, ser: Serializer) -> Value {
        T::to_value(self.as_ref(), ser)
    }
}

impl<T: ToValue> ToValue for Option<T> {
    fn to_value(&self, ser: Serializer) -> Value {
        if let Some(ref inner) = *self {
            inner.to_value(ser)
        } else {
            ser.serialize_nil()
        }
    }
}

impl<'a, T: ToValue + ?Sized> ToValue for &'a T {
    fn to_value(&self, ser: Serializer) -> Value {
        (*self).to_value(ser)
    }
}

impl<T: ToValue> ToValue for [T] {
    fn to_value(&self, ser: Serializer) -> Value {
        ser.serialize_array(self)
    }
}

impl<T: ToValue> ToValue for BTreeSet<T> {
    fn to_value(&self, ser: Serializer) -> Value {
        ser.serialize_array(self)
    }
}

impl<T: ToValue, S> ToValue for HashSet<T, S> {
    fn to_value(&self, ser: Serializer) -> Value {
        ser.serialize_array(self)
    }
}

impl<T: ToValue> ToValue for Vec<T> {
    fn to_value(&self, ser: Serializer) -> Value {
        ser.serialize_array(self.as_slice())
    }
}

macro_rules! impl_value_array {
    ( $($arity:tt)+ ) => (
        $(
            impl<T: ToValue> ToValue for [T; $arity] {
                #[allow(non_snake_case)]
                fn to_value(&self, ser: Serializer) -> Value {
                    (&self[..]).to_value(ser)
                }
            }
        )*
    );
}

impl_value_array! { 0 1 2 3 4 5 6 7 8 9 10 11 12 }

macro_rules! impl_value_tuple {
    ( $($field:ident)+ ) => (
        impl<$($field),*> ToValue for ($($field,)*)
        where
            $(
                $field: ToValue,
            )*
        {
            #[allow(non_snake_case)]
            fn to_value(&self, ser: Serializer) -> Value {
                use mruby_sys::{mrb_ary_new, mrb_ary_push};

                let ($(ref $field,)*) = self;
                let Serializer(state) = ser;


                unsafe {
                    let array = mrb_ary_new(state);

                    $(
                        let Value(val) = $field.to_value(Serializer::new(state));
                        mrb_ary_push(state, array, val);
                    )*

                    println!("serializing: {:?}", Value(array));

                    Value(array)
                }
            }
        }
    );
}

impl_value_tuple!(A);
impl_value_tuple!(A B);
impl_value_tuple!(A B C);
impl_value_tuple!(A B C D);
impl_value_tuple!(A B C D E);
impl_value_tuple!(A B C D E F);
impl_value_tuple!(A B C D E F G);
impl_value_tuple!(A B C D E F G H);
impl_value_tuple!(A B C D E F G H I);
impl_value_tuple!(A B C D E F G H I J);
impl_value_tuple!(A B C D E F G H I J K);
impl_value_tuple!(A B C D E F G H I J K L);
impl_value_tuple!(A B C D E F G H I J K L M);

impl<K, V> ToValue for BTreeMap<K, V>
where
    K: ToValue + Ord,
    V: ToValue,
{
    fn to_value(&self, ser: Serializer) -> Value {
        ser.serialize_hash(self)
    }
}

impl<K, V, S> ToValue for HashMap<K, V, S>
where
    K: ToValue + Eq + Hash,
    V: ToValue,
{
    fn to_value(&self, ser: Serializer) -> Value {
        ser.serialize_hash(self)
    }
}
