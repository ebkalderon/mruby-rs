extern crate cc;
extern crate walkdir;

use std::env;
use std::path::Path;

use walkdir::{DirEntry, WalkDir};

const MRUBY_INCLUDE: &str = "mruby/include";
const MRUBY_SRC_CORE: &str = "mruby/src";
const MRUBY_SRC_MRBGEN: &str = "mruby/mrbgen";
const MRUBY_SRC_MRBLIB: &str = "mruby/mrblib";

fn main() {
    if cfg!(all(feature = "static", feature = "dynamic")) {
        panic!("Cannot build `static` and `dynamic` features at the same time.")
    } else if cfg!(not(any(feature = "static", feature = "dynamic"))) {
        panic!("Please select either the `static` or `dynamic` feature.")
    }

    let mut build = cc::Build::new();
    build.warnings(false).flag_if_supported("-std=gnu99").flag_if_supported("-O3");

    if cfg!(feature = "debug") {
        build.define("MRB_DEBUG", None);
    }

    if cfg!(feature = "disable-stdio") {
        build.define("MRB_DISABLE_STDIO", None);
    }

    if cfg!(feature = "use-floats") {
        build.define("MRB_USE_FLOAT", None);
    } else {
        build.define("MRB_WITHOUT_FLOAT", None);
    }

    if cfg!(feature = "utf8") {
        build.define("MRB_UTF8_STRING", None);
    }

    walk_files(&mut build, MRUBY_SRC_CORE);
    // walk_files(&mut build, MRUBY_SRC_MRBGEN);
    // walk_files(&mut build, MRUBY_SRC_MRBLIB);

    let include_dir = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join(MRUBY_INCLUDE);
    build.include(include_dir);
    let include_dir2 = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join(MRUBY_SRC_CORE);
    build.include(include_dir2);
    
    println!("cargo:rustc-link-lib=m");

    if cfg!(feature = "static") {
        build.static_flag(true).compile("mruby");
        // println!("cargo:rustc-link-search=native={}", env::var("OUT_DIR").unwrap());
        // println!("cargo:rustc-link-lib=static=mruby");
    } else if cfg!(feature = "dynamic") {
        build.compile("mruby");
        // println!("cargo:rustc-link-search=native={}", env::var("OUT_DIR").unwrap());
        // println!("cargo:rustc-link-lib=mruby");
    }
}

fn is_c_file(entry: &DirEntry) -> bool {
    entry.path().extension().map(|e| e == "c").unwrap_or(false)
}

fn walk_files(build: &mut cc::Build, root: &str) {
    let src_dir = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join(root);
    let sources = WalkDir::new(src_dir)
        .into_iter()
        .filter_entry(|e| e.file_type().is_dir() || is_c_file(e))
        .flat_map(|e| e.ok());

    for file in sources {
        if is_c_file(&file) {
            build.file(file.path());
        }
    }
}
