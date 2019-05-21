//! Useful declarative macros for internal use.

macro_rules! replace_expr {
    ( $_first:tt $sub:expr ) => {
        $sub
    };
}

#[macro_use]
macro_rules! count_tokens {
    ( $($elem:tt)* ) => {
        <[()]>::len(&[$(replace_expr!($elem ())),*])
    };
}
