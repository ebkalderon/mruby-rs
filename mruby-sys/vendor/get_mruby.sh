#!/usr/bin/env bash

# Copyright (C) 2018-2019 Eyal Kalderon
# Copyright (C) 2016 Dragoș Tiselice
#
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.
#
# This script generates mruby-out.tar which is the mruby repository compiled to
# C files.
#
# Dependencies:
# * bison
# * compiler, linker, and archiver
# * curl
# * ruby
# * unzip

set -Euo pipefail

VERSION=latest
BASE_DIR="$(cd "$(dirname "$0")" && pwd)"
TEMP_DIR="$(mktemp -dt 'mruby')"

trap -- "rm -rf '${TEMP_DIR}'" EXIT

cd "${TEMP_DIR}" || exit 1

if [ "${VERSION}" == 'latest' ]; then
  curl -Lo latest.zip https://github.com/mruby/mruby/zipball/master
else
  curl -LO "https://github.com/mruby/mruby/archive/${VERSION}.zip"
fi

unzip -u "${VERSION}.zip"
mv mruby-* "mruby-${VERSION}"
mkdir -p mruby-out/src/mrblib
mkdir -p mruby-out/src/mrbgems
cd "mruby-${VERSION}" || exit 1

# minirake compiles the compiler and rb files to C.

./minirake

# Adds all .h files from include.

cp -R include ../mruby-out

# Adds src and C-compiled mrblib.

cp src/*.c ../mruby-out/src
cp src/*.h ../mruby-out/include
cp build/host/mrblib/mrblib.c ../mruby-out/src/mrblib/mrblib.c

# Removes incompatible files.

find mrbgems -type f ! -name "*.c" -and ! -name "*.h" -and ! -name "*.def" -and ! -name "*.cstub" -delete
find mrbgems -type d -empty -delete
find build/host/mrbgems -type f ! -name "*.c" -and ! -name "*.h" -delete
find build/host/mrbgems -type d -empty -delete

# Removes incompatible gems.

rm -rf mrbgems/mruby-bin*
rm -rf build/host/mrbgems/mruby-bin*

rm -rf mrbgems/mruby-test
rm -rf build/host/mrbgems/mruby-test

# Copies some .h files from gems.

cp -R mrbgems/mruby-io/include/mruby/* ../mruby-out/include/mruby
cp -R mrbgems/mruby-time/include/mruby/* ../mruby-out/include/mruby

# Copies all gems.

cp -R mrbgems/* ../mruby-out/src/mrbgems
cp -R build/host/mrbgems/* ../mruby-out/src/mrbgems

cd ..

# Generate FFI bindings with Bindgen.

WHITELIST='_mrb*|MRB*|MRUBY*|mrb*|mruby*'
SWITCHES=(
  --blacklist-type 'FILE'
  --opaque-type 'FILE'
  --whitelist-type "${WHITELIST}"
  --whitelist-function "${WHITELIST}"
  --whitelist-var "${WHITELIST}"
  --generate-inline-functions
  --distrust-clang-mangling
  --no-prepend-enum-name
  --impl-debug
)

while IFS=$'\n' read -r line; do
  IFS=',' read -ra data <<< "${line}"
  file_name="${data[0]}"
  read -ra defines <<< "${data[1]}"
  bindgen "${SWITCHES[@]/#/}" \
    "${BASE_DIR}/wrapper.h" -- "${defines[@]/#/}" \
    -I mruby-out/include/ > "${BASE_DIR}/../src/${file_name}.rs"
done <<< "$(ruby "${BASE_DIR}/configure.rb")"

tar -cf "${BASE_DIR}/mruby-out.tar" mruby-out
