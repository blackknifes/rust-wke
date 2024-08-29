use super::WebView;
use crate::{error::Result, wke::common::InvokeFuture};
use wke_sys::wkeWebView;

pub(crate) extern "C" fn on_show_dev_tools(
    webview: wkeWebView,
    param: *mut ::std::os::raw::c_void,
) {
    unsafe {
        InvokeFuture::ready(param, Result::Ok(WebView::from_native(webview)));
    }
}
