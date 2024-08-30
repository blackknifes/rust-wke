use bindgen;
use std::{
    env,
    path::{Path, PathBuf},
};

const WKE_HEADER: &str = "wke/wke.h";
const WKE_SOURCE: &str = "wke/wke.c";

#[cfg(feature = "enable_report")]
const WKE_REPORT_HEADER: &str = "wke/wke_report.h";
#[cfg(feature = "enable_report")]
const WKE_REPORT_SOURCE: &str = "wke/wke_report.c";

fn build_gen(
    dir: impl AsRef<Path>,
    output: impl AsRef<Path>,
    name: &str,
    header: &str,
    source: &str,
    binding_name: &str,
) {
    println!("cargo:rerun-if-changed={}", header);
    println!("cargo:rerun-if-changed={}", source);

    cc::Build::new()
        .include(dir.as_ref().join("mb").join("include"))
        .file(dir.as_ref().join(source))
        .compile(name);

    println!("cargo:rustc-link-lib={}", name);

    bindgen::Builder::default()
        .header(header)
        .allowlist_file(header)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect(&format!("Unable to generate bindings: {}", binding_name))
        .write_to_file(output.as_ref().join(binding_name))
        .expect(&format!("Couldn't write bindings: {}!", binding_name));
}

fn main() {
    println!("cargo:rustc-link-lib=User32");

    let dir = std::env::current_dir().expect("cannot get current dir");
    let output = PathBuf::from(env::var("OUT_DIR").expect("cannot get env $OUT_DIR"));
    build_gen(&dir, &output, "wke", WKE_HEADER, WKE_SOURCE, "bindings.rs");
    #[cfg(feature = "enable_report")]
    {
        build_gen(&dir, &output, "wke_report", WKE_REPORT_HEADER, WKE_REPORT_SOURCE, "report_bindings.rs");
    }
}
