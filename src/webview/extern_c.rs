use super::{Cookie, WebView};
use crate::{
    common::{handle::HandleResult, InvokeFuture, Rect},
    error::Result,
    utils::{from_cstr_ptr, from_wkestring, set_wkestring},
    webframe::WebFrame,
    webview::NavigationType,
};
use std::ptr::null_mut;
use wke_sys::{
    utf8, wkeConsoleCallback, wkeConsoleLevel, wkeDraggableRegion, wkeLoadUrlBeginCallback,
    wkeLoadUrlEndCallback, wkeLoadUrlFinishCallback, wkeLoadingFinishCallback, wkeLoadingResult,
    wkeMediaLoadInfo, wkeNavigationType, wkeNetJob, wkeOnPrintCallback, wkeRect, wkeString,
    wkeWebFrameHandle, wkeWebView, wkeWindowFeatures, HDC,
};

pub(crate) extern "C" fn on_show_dev_tools(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
) {
    unsafe {
        let result = if webview.is_null() {
            Result::Err(crate::error::Error::InvalidReference)
        } else {
            Result::Ok(WebView::from_native(webview))
        };
        InvokeFuture::from_raw(_param).ready(result);
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
    webview.delegates().window_destroy_delegate.emit();
    webview.inner.borrow_mut().webview = null_mut();
}

pub(crate) extern "C" fn on_caret_changed(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
    r: *const wkeRect,
) {
    unsafe {
        if let Ok(webview) = WebView::from_native(webview) {
            webview
                .delegates()
                .caret_changed_delegate
                .emit(Rect::from_native(r.as_ref().unwrap()));
        }
    }
}

pub(crate) extern "C" fn on_mouse_over_url_changed(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
    url: wkeString,
) {
    let webview = WebView::detach_webview(webview).unwrap();
    webview
        .delegates()
        .mouse_over_url_changed_delegate
        .emit(&from_wkestring(url));
}

pub(crate) extern "C" fn on_title_changed(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
    title: wkeString,
) {
    let webview = WebView::detach_webview(webview).unwrap();
    webview
        .delegates()
        .title_changed_delegate
        .emit(&from_wkestring(title));
}

pub(crate) extern "C" fn on_url_changed(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
    url: wkeString,
) {
    let webview = WebView::detach_webview(webview).unwrap();
    webview
        .delegates()
        .url_changed_delegate
        .emit(&from_wkestring(url));
}

pub(crate) extern "C" fn on_frame_url_changed(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
    frame: wkeWebFrameHandle,
    url: wkeString,
) {
    let frame = WebFrame::from_native(webview, frame);
    let webview = WebView::detach_webview(webview).unwrap();
    webview
        .delegates()
        .frame_url_changed_delegate
        .emit(&frame, &from_wkestring(url));
}

pub(crate) extern "C" fn on_alert_box(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
    msg: wkeString,
) {
    let webview = WebView::detach_webview(webview).unwrap();
    webview
        .delegates()
        .dialog_delegate
        .emit(&mut super::DialogType::Alert, &from_wkestring(msg));
}

pub(crate) extern "C" fn on_confirm_box(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
    msg: wkeString,
) -> bool {
    let webview = WebView::detach_webview(webview).unwrap();
    let mut result = HandleResult::default();
    webview.delegates().dialog_delegate.emit(
        &mut super::DialogType::Confirm(&mut result),
        &from_wkestring(msg),
    );

    result.unwrap_or(false)
}

pub(crate) extern "C" fn on_prompt_box(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
    msg: wkeString,
    default_result: wkeString,
    wke_result: wkeString,
) -> bool {
    let webview = WebView::detach_webview(webview).unwrap();
    let mut result = HandleResult::default();
    webview
        .delegates()
        .dialog_delegate
        .emit(&mut super::DialogType::Prompt(&mut result), &from_wkestring(msg));
    let (ret, result) = result.unwrap_or_with(Default::default);
    set_wkestring(default_result, &result.default);
    set_wkestring(wke_result, &result.result);
    ret
}

pub(crate) extern "C" fn on_navigation(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
    navigation_type: wkeNavigationType,
    url: wkeString,
) -> bool {
    let webview = WebView::detach_webview(webview).unwrap();

    let mut result = HandleResult::default();
    webview.delegates().navigation_delegate.emit(
        NavigationType::from_native(navigation_type),
        &from_wkestring(url),
        &mut result,
    );
    result.unwrap_or(false)
}

pub(crate) extern "C" fn on_create_view(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
    navigationType: wkeNavigationType,
    url: wkeString,
    windowFeatures: *const wkeWindowFeatures,
) -> wkeWebView {
    let webview = WebView::detach_webview(webview).unwrap();
    todo!()
}

pub(crate) extern "C" fn on_document_ready(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
) {
    let webview = WebView::detach_webview(webview).unwrap();
    todo!()
}

pub(crate) extern "C" fn on_frame_document_ready(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
    frameId: wkeWebFrameHandle,
) {
    let webview = WebView::detach_webview(webview).unwrap();
    todo!()
}

pub(crate) extern "C" fn on_loading_finish(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
    url: wkeString,
    result: wkeLoadingResult,
    failedReason: wkeString,
) {
    let webview = WebView::detach_webview(webview).unwrap();
    todo!()
}

pub(crate) extern "C" fn on_console(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
    level: wkeConsoleLevel,
    message: wkeString,
    sourceName: wkeString,
    sourceLine: ::std::os::raw::c_uint,
    stackTrace: wkeString,
) {
    let webview = WebView::detach_webview(webview).unwrap();
    todo!()
}

pub(crate) extern "C" fn on_load_url_begin(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
    url: *const utf8,
    job: wkeNetJob,
) -> bool {
    let webview = WebView::detach_webview(webview).unwrap();
    todo!()
}

pub(crate) extern "C" fn on_load_url_end(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
    url: *const utf8,
    job: wkeNetJob,
    buf: *mut ::std::os::raw::c_void,
    len: ::std::os::raw::c_int,
) {
    let webview = WebView::detach_webview(webview).unwrap();
    todo!()
}

pub(crate) extern "C" fn on_load_url_headers_received(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
    url: *const utf8,
    job: wkeNetJob,
) {
    let webview = WebView::detach_webview(webview).unwrap();
    todo!()
}

pub(crate) extern "C" fn on_load_url_finish(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
    url: *const utf8,
    job: wkeNetJob,
    len: ::std::os::raw::c_int,
) {
    let webview = WebView::detach_webview(webview).unwrap();
    todo!()
}

pub(crate) extern "C" fn on_load_url_fail(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
    url: *const utf8,
    job: wkeNetJob,
) {
    let webview = WebView::detach_webview(webview).unwrap();
    todo!()
}

pub(crate) extern "C" fn on_did_create_script_context(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
    frameId: wkeWebFrameHandle,
    context: *mut ::std::os::raw::c_void,
    extensionGroup: ::std::os::raw::c_int,
    worldId: ::std::os::raw::c_int,
) {
    let webview = WebView::detach_webview(webview).unwrap();
    todo!()
}

pub(crate) extern "C" fn on_will_release_script_context(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
    frameId: wkeWebFrameHandle,
    context: *mut ::std::os::raw::c_void,
    worldId: ::std::os::raw::c_int,
) {
    let webview = WebView::detach_webview(webview).unwrap();
    todo!()
}

pub(crate) extern "C" fn on_window_closing(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
) -> bool {
    let webview = WebView::detach_webview(webview).unwrap();
    todo!()
}

pub(crate) extern "C" fn on_draggable_regions_changed(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
    rects: *const wkeDraggableRegion,
    rectCount: ::std::os::raw::c_int,
) {
    let webview = WebView::detach_webview(webview).unwrap();
    todo!()
}

pub(crate) extern "C" fn on_will_media_load(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
    url: *const ::std::os::raw::c_char,
    info: *mut wkeMediaLoadInfo,
) {
    let webview = WebView::detach_webview(webview).unwrap();
    todo!()
}

pub(crate) extern "C" fn on_print(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
    frameId: wkeWebFrameHandle,
    printParams: *mut ::std::os::raw::c_void,
) {
    let webview = WebView::detach_webview(webview).unwrap();
    todo!()
}
