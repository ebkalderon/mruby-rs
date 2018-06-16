use std::ffi::CString;

use mruby_sys::{mrb_bool, mrb_float, mrb_int, mrb_state, mrb_value};

pub struct Value(pub(crate) mrb_value);

#[derive(Debug)]
pub struct State(pub(crate) *mut mrb_state);

impl State {
    pub(crate) fn new(state: *mut mrb_state) -> Self {
        State(state)
    }

    pub fn serialize_array<T: ToValue>(&mut self, val: &[T]) -> Value {
        use mruby_sys::{mrb_ary_new, mrb_ary_push};

        let State(inner) = *self;
        unsafe {
            let array = mrb_ary_new(inner);
            for item in val {
                let v = item.to_value(self);
                mrb_ary_push(inner, array, v.0);
            }

            Value(array)
        }
    }

    pub fn serialize_bool(&mut self, val: bool) -> Value {
        use mruby_sys::mrb_ext_bool_value;
        unsafe { Value(mrb_ext_bool_value(val as mrb_bool)) }
    }

    pub fn serialize_char(&mut self, val: char) -> Value {
        use std::str::from_utf8_unchecked;
        
        let buf = [val as u8];
        let thing = unsafe { from_utf8_unchecked(&buf) };
        thing.to_value(self)
    }

    pub fn serialize_fixed_num(&mut self, val: mrb_int) -> Value {
        use mruby_sys::mrb_ext_fixnum_value;
        unsafe { Value(mrb_ext_fixnum_value(val)) }
    }

    pub fn serialize_float(&mut self, val: mrb_float) -> Value {
        use mruby_sys::mrb_ext_float_value;

        let State(inner) = *self;
        unsafe { Value(mrb_ext_float_value(inner, val)) }
    }

    pub fn serialize_nil(&mut self) -> Value {
        use mruby_sys::mrb_ext_nil_value;
        unsafe { Value(mrb_ext_nil_value()) }
    }

    pub fn serialize_undef(&mut self) -> Value {
        use mruby_sys::mrb_ext_undef_value;
        unsafe { Value(mrb_ext_undef_value()) }
    }

    pub fn serialize_string<S: AsRef<str>>(&mut self, val: S) -> Value {
        use mruby_sys::mrb_str_new_cstr;

        let State(inner) = *self;
        unsafe {
            let s = CString::new(val.as_ref()).expect("Unterminated string");
            Value(mrb_str_new_cstr(inner, s.as_ptr() as *mut _))
        }
    }
}

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
        state.serialize_fixed_num(*self as mrb_int)
    }
}

impl ToValue for i32 {
    fn to_value(&self, state: &mut State) -> Value {
        state.serialize_fixed_num(*self as mrb_int)
    }
}

impl ToValue for i64 {
    fn to_value(&self, state: &mut State) -> Value {
        state.serialize_fixed_num(*self as mrb_int)
    }
}

impl ToValue for u8 {
    fn to_value(&self, state: &mut State) -> Value {
        state.serialize_fixed_num(*self as mrb_int)
    }
}

impl ToValue for u32 {
    fn to_value(&self, state: &mut State) -> Value {
        state.serialize_fixed_num(*self as mrb_int)
    }
}

impl ToValue for u64 {
    fn to_value(&self, state: &mut State) -> Value {
        state.serialize_fixed_num(*self as mrb_int)
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
