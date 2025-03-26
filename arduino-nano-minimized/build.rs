//! This build script copies the `memory.x` file from the crate root into
//! a directory where the linker can always find it at build time.
//! For many projects this is optional, as the linker always searches the
//! project root directory -- wherever `Cargo.toml` is. However, if you
//! are using a workspace or have a more complicated build setup, this
//! build script becomes required. Additionally, by requesting that
//! Cargo re-run the build script whenever `memory.x` is changed,
//! updating `memory.x` ensures a rebuild of the application with the
//! new memory settings.

use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    // Put `memory.x` in our output directory and ensure it's
    // on the linker search path.
    // let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    // let out = std::path::absolute(".").unwrap();
    let out = std::fs::canonicalize("..").unwrap();
    // let out = &PathBuf::from(".").;
    fn move_file(out: &PathBuf, file_name: &str, bytes: &[u8]) {
        File::create(out.join(file_name))
            .unwrap()
            .write_all(bytes)
            .unwrap();
        println!("cargo:rerun-if-changed={}", file_name);
    }
    fs::write("C:/Users/Gabriel/Desktop/abc.txt", format!("{:?}\n{:?}\n{:?}", out, out.display(), std::env::current_dir().unwrap())).unwrap();
    move_file(&out, "./linker-script", include_bytes!("./linker-script"));
    move_file(&out, "interrupt_vector.S", include_bytes!("./interrupt_vector.S"));
    move_file(&out, "initialize_memory.S", include_bytes!("./initialize_memory.S"));
    // println!("cargo:rustc-link-search={}", out.display());
    // By default, Cargo will re-run a build script whenever
    // any file in the project changes. By specifying `memory.x`
    // here, we ensure the build script is only re-run when
    // `memory.x` is changed.
}
