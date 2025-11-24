use std::env;
use std::path::PathBuf;

use cmake::Config;

fn main() {
    let libdir_path = PathBuf::from("vendor")
        // Canonicalize the path as `rustc-link-search` requires an absolute
        // path.
        .canonicalize()
        .expect("cannot canonicalize path");

    let headers_path = libdir_path.join("include/libobsensor/ObSensor.h");
    let headers_path_str = headers_path.to_str().expect("Path is not a valid string");

    let build_destination = Config::new(&libdir_path)
        .define("OB_BUILD_EXAMPLES", "OFF")
        .define("OB_BUILD_TESTS", "OFF")
        .define("OB_BUILD_DOCS", "OFF")
        .define("OB_BUILD_TOOLS", "OFF")
        .build();

    println!(
        "cargo:rustc-link-search=native={}",
        build_destination.join("lib").display()
    );
    println!("cargo:rustc-link-lib=dylib=OrbbecSDK");
    println!("cargo:rerun-if-changed={}", headers_path_str);

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header(headers_path_str)
        .clang_arg(format!("-I{}", libdir_path.join("include/").display()))
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs");
    bindings
        .write_to_file(out_path)
        .expect("Couldn't write bindings!");
}
