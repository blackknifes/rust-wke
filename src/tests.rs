use super::error::{Error, Result};
use super::{
    common::Rect,
    webview::{self, DebugConfig},
};
use crate::javascript::{JsDelegate, JsValue};
use lazy_static::lazy_static;
use std::time::Duration;

#[cfg(test)]
mod wke {
    pub use crate::init;
    pub use crate::run_once;
    pub use crate::RunOnceFlag;
}

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

pub struct TestCaller;
impl JsDelegate for TestCaller {
    fn has_get(&self) -> bool {
        false
    }

    fn has_set(&self) -> bool {
        false
    }

    fn has_call(&self) -> bool {
        true
    }

    fn get(&mut self, _name: &str) -> Result<crate::javascript::JsValuePerssist> {
        Err(Error::NotImplement)
    }

    fn set(&mut self, _name: &str, _val: &JsValue) -> Result<()> {
        Err(Error::NotImplement)
    }

    fn call(&mut self, args: &[&JsValue]) -> Result<crate::javascript::JsValuePerssist> {
        println!("TestCaller: 测试");
        println!("参数长度: {}", args.len());
        if args.len() > 0 {
            println!("参数1: {}", args[0].to_string()?);
        }
        JsValue::undefined()
    }

    fn finalize(&mut self) -> Result<()> {
        Ok(())
    }
}

async fn test_popup() -> Result<()> {
    let webview = webview::WebView::popup(Rect {
        x: 0,
        y: 0,
        width: 800,
        height: 600,
    });
    webview.load_url("https://baidu.com");
    // webview.set_debug_config(DebugConfig::ShowDevTools(DEV_TOOLS_PATH.clone()))?;
    webview
        .delegates()
        .on_did_create_script_context
        .add(|frame| {
            let ctx = frame.get_context();
            let _holder = ctx.enter();
            let func = JsValue::bind_function("test", TestCaller {})?;
            ctx.global().set("test", &func)?;
            Ok(())
        });
    webview.show();

    let devtools = webview.show_devtools(&DEV_TOOLS_PATH).await?;
    webview.delegates().on_window_destroy.add(move || {
        devtools.close();
        super::exit();
        Ok(())
    });

    Ok(())
}

#[test]
fn test_tokio() -> std::result::Result<(), Box<dyn std::error::Error>> {
    main()?;
    Ok(())
}

#[cfg(test)]
#[super::main(dll = get_dll_path)]
async fn main() -> crate::error::Result<()> {
    // std::fs::write("target/apis.txt", wke::report())?;
    test_popup().await?;
    return Ok(());
}
