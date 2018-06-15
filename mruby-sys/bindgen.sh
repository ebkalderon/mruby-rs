#!/bin/bash

readonly WHITELIST='_mrb*|MRB*|MRUBY*|mrb*|mruby*'

bindgen --whitelist-type "${WHITELIST}" \
  --whitelist-function "${WHITELIST}" \
  --whitelist-var "${WHITELIST}" \
  wrapper.h \
  -- -I mruby/include/ > ./src/ffi.rs
