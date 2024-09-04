use bindgen;
use zip::ZipArchive;
use std::{
    env, fs::{remove_file, OpenOptions}, io::{BufWriter, Read, Seek, Write}, path::{Path, PathBuf}
};

const WKE_HEADER: &str = "wke/wke.h";
const WKE_SOURCE: &str = "wke/wke.c";

#[cfg(feature = "enable_report")]
const WKE_REPORT_HEADER: &str = "wke/wke_report.h";
#[cfg(feature = "enable_report")]
const WKE_REPORT_SOURCE: &str = "wke/wke_report.c";


const DOWNLOAD_URL: &str = 
    "https://mirror.ghproxy.com/?q=https%3A%2F%2Fgithub.com%2Fweolar%2Fminiblink49%2Freleases%2Fdownload%2F20230412%2Fminiblink-20230412.zip";
const DOWNLOAD_URL2: &str =
    "https://github.com/weolar/miniblink49/releases/download/20230412/miniblink-20230412.zip";

const EXTRACT_X32_PATH: &str = "release/miniblink_4975_x32.dll";
const EXTRACT_X64_PATH: &str = "release/miniblink_4975_x64.dll";

struct OnDrop(Option<Box<dyn FnOnce()>>);

impl OnDrop
{
    pub fn from_fn<FN>(func: FN) -> Self
    where
        FN: FnOnce() + 'static
    {
        Self(Some(Box::new(func)))
    }

    pub fn reset(&mut self)
    {
        self.0 = None;
    }
}

impl Drop for OnDrop {
    fn drop(&mut self) {
        if let Some(cb) = self.0.take() {
            cb();
        }
    }
}

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

fn download_dll(output: impl AsRef<Path>, url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let rm_output = output.as_ref().to_path_buf();
    let mut rm_task = OnDrop::from_fn(move || {
        let _ = remove_file(rm_output);
    });

    let client = reqwest::blocking::Client::new();
    let mut response = client.get(url).send()?;

    if !response.status().is_success() {
        return Err("response is not success".into());
    }

    let file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(output.as_ref())
        .expect(&format!("open file \"{:?}\" failed", output.as_ref()));

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
    rm_task.reset();
    Ok(())
}

fn extract_dll<R: Read + Seek>(archive: &mut ZipArchive<R>, path: &str, output: impl AsRef<Path>) {    
    let mut zip_file = archive
        .by_name(path)
        .expect("cannot find dll file in zip archive");

    let mut output = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output.as_ref())
        .expect(&format!("cannot create {:?}", output.as_ref()));

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
}

// 下载并解压
fn download_and_extract(output: impl AsRef<Path>) {
    let download_path = output.as_ref().join("download.zip");
    let bin_dir = output.as_ref().join("..").join("..").join("..").join("bin");
    let extract_x32_target = bin_dir.join("miniblink_x32.dll");
    let extract_x64_target = bin_dir.join("miniblink_x64.dll");

    // 已下载，直接退出任务
    if extract_x32_target.is_file() && extract_x64_target.is_file() {
        println!("cargo:warning=found bin: [{:?}, {:?}]", extract_x32_target, extract_x64_target);
        return;
    }

    let remove_download_path = download_path.clone();
    let _ondrop = OnDrop::from_fn(|| {
        let _ = remove_file(remove_download_path);
    });

    // 创建bin文件夹
    let _ = std::fs::create_dir_all(&bin_dir);

    // 按备选列表顺序下载
    let urls = [DOWNLOAD_URL, DOWNLOAD_URL2];
    let mut download_result = false;
    for url in urls {
        if let Ok(_) = download_dll(&download_path, url) {
            download_result = true;
            break;
        }
    }
    if !download_result {
        panic!("download zip failed");
    }

    // 解压文件
    let mut archive = ZipArchive::new(OpenOptions::new()
        .read(true).open(download_path).expect("cannot open zip file")).expect("cannot open zip file");
    extract_dll(&mut archive, EXTRACT_X32_PATH, &extract_x32_target);
    extract_dll(&mut archive, EXTRACT_X64_PATH, &extract_x64_target);
    println!("cargo:warning=download miniblink bin to [{:?}, {:?}]", extract_x32_target, extract_x64_target);
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
