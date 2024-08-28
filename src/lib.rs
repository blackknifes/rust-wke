pub mod error;
pub mod wke;
mod utils;

#[proc_macro_attribute]
pub fn setup_teardown(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let name = &input.sig.ident;
    let block = &input.block;

    let gen = quote! {
        #[test]
        fn #name() {
            setup();
            let result = std::panic::catch_unwind(|| {
                #block
            });
            teardown();
            if let Err(err) = result {
                std::panic::resume_unwind(err);
            }
        }
    };

    gen.into()
}

#[cfg(test)]
mod tests {
    use ctor::ctor;
    use lazy_static::lazy_static;
    use wke::{
        common::Rect,
        run_once,
        webview::{self, DebugConfig},
    };

    use super::*;

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
            .join(DLL)
            .to_str()
            .expect("cannot get dll path")
            .to_owned();
    };

    #[ctor]
    fn init() {
        let dll = std::env::current_dir()
            .expect("cannot get current dir")
            .join("wke-sys")
            .join("wke")
            .join(DLL)
            .to_str()
            .expect("cannot get dll path")
            .to_owned();

        wke::init(&dll).expect("init failed");
    }

    #[test]
    fn test_popup() {
        let webview = webview::WebView::popup(Rect {
            x: 0,
            y: 0,
            width: 800,
            height: 600,
        });
        webview.load_url("https://baidu.com");
        webview.set_debug_config(DebugConfig::ShowDevTools(DEV_TOOLS_PATH.clone()));
        webview.show();
        webview
            .show_devtools(&DEV_TOOLS_PATH)
            .expect("show dev tools failed");

        loop {
            run_once();
        }
    }

    #[test]
    fn test_tokio() -> crate::error::Result<()> {
        let runtime = tokio::runtime::Runtime::new().expect("");
        runtime.block_on(async move {
            tokio::fs::write("D:/test.txt", "test").await?;
            crate::error::Result::Ok(())
        })?;
        Ok(())
    }
}
