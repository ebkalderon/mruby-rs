use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::convert::TryInto;
use std::hash::{BuildHasher, Hash};

use super::{CastError, Deserializer, Value};

pub trait FromValue: Sized {
    fn from_value(de: Deserializer) -> Result<Self, CastError>;
}

impl FromValue for Value {
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        Ok(de.value)
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

impl FromValue for i8 {
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        let num = de.deserialize_integer()?;
        num.try_into().map_err(|_| CastError)
    }
}

impl FromValue for i16 {
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        let num = de.deserialize_integer()?;
        num.try_into().map_err(|_| CastError)
    }
}

impl FromValue for i32 {
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        let num = de.deserialize_integer()?;
        num.try_into().map_err(|_| CastError)
    }
}

impl FromValue for i64 {
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        de.deserialize_integer()
    }
}

impl FromValue for isize {
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        let num = de.deserialize_integer()?;
        num.try_into().map_err(|_| CastError)
    }
}

impl FromValue for u8 {
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        let num = de.deserialize_integer()?;
        num.try_into().map_err(|_| CastError)
    }
}

impl FromValue for u16 {
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        let num = de.deserialize_integer()?;
        num.try_into().map_err(|_| CastError)
    }
}

impl FromValue for u32 {
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        let num = de.deserialize_integer()?;
        num.try_into().map_err(|_| CastError)
    }
}

impl FromValue for u64 {
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        let num = de.deserialize_integer()?;
        num.try_into().map_err(|_| CastError)
    }
}

impl FromValue for usize {
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        let num = de.deserialize_integer()?;
        num.try_into().map_err(|_| CastError)
    }
}

impl<'a> FromValue for &'a str {
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        de.deserialize_str()
    }
}

impl FromValue for String {
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        de.deserialize_str().map(|s| s.to_string())
    }
}

impl<'a, T> FromValue for Cow<'a, T>
where
    T: FromValue + ToOwned + 'a,
    Self: From<T>,
{
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        T::from_value(de).map(Cow::from)
    }
}

impl<T: FromValue> FromValue for Option<T> {
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        de.deserialize_option()
    }
}

impl<T: FromValue + Ord> FromValue for BTreeSet<T> {
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        de.deserialize_vec().map(|v| v.into_iter().collect())
    }
}

impl<T, S> FromValue for HashSet<T, S>
where
    T: FromValue + Hash + Eq,
    S: BuildHasher + Default,
{
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        de.deserialize_vec().map(|v| v.into_iter().collect())
    }
}

impl<T: FromValue> FromValue for Vec<T> {
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        de.deserialize_vec()
    }
}

macro_rules! impl_value_array {
    ( $($arity:tt)+ ) => {
        $(
            impl<T: FromValue> FromValue for [T; $arity] {
                fn from_value(de: Deserializer) -> Result<Self, CastError> {
                    use std::mem;
                    use mruby_sys::{mrb_ary_ref, mrb_ext_ary_len, mrb_int, MRB_TT_ARRAY};

                    let value = de.value.into_inner();
                    if value.tt == MRB_TT_ARRAY {
                        unsafe {
                            let len = mrb_ext_ary_len(value) as usize;
                            let mut array: [T; $arity] = mem::uninitialized();

                            for i in 0..len {
                                let elem = Value(mrb_ary_ref(de.state, value, i as mrb_int));
                                let de = Deserializer::new(de.state, elem);
                                array[i] = T::from_value(de)?;
                            }

                            Ok(array)
                        }
                    } else {
                        Err(CastError)
                    }
                }
            }
        )+
    };
}

impl_value_array! { 0 1 2 3 4 5 6 7 8 9 10 11 12 }

// TODO: Use `replace_expr!` and `count_tokens!` macros in `to_value` as well.

macro_rules! replace_expr {
    ( $_first:tt $sub:expr ) => {
        $sub
    };
}

macro_rules! count_tokens {
    ( $($elem:tt)* ) => {
        <[()]>::len(&[$(replace_expr!($elem ())),*])
    };
}

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
                use mruby_sys::{mrb_ary_ref, mrb_ext_ary_len, mrb_int, MRB_TT_ARRAY};

                let value = de.value.into_inner();
                if value.tt == MRB_TT_ARRAY {
                    let len = unsafe { mrb_ext_ary_len(value) as usize };
                    let tuple_arity = count_tokens!($($field)*);

                    if tuple_arity != len {
                        return Err(CastError);
                    }

                    let mut index = 0;

                    $(
                        let elem = unsafe { mrb_ary_ref(de.state, value, index as mrb_int) };
                        let elem_de = Deserializer::new(de.state, Value(elem));
                        let $field = $field::from_value(elem_de)?;
                        index += 1;
                    )*

                    drop(index);

                    Ok(($($field,)*))
                } else {
                    Err(CastError)
                }
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
        de.deserialize_map().map(|m| m.collect())
    }
}

impl<K, V, S> FromValue for HashMap<K, V, S>
where
    K: FromValue + Hash + Eq,
    V: FromValue,
    S: BuildHasher + Default,
{
    fn from_value(de: Deserializer) -> Result<Self, CastError> {
        de.deserialize_map().map(|m| m.collect())
    }
}
