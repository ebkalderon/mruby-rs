use std::fmt::{Debug, Formatter, Result as FmtResult};

use mruby_sys::*;

#[derive(Clone)]
pub struct Value(pub(crate) mrb_value);

impl Value {
    pub const fn get_ref(&self) -> &mrb_value {
        &self.0
    }

    pub const fn into_inner(self) -> mrb_value {
        self.0
    }

    pub const fn is_array(&self) -> bool {
        let Value(ref value) = *self;
        value.tt == MRB_TT_ARRAY
    }

    // TODO: Switch to `const fn` once `match` statements are stabilized. See:
    // https://github.com/rust-lang/rust/issues/49146
    pub fn is_bool(&self) -> bool {
        match self.0.tt {
            MRB_TT_FALSE | MRB_TT_TRUE => true,
            _ => false,
        }
    }

    pub const fn is_exception(&self) -> bool {
        let Value(ref value) = *self;
        value.tt == MRB_TT_EXCEPTION
    }

    pub const fn is_fixnum(&self) -> bool {
        let Value(ref value) = *self;
        value.tt == MRB_TT_FIXNUM
    }

    pub const fn is_float(&self) -> bool {
        let Value(ref value) = *self;
        value.tt == MRB_TT_FLOAT
    }

    // TODO: Switch to `const fn` once short-circuiting boolean comparisons operations and `unsafe`
    // calls are stabilized. See: https://github.com/rust-lang/rust/issues/49146
    pub fn is_nil(&self) -> bool {
        let Value(ref value) = *self;
        unsafe { mrb_ext_is_value_nil(*value) == 1 }
    }

    pub const fn is_object(&self) -> bool {
        let Value(ref value) = *self;
        value.tt == MRB_TT_OBJECT
    }

    pub const fn is_hash(&self) -> bool {
        let Value(ref value) = *self;
        value.tt == MRB_TT_HASH
    }

    pub const fn is_string(&self) -> bool {
        let Value(ref value) = *self;
        value.tt == MRB_TT_STRING
    }

    pub const fn is_symbol(&self) -> bool {
        let Value(ref value) = *self;
        value.tt == MRB_TT_SYMBOL
    }

    pub const fn is_undef(&self) -> bool {
        let Value(ref value) = *self;
        value.tt == MRB_TT_UNDEF
    }
}

impl Debug for Value {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        let Value(ref inner) = *self;

        let mut debug = fmt.debug_struct(stringify!(Value));
        debug.field("type", &inner.tt);

        let mut value: &dyn Debug = match inner.tt {
            MRB_TT_FALSE => &false,
            MRB_TT_FIXNUM => unsafe { &inner.value.i },
            // NOTE: Cannot check the float value in `val.value.f` here because it will not work
            // with `MRB_WORD_BOXING`.
            MRB_TT_FLOAT => unsafe { &inner.value.f },
            MRB_TT_TRUE => &true,
            MRB_TT_UNDEF => &"undef",
            _ => &"<unknown>",
        };

        if unsafe { mrb_ext_is_value_nil(*inner) } == 1 {
            value = &"nil";
        }

        debug.field("value", value).finish()
    }
}
