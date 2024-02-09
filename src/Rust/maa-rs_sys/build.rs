// build.rs

fn main() {
    // Make cargo look for shared libraries in the folder two levels up    
    println!("cargo:rustc-link-search=native=../..");
    println!("cargo:rustc-link-lib=MaaCore");
}
