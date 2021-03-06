use std::env;
use std::fs::File;
use std::path::Path;

use cc::Build;
use tar::Archive;
use walkdir::{DirEntry, WalkDir};

const MRUBY_ARCHIVE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/vendor/mruby-out.tar");
const MRUBY_INCLUDE_DIR: &str = "mruby-out/include";
const MRUBY_SRC_DIR: &str = "mruby-out/src";

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    let mut archive = Archive::new(File::open(MRUBY_ARCHIVE).unwrap());
    archive.unpack(&out_dir).unwrap();

    let mut build = Build::new();
    build.warnings(false);

    if cfg!(target_endian = "big") {
        build.define("MRB_ENDIAN_BIG", None);
    }

    if cfg!(feature = "debug") {
        build.define("MRB_DEBUG", None);
        build.define("MRB_ENABLE_ALL_SYMBOLS", None);
        build.define("MRB_ENABLE_DEBUG_HOOK", None);
    }

    if cfg!(feature = "disable-floats") {
        build.define("MRB_WITHOUT_FLOAT", None);
    }

    if cfg!(feature = "disable-generational-gc") {
        build.define("MRB_GC_TURN_OFF_GENERATIONAL", None);
    }

    if cfg!(feature = "use-f32") {
        build.define("MRB_USE_FLOAT", None);

        if cfg!(feature = "disable-floats") {
            panic!("Cannot enable `disable-floats` and `use-f32` features together");
        }
    }

    if cfg!(not(feature = "stdio")) {
        build.define("MRB_DISABLE_STDIO", None);
    }

    if cfg!(feature = "utf8") {
        build.define("MRB_UTF8_STRING", None);
    }

    let include_dir = Path::new(&out_dir).join(MRUBY_INCLUDE_DIR);
    build.include(include_dir);

    let src_dir = Path::new(&out_dir).join(MRUBY_SRC_DIR);
    let sources = WalkDir::new(src_dir)
        .into_iter()
        .filter_entry(|e| e.file_type().is_dir() || is_c_file(e))
        .flat_map(|e| e.ok());

    for file in sources {
        if is_c_file(&file) {
            build.file(file.path());
        }
    }

    println!("cargo:rustc-link-lib=m");

    build.file(concat!(env!("CARGO_MANIFEST_DIR"), "/vendor/wrapper.c"));
    build.compile("mruby");
}

fn is_c_file(entry: &DirEntry) -> bool {
    entry.path().extension().map(|e| e == "c").unwrap_or(false)
}
