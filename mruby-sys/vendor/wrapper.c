#include <mruby.h>
#include <mruby/array.h>
#include <mruby/class.h>
#include <mruby/value.h>

mrb_int mrb_ext_ary_len(mrb_value array) {
    return RARRAY_LEN(array);
}

mrb_value mrb_ext_bool_value(mrb_bool boolean) {
    return mrb_bool_value(boolean);
}

mrb_value mrb_ext_class_value(struct RClass *c) {
    mrb_value value;
    value.value.p = c;
    value.tt = MRB_TT_CLASS;
    return value;
}

mrb_value mrb_ext_cptr_value(struct mrb_state *mrb, void *p) {
    return mrb_cptr_value(mrb, p);
}

mrb_int mrb_ext_fixnum_to_cint(mrb_value num) {
    return mrb_fixnum(num);
}

mrb_value mrb_ext_fixnum_value(mrb_int i) {
    return mrb_fixnum_value(i);
}

#ifndef MRB_WITHOUT_FLOAT
mrb_float mrb_ext_float_to_cfloat(mrb_value flt) {
    return mrb_float(flt);
}

mrb_value mrb_ext_float_value(struct mrb_state *mrb, mrb_float f) {
    return mrb_float_value(mrb, f);
}
#endif

mrb_bool mrb_ext_is_value_nil(mrb_value v) {
    return mrb_nil_p(v);
}

mrb_value mrb_ext_nil_value() {
    return mrb_nil_value();
}

mrb_noreturn void mrb_ext_raise(struct mrb_state *mrb, const char *err, const char *msg) {
    mrb_raise(mrb, mrb_exc_get(mrb, err), msg);
}

mrb_sym mrb_ext_symbol_to_sym(mrb_value sym) {
    return mrb_symbol(sym);
}

mrb_value mrb_ext_symbol_value(mrb_sym i) {
    return mrb_symbol_value(i);
}

mrb_value mrb_ext_undef_value() {
    return mrb_undef_value();
}
