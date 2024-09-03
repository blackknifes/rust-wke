use bindgen;
use std::{
    env,
    io::{BufWriter, Read, Write},
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

const DOWNLOAD_URL: &str = 
    "https://mirror.ghproxy.com/?q=https%3A%2F%2Fgithub.com%2Fweolar%2Fminiblink49%2Freleases%2Fdownload%2F20230412%2Fminiblink-20230412.zip";
const DOWNLOAD_URL2: &str =
    "https://github.com/weolar/miniblink49/releases/download/20230412/miniblink-20230412.zip";

#[cfg(target_arch = "x86")]
const EXTRACT_PATH: &str = "release/miniblink_4975_x32.dll";
#[cfg(target_arch = "x86_64")]
const EXTRACT_PATH: &str = "release/miniblink_4975_x64.dll";

fn download_dll(output: impl AsRef<Path>, url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();
    let mut response = client.get(url).send()?;

    if !response.status().is_success() {
        return Err("response is not success".into());
    }

    let dl_path = output.as_ref().join("download.zip");
    let file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(&dl_path)
        .expect(&format!("open file \"{:?}\" failed", dl_path));

    let mut writer = BufWriter::new(file);

    let mut buf = [0; 0x1000];

    loop {
        let read_bytes = response.read(&mut buf).expect("read from response failed");
        if read_bytes == 0 {
            break;
        }
        writer
            .write_all(&buf[..read_bytes])
            .expect("write file failed");
    }
    Ok(())
}

fn extract_dll(output: impl AsRef<Path>) -> PathBuf {
    let dl_path = output.as_ref().join("download.zip");
    let file = std::fs::OpenOptions::new()
        .read(true)
        .open(dl_path)
        .expect("open download file failed");
    let mut archive = zip::ZipArchive::new(file).expect("open zip file failed");
    let mut zip_file = archive
        .by_name(EXTRACT_PATH)
        .expect("cannot find dll file in zip archive");

    let dll_path = output.as_ref().join("miniblink.dll");
    let mut output = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&dll_path)
        .expect(&format!("cannot create {:?}", dll_path));

    let mut buf = [0; 0x4000];

    loop {
        let read_size = zip_file
            .read(&mut buf)
            .expect("read content failed from archive");
        if read_size == 0 {
            break;
        }

        output
            .write_all(&buf[..read_size])
            .expect("write to miniblink.dll failed");
    }

    dll_path.to_path_buf()
}

fn download_and_extract(output: impl AsRef<Path>) {
    let urls = [DOWNLOAD_URL, DOWNLOAD_URL2];
    let mut download_result = false;
    for url in urls {
        if let Ok(_) = download_dll(output.as_ref(), url) {
            download_result = true;
            break;
        }
    }
    if !download_result {
        panic!("download zip failed");
    }

    let dll_path = extract_dll(output.as_ref());

    // 删除下载的文件
    let dl_path = output.as_ref().join("download.zip");
    let _ = std::fs::remove_file(dl_path);
    let bin_dir = output.as_ref().join("..").join("..").join("..").join("bin");
    let output_dll_path = bin_dir.join("miniblink.dll");

    let _ = std::fs::create_dir_all(&bin_dir);
    std::fs::copy(dll_path, &output_dll_path).expect("copy file to bin dir failed");

    println!("cargo:warning=dll={:?}", output_dll_path);
}

fn main() {
    println!("cargo:rustc-link-lib=User32");

    let dir = std::env::current_dir().expect("cannot get current dir");
    let output = PathBuf::from(env::var("OUT_DIR").expect("cannot get env $OUT_DIR"));

    // 下载miniblink.dll
    download_and_extract(&output);

    // 生成bindings.rs
    build_gen(&dir, &output, "wke", WKE_HEADER, WKE_SOURCE, "bindings.rs");

    #[cfg(feature = "enable_report")]
    {
        build_gen(
            &dir,
            &output,
            "wke_report",
            WKE_REPORT_HEADER,
            WKE_REPORT_SOURCE,
            "report_bindings.rs",
        );
    }
}
