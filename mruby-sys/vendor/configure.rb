#
# Configures the compile-time feature sets available for mruby and generates all
# possible permutations of these features.
#
# ### Example
#
# If there are 2 features `foo` (variants `foo` and `nofoo`) and `bar` (variants
# `bar` and `nobar`), this script will print a set of newline-delimited lines in
# the following format:
#
# ```
# foo_bar,
# foo_nobar, -DDISABLE_Y
# nofoo_bar, -DDISABLE_X
# nofoo_nobar, -DDISABLE_X -DDISABLE_Y
# ```
#
# These lines are meant to be processed by the `get_mruby.sh` script and are
# used to form the output files and command-line defines for `bindgen`,
# respectively.
#

floats = {
    'double' => '',
    'float' => '-DMRB_USE_FLOAT',
    'nofloat' => '-DMRB_WITHOUT_FLOAT',
}

debug = {
    'debug' => '-DMRB_DEBUG -DMRB_ENABLE_DEBUG_HOOK',
    'nodebug' => '',
}

stdio = {
    'stdio' => '',
    'nostdio' => '-DMRB_DISABLE_STDIO',
}

filenames = floats.keys.product(debug.keys.product(stdio.keys)).map {|x| x.flatten}
options = floats.values.product(debug.values.product(stdio.values)).map {|x| x.flatten}
dict = Hash[filenames.zip(options)]

bindgen_data = dict.map { |names, opts| [names.join('_'), opts.join(' ')] }
bindgen_data.each {|name, opts| puts "#{name},#{opts}"}
