pub use self::deserializer::{Deserializer, MapIter, SeqDeserializer, SeqIter};
pub use self::error::CastError;

use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::collections::*;
use std::convert::TryFrom;
use std::hash::{BuildHasher, Hash};
use std::rc::Rc;
use std::sync::Arc;

use crate::symbol::Symbol;
use crate::value::Value;

mod deserializer;
mod error;

pub trait FromValue: Sized {
    fn from_value(de: Deserializer) -> Result<Self, CastError>;
}

impl FromValue for Value {
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        Ok(de.value)
    }
}

impl FromValue for Symbol {
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        de.deserialize_symbol()
    }
}

impl FromValue for () {
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        de.deserialize_unit()
    }
}

impl FromValue for bool {
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        de.deserialize_bool()
    }
}

impl FromValue for char {
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        de.deserialize_char()
    }
}

#[cfg(not(feature = "disable-floats"))]
impl FromValue for f32 {
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        de.deserialize_float().map(|f| f as f32)
    }
}

#[cfg(not(feature = "disable-floats"))]
impl FromValue for f64 {
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        de.deserialize_float().map(From::from)
    }
}

macro_rules! impl_value_integer {
    ( $($ty:ident)* ) => {
        $(
            impl FromValue for $ty {
                fn from_value(de: Deserializer) -> Result<Self, CastError> {
                    let num = de.deserialize_integer()?;
                    Self::try_from(num).map_err(|_| CastError::numeric_conversion(num, stringify!($ty)))
                }
            }
        )*
    };
}

impl_value_integer!(i8 i16 i32 i64 isize);
impl_value_integer!(u8 u16 u32 u64 usize);

impl<'a> FromValue for &'a str {
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        de.deserialize_str()
    }
}

impl FromValue for String {
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        de.deserialize_str().map(ToString::to_string)
    }
}

impl<'a> FromValue for Cow<'a, str> {
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        de.deserialize_str().map(Cow::from)
    }
}

impl<'a, T: FromValue + Clone> FromValue for Cow<'a, [T]> {
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        de.deserialize_seq().collect()
    }
}

impl<T: FromValue> FromValue for Option<T> {
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        de.deserialize_option()
    }
}

impl<T: FromValue> FromValue for Arc<T> {
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        T::from_value(de).map(Arc::from)
    }
}

impl<T: FromValue> FromValue for Box<T> {
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        T::from_value(de).map(Box::new)
    }
}

impl FromValue for Box<str> {
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        de.deserialize_str().map(Box::from)
    }
}

impl<T: FromValue> FromValue for Box<[T]> {
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        de.deserialize_seq().collect()
    }
}

impl<T: FromValue> FromValue for Rc<T> {
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        T::from_value(de).map(Rc::from)
    }
}

impl<T> FromValue for BinaryHeap<T>
where
    T: FromValue + Ord,
{
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        de.deserialize_seq().collect()
    }
}

impl<T> FromValue for BTreeSet<T>
where
    T: FromValue + Ord,
{
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        de.deserialize_seq().collect()
    }
}

impl<T, S> FromValue for HashSet<T, S>
where
    T: FromValue + Eq + Hash,
    S: BuildHasher + Default,
{
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        de.deserialize_seq().collect()
    }
}

impl<T: FromValue> FromValue for LinkedList<T> {
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        de.deserialize_seq().collect()
    }
}

impl<T: FromValue> FromValue for Vec<T> {
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        de.deserialize_seq().collect()
    }
}

impl<T: FromValue> FromValue for VecDeque<T> {
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        de.deserialize_seq().collect()
    }
}

macro_rules! impl_value_array {
    ( $($arity:tt)+ ) => {
        $(
            impl<T: FromValue> FromValue for [T; $arity] {
                fn from_value(de: Deserializer) -> Result<Self, CastError> {
                    let iter = de.deserialize_seq::<T>();
                    let actual_len = iter.len();
                    let expected_len = $arity;

                    if actual_len != expected_len {
                        return Err(CastError::length(expected_len, actual_len));
                    }

                    let mut array: [T; $arity] = unsafe { std::mem::uninitialized() };
                    for (i, elem) in iter.enumerate() {
                        array[i] = elem?;
                    }

                    Ok(array)
                }
            }
        )+
    };
}

impl_value_array! { 0 1 2 3 4 5 6 7 8 9 10 11 12 }

macro_rules! impl_value_tuple {
    ( $($field:ident)+ ) => {
        impl<$($field,)*> FromValue for ($($field,)*)
        where
            $(
                $field: FromValue,
            )*
        {
            #[allow(non_snake_case)]
            fn from_value(de: Deserializer) -> Result<Self, CastError> {
                let mut iter = de.deserialize_tuple();
                let actual_len = iter.len();
                let expected_len = count_tokens!($($field)*);

                if actual_len != expected_len {
                    return Err(CastError::length(expected_len, actual_len));
                }

                $(
                    let $field = iter.next_element().expect("expected another element")?;
                )*

                Ok(($($field,)*))
            }
        }
    };
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

impl<K, V> FromValue for BTreeMap<K, V>
where
    K: FromValue + Ord,
    V: FromValue,
{
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        de.deserialize_map().collect()
    }
}

impl<K, V, S> FromValue for HashMap<K, V, S>
where
    K: FromValue + Hash + Eq,
    V: FromValue,
    S: BuildHasher + Default,
{
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        de.deserialize_map().collect()
    }
}

impl<T: FromValue> FromValue for Cell<T> {
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        T::from_value(de).map(Cell::from)
    }
}

impl<T: FromValue> FromValue for RefCell<T> {
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        T::from_value(de).map(RefCell::from)
    }
}
