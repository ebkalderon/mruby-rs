#include <mruby.h>
#include <mruby/value.h>

mrb_value mrb_ext_bool_value(mrb_bool boolean) {
    return mrb_bool_value(boolean);
}

mrb_value mrb_ext_cptr_value(struct mrb_state *mrb, void *p) {
    return mrb_cptr_value(mrb, p);
}

mrb_value mrb_ext_fixnum_value(mrb_int i) {
    return mrb_fixnum_value(i);
}

#ifndef MRB_WITHOUT_FLOAT
mrb_value mrb_ext_float_value(struct mrb_state *mrb, mrb_float f) {
    return mrb_float_value(mrb, f);
}
#endif

mrb_value mrb_ext_nil_value() {
    return mrb_nil_value();
}

mrb_value mrb_ext_symbol_value(mrb_sym i) {
    return mrb_symbol_value(i);
}

mrb_value mrb_ext_undef_value() {
    return mrb_undef_value();
}
