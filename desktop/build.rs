use std::env;

fn main() {
    if env::var("TARGET").unwrap() == "x86_64-pc-windows-gnu" {
        let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        println!("cargo:rustc-link-search={}", std::path::Path::new(&dir).join("../../libsdl2/x86_64-pc-windows-gnu").to_str().unwrap());
        // println!("cargo:rustc-link-search=C:\\Users\\Gabriel\\Workspace\\PiScreen\\ripped");
        // println!("cargo:rustc-link-search=./libsdl2-aarch64");
        println!("cargo:rustc-link-lib=SDL2");
    }
    if env::var("TARGET").unwrap() == "x86_64-pc-windows-msvc" {
        let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        println!("cargo:rustc-link-search={}", std::path::Path::new(&dir).join("../../libsdl2/x86_64-pc-windows-msvc").to_str().unwrap());
        // println!("cargo:rustc-link-search=C:\\Users\\Gabriel\\Workspace\\PiScreen\\ripped");
        // println!("cargo:rustc-link-search=./libsdl2-aarch64");
        println!("cargo:rustc-link-lib=SDL2");
    }
}
