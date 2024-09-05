mod javascript;

use super::common::Rect;
use super::error::Result;
use crate::javascript::{FromJs, JsValue};
use crate::webview;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use wke::javascript::query::emit;
use wke::javascript::{query::on_query, Context, IntoJs};
use wke::Error;
use wke_jsbind::{FromJs, IntoJs};

#[cfg(test)]
mod wke {
    pub use crate::*;
}

#[derive(Default, FromJs, IntoJs)]
struct AutoJs {
    #[js]
    val: i32,
    #[js]
    str: String,
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

async fn test_jsbind() -> Result<()> {
    let webview = webview::WebView::popup(Rect {
        x: 0,
        y: 0,
        width: 800,
        height: 600,
    });
    webview
        .delegates()
        .on_did_create_script_context
        .add(|frame| {
            let ctx = Context::current()?;
            let log = JsValue::bind_function(
                "log",
                javascript::JsFunction::from(|args: &[&JsValue]| {
                    if args.len() == 0 {
                        return ();
                    }
                    if let Ok(str) = String::from_js(args[0]) {
                        println!("js log: {}", str);
                    }
                    ()
                }),
            )?;
            ctx.global().set("log", &log)?;

            ctx.global().set(
                "TestGetterSetter",
                &JsValue::bind_object("test", javascript::TestGetterSetter::default())?,
            )?;
            let auto_js = AutoJs {
                val: 50,
                str: "auto_js".to_owned(),
            };
            ctx.global().set("auto_js", &auto_js.into_js()?)?;

            ctx.eval("window.test(\"test interface\")")?;
            ctx.eval(
                r#"(function() {
                window.TestGetterSetter.number = 5;
                window.TestGetterSetter.string = "string";

                window.log("TestGetterSetter::number=" + window.TestGetterSetter.number);
                window.log("TestGetterSetter::string=" + window.TestGetterSetter.string);
                window.log("TestGetterSetter::const=" + window.TestGetterSetter.const_value);
            })()"#,
            )?;

            ctx.eval(
                r#"
                window.log("auto_js=" + JSON.stringify(window.auto_js));
            "#,
            )?;
            let ret = ctx.eval("return {val: 50, str: \"test\"}")?;
            let auto_js = AutoJs::from_js(&ret)?;
            assert_eq!(auto_js.val, 50);
            assert_eq!(auto_js.str, "test");

            let webview = frame.webview()?;
            webview.close();
            Result::Ok(())
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

async fn test_mb_query() -> Result<()> {
    on_query("test", |query: TestQuery| async move {
        println!("query: str={}, val={}", query.str, query.val);
        emit("test", query)?;

        Result::<()>::Err(Error::NotImplement)
    });

    let webview = webview::WebView::popup(Rect {
        x: 0,
        y: 0,
        width: 800,
        height: 600,
    });

    webview.delegates().on_did_create_script_context.add(|_| {
        let ctx = Context::current()?;
        ctx.eval("console.log('on_did_create_script_context')")?;
        super::javascript::query::on_context_created(ctx.clone())?;
        Ok(())
    });

    webview.delegates().on_will_release_script_context.add(|_| {
        let ctx = Context::current()?;
        super::javascript::query::on_context_released(ctx)?;
        Ok(())
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

#[derive(Serialize, Deserialize)]
struct TestQuery {
    str: String,
    val: i32,
}

#[cfg(test)]
#[super::main(dll = get_dll_path)]
async fn main() -> crate::error::Result<()> {
    // 将报告写出到文件
    // std::fs::write("target/apis.txt", wke::report())?;

    test_jsbind().await?;
    test_mb_query().await?;

    wke::exit();
    return Ok(());
}

#[test]
fn test_wke() -> std::result::Result<(), Box<dyn std::error::Error>> {
    main()
}
