pub use self::serializer::{ArraySerializer, Serializer};

use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};
use std::hash::Hash;
use std::rc::Rc;
use std::sync::Arc;

use mruby_sys::{mrb_float, mrb_int};

use crate::symbol::Symbol;
use crate::value::Value;

mod serializer;

pub trait ToValue {
    fn to_value(&self, ser: Serializer) -> Value;
}

impl ToValue for Value {
    fn to_value(&self, _: Serializer) -> Value {
        self.clone()
    }
}

impl ToValue for Symbol {
    fn to_value(&self, ser: Serializer) -> Value {
        ser.serialize_symbol(self)
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
        ser.serialize_float(mrb_float::from(*self))
    }
}

#[cfg(all(not(feature = "disable-floats"), not(feature = "use-f32")))]
impl ToValue for f64 {
    fn to_value(&self, ser: Serializer) -> Value {
        ser.serialize_float(*self)
    }
}

macro_rules! impl_value_integer {
    ( $($ty:ident)* ) => {
        $(
            impl ToValue for $ty {
                fn to_value(&self, ser: Serializer) -> Value {
                    ser.serialize_integer(*self as mrb_int)
                }
            }
        )*
    };
}

impl_value_integer!(i8 i32 i64 isize);
impl_value_integer!(u8 u32 u64 usize);

impl ToValue for str {
    fn to_value(&self, ser: Serializer) -> Value {
        ser.serialize_string(self)
    }
}

impl ToValue for String {
    fn to_value(&self, ser: Serializer) -> Value {
        ser.serialize_string(self)
    }
}

impl<'a, T> ToValue for Cow<'a, T>
where
    T: ToOwned + ToValue + ?Sized + 'a,
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

impl<'a, T: ToValue + ?Sized + 'a> ToValue for &'a T {
    fn to_value(&self, ser: Serializer) -> Value {
        (*self).to_value(ser)
    }
}

impl<'a, T: ToValue + ?Sized + 'a> ToValue for Arc<T> {
    fn to_value(&self, ser: Serializer) -> Value {
        self.as_ref().to_value(ser)
    }
}

impl<'a, T: ToValue + ?Sized + 'a> ToValue for Box<T> {
    fn to_value(&self, ser: Serializer) -> Value {
        self.as_ref().to_value(ser)
    }
}

impl<'a, T: ToValue + ?Sized + 'a> ToValue for Rc<T> {
    fn to_value(&self, ser: Serializer) -> Value {
        self.as_ref().to_value(ser)
    }
}

impl<T: ToValue> ToValue for [T] {
    fn to_value(&self, ser: Serializer) -> Value {
        ser.serialize_array(self)
    }
}

impl<T: ToValue> ToValue for BinaryHeap<T> {
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

impl<T: ToValue> ToValue for LinkedList<T> {
    fn to_value(&self, ser: Serializer) -> Value {
        ser.serialize_array(self)
    }
}

impl<T: ToValue> ToValue for Vec<T> {
    fn to_value(&self, ser: Serializer) -> Value {
        ser.serialize_array(self)
    }
}

impl<T: ToValue> ToValue for VecDeque<T> {
    fn to_value(&self, ser: Serializer) -> Value {
        ser.serialize_array(self)
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

impl_value_array!(0 1 2 3 4 5 6 7 8 9 10 11 12);

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
                let ($(ref $field,)*) = self;
                ser.serialize_array_hetero()
                    $(.next_element($field))*
                    .finish()
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

impl<T: ToValue + ?Sized> ToValue for Cell<T> {
    fn to_value(&self, ser: Serializer) -> Value {
        (&self).to_value(ser)
    }
}

impl<T: ToValue + ?Sized> ToValue for RefCell<T> {
    fn to_value(&self, ser: Serializer) -> Value {
        self.borrow().to_value(ser)
    }
}
