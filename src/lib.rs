pub mod error;
pub mod wke;

mod utils;

#[cfg(test)]
mod tests {
    use wke::{
        common::Rect,
        run, run_once,
        webview::{self, DebugConfig},
    };

    use super::*;

    #[cfg(target_os = "windows")]
    #[cfg(target_arch = "x86")]
    const DLL: &str = "miniblink_4975_x32.dll";

    #[cfg(target_os = "windows")]
    #[cfg(target_arch = "x86_64")]
    const DLL: &str = "miniblink_4975_x64.dll";

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

    fn get_dev_tools_path() -> String {
        std::env::current_dir()
            .expect("cannot get current dir")
            .join("wke-sys")
            .join("wke")
            .join("front_end")
            .join("inspector.html")
            .to_str()
            .expect("cannot get dll path")
            .to_owned()
    }

    // #[test]
    // fn test_init() {
    //     init();
    // }

    #[test]
    fn test_popup() {
        init();
        let webview = webview::WebView::popup(Rect {
            x: 0,
            y: 0,
            width: 800,
            height: 600,
        });
        webview.load_url("https://baidu.com");
        webview.set_debug_config(DebugConfig::ShowDevTools(get_dev_tools_path()));
        webview.show();
        webview
            .show_devtools(&get_dev_tools_path())
            .expect("show dev tools failed");

        loop {
            run_once();
        }
    }
}
