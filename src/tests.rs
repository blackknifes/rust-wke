use super::common::Rect;
use super::error::Result;
use crate::webview;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use wke::javascript;

#[cfg(test)]
mod wke {
    pub use crate::*;
}

#[cfg(target_arch = "x86")]
const DLL_NAME: &str = "miniblink_x32.dll";
#[cfg(target_arch = "x86_64")]
const DLL_NAME: &str = "miniblink_x64.dll";

#[cfg(debug_assertions)]
const CONFIG_DIR: &str = "debug";
#[cfg(not(debug_assertions))]
const CONFIG_DIR: &str = "release";

lazy_static! {
    static ref DEV_TOOLS_PATH: String = std::env::current_dir()
        .expect("cannot get current dir")
        .join("wke-sys")
        .join("wke")
        .join("front_end")
        .join("inspector.html")
        .to_str()
        .expect("cannot get dll path")
        .to_owned();
}

fn get_dll_path() -> String {
    std::env::current_dir()
        .expect("cannot get current dir")
        .join("target")
        .join(CONFIG_DIR)
        .join("bin")
        .join(DLL_NAME)
        .to_str()
        .expect("cannot get dll path")
        .to_owned()
}

#[derive(Serialize, Deserialize)]
struct Test {
    name: String,
}

#[wke::command]
async fn test(name: String) -> Result<Test> {
    println!("test: {}", name);
    Ok(Test { name })
}

async fn test_jsbind() -> Result<()> {
    javascript::register("test", test);

    let webview = webview::WebView::popup(Rect {
        x: 0,
        y: 0,
        width: 800,
        height: 600,
    });

    let devtools = webview.show_devtools(&DEV_TOOLS_PATH).await?;
    webview.delegates().on_window_destroy.add(move || {
        devtools.close();
        Ok(())
    });

    webview.load_url("https://baidu.com");
    webview.show();
    webview.wait_close().await;
    Ok(())
}

#[cfg(test)]
#[super::main(dll = get_dll_path)]
async fn main() -> crate::error::Result<()> {
    // 将报告写出到文件
    // std::fs::write("target/apis.txt", wke::report())?;

    test_jsbind().await?;

    wke::exit();
    return Ok(());
}

#[test]
fn test_wke() -> std::result::Result<(), Box<dyn std::error::Error>> {
    main()
}
