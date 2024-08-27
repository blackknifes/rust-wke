use bindgen;
use std::{env, path::PathBuf};

const WKE_HEADER: &str = "wke/wke.h";
const WKE_SOURCE: &str = "wke/wke.c";
const WKE_LIB_NAME: &str = "wke";

fn main() {
    println!("cargo:rerun-if-changed={}", WKE_HEADER);
    println!("cargo:rerun-if-changed={}", WKE_SOURCE);

    let cur_dir = std::env::current_dir().expect("cannot get current dir");
    let out_path = PathBuf::from(env::var("OUT_DIR").expect("cannot get env $OUT_DIR"));

    cc::Build::new()
        .include(cur_dir.join("wke"))
        .file(cur_dir.join(WKE_SOURCE))
        .compile(WKE_LIB_NAME);

    println!(
        "cargo:rustc-link-search=native={}",
        out_path.to_str().unwrap()
    );
    println!("cargo:rustc-link-lib={}", WKE_LIB_NAME);
    println!("cargo:rustc-link-lib=User32");

    bindgen::Builder::default()
        .header(WKE_HEADER)
        .allowlist_file(WKE_HEADER)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
