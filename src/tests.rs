
use std::time::Duration;

use super::*;
use error::Result;
use lazy_static::lazy_static;
use tokio::task::LocalSet;
use wke::{
    common::Rect,
    webview::{self, DebugConfig},
};

#[cfg(target_os = "windows")]
#[cfg(target_arch = "x86")]
const DLL: &str = "miniblink_4975_x32.dll";

#[cfg(target_os = "windows")]
#[cfg(target_arch = "x86_64")]
const DLL: &str = "miniblink_4975_x64.dll";

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
        .join("wke-sys")
        .join("wke")
        .join(DLL)
        .to_str()
        .expect("cannot get dll path")
        .to_owned()
}

async fn test_popup() -> Result<()> {
    let webview = webview::WebView::popup(Rect {
        x: 0,
        y: 0,
        width: 800,
        height: 600,
    });
    webview.load_url("https://baidu.com");
    webview.set_debug_config(DebugConfig::ShowDevTools(DEV_TOOLS_PATH.clone()));
    webview.show();
    let devtools = webview.show_devtools(&DEV_TOOLS_PATH).await?;
    tokio::time::sleep(Duration::from_secs(5)).await;
    devtools.close();
    tokio::time::sleep(Duration::from_secs(5)).await;
    webview.close();
    tokio::time::sleep(Duration::from_secs(2)).await;
    webview.close();
    tokio::time::sleep(Duration::from_secs(2)).await;
    webview.close();
    tokio::time::sleep(Duration::from_secs(2)).await;
    webview.get_main_frame();
    println!("valid: {}", webview.is_valid());
    wke::exit();

    Ok(())
}

#[test]
fn test_tokio() -> std::result::Result<(), Box<dyn std::error::Error>> {
    main()?;
    Ok(())
}

#[wke::main(dll = get_dll_path)]
async fn main() -> crate::error::Result<()> {
    test_popup().await?;
    return Ok(());
}