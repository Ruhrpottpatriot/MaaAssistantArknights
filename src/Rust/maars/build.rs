#![allow(deprecated)]

use bindgen::{Builder, CargoCallbacks};
use std::{env, path::PathBuf};

fn main() {
    // Make cargo look for shared libraries in the folder two levels up
    println!("cargo:rustc-link-search=native=./maars");
    println!("cargo:rustc-link-lib=MaaCore");

    // Assume that the header file for rust-bindgen is located in the main
    // MAA project folder, which is three levels up from the current build file
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let include_dir = manifest_dir.join("../../../include");
    let caller_path = include_dir.join("AsstCaller.h");

    let bindings = Builder::default()
        .header(caller_path.to_string_lossy())
        .parse_callbacks(Box::new(CargoCallbacks::new()))

        // Only allow the AsstCaller.h to be parsed
        .allowlist_file(".*AsstCaller.h")

        // Mark the AsstExtAPI as opaque blob
        .no_copy("AsstExtAPI")
        .no_debug("AsstExtAPI")

        // Generate bindings
        .generate()
        .expect("Unable to generate bindings");

    // Place in ./src/bind.rs
    let out_path = manifest_dir.join("src/bind.rs");
    bindings
        .write_to_file(out_path)
        .expect("Could not write bindings to file");
}
