use std::ptr::null_mut;

use super::{Cookie, WebView};
use crate::{error::Result, utils::from_cstr_ptr, wke::common::InvokeFuture};
use wke_sys::wkeWebView;

pub(crate) extern "C" fn on_show_dev_tools(
    webview: wkeWebView,
    param: *mut ::std::os::raw::c_void,
) {
    unsafe {
        let result = if webview.is_null() {
            Result::Err(crate::error::Error::InvalidReference)
        } else {
            Result::Ok(WebView::from_native(webview))
        };
        InvokeFuture::from_raw(param).ready(result);
    }
}

pub(crate) struct FindCookie {
    name: String,
    pub(crate) cookie: Option<Cookie>,
}

impl FindCookie {
    pub(crate) fn new(name: String) -> Self {
        Self { name, cookie: None }
    }
}

pub(crate) extern "C" fn find_cookie_on_visit_all_cookie(
    params: *mut ::std::os::raw::c_void,
    name: *const ::std::os::raw::c_char,
    value: *const ::std::os::raw::c_char,
    domain: *const ::std::os::raw::c_char,
    path: *const ::std::os::raw::c_char,
    secure: ::std::os::raw::c_int,
    http_only: ::std::os::raw::c_int,
    expires: *mut ::std::os::raw::c_int,
) -> bool {
    unsafe {
        if let Some(find_cookie) = (params as *mut FindCookie).as_mut() {
            if let Ok(name) = from_cstr_ptr(name) {
                if find_cookie.name.eq(&name) {
                    if let Ok(cookie) =
                        Cookie::from_native(name, value, domain, path, secure, http_only, expires)
                    {
                        find_cookie.cookie.replace(cookie);
                        return false;
                    }
                }
            }
            true
        } else {
            false
        }
    }
}

pub(crate) extern "C" fn on_window_destroy(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
) {
    let webview = WebView::detach_webview(webview).unwrap();
    webview.inner.borrow().on_destroy.emit();
    webview.inner.borrow_mut().webview = null_mut();
}
