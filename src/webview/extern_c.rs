use super::{Cookie, LoadingResult, MediaInfo, WebView};
use crate::{
    common::{handle::HandleResult, InvokeFuture, Rect},
    error::Result,
    net::{Job, JobBuf},
    utils::{from_cstr_ptr, from_wkestring, set_wkestring},
    webframe::WebFrame,
    webview::{ConsoleLevel, DraggableRegion, NavigationType, WindowFeature},
};
use std::ptr::null_mut;
use wke_sys::*;

pub(crate) extern "C" fn on_show_dev_tools(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
) {
    unsafe {
        let result = if webview.is_null() {
            Result::Err(crate::error::Error::InvalidReference)
        } else {
            Ok(WebView::attach_webview(webview))
        };
        let future = InvokeFuture::from_raw(_param);
        future.ready(result);
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
    webview.delegates().on_window_destroy.emit();
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
                .on_caret_changed
                .emit(Rect::from_native(r.as_ref().unwrap()));
        }
    }
}

pub(crate) extern "C" fn on_mouse_over_url_changed(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
    url: wkeString,
) {
    let webview = WebView::from_native(webview).unwrap();
    webview
        .delegates()
        .on_mouse_over_url_changed
        .emit(&from_wkestring(url));
}

pub(crate) extern "C" fn on_title_changed(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
    title: wkeString,
) {
    let webview = WebView::from_native(webview).unwrap();
    webview
        .delegates()
        .on_title_changed
        .emit(&from_wkestring(title));
}

pub(crate) extern "C" fn on_url_changed(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
    url: wkeString,
) {
    let webview = WebView::from_native(webview).unwrap();
    webview
        .delegates()
        .on_url_changed
        .emit(&from_wkestring(url));
}

pub(crate) extern "C" fn on_frame_url_changed(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
    frame: wkeWebFrameHandle,
    url: wkeString,
) {
    let frame = WebFrame::from_native(webview, frame);
    let webview = WebView::from_native(webview).unwrap();
    webview
        .delegates()
        .on_frame_url_changed
        .emit(&frame, &from_wkestring(url));
}

pub(crate) extern "C" fn on_alert_box(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
    msg: wkeString,
) {
    let webview = WebView::from_native(webview).unwrap();
    webview
        .delegates()
        .on_dialog
        .emit(&mut super::DialogType::Alert, &from_wkestring(msg));
}

pub(crate) extern "C" fn on_confirm_box(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
    msg: wkeString,
) -> bool {
    let webview = WebView::from_native(webview).unwrap();
    let mut result = HandleResult::default();
    webview.delegates().on_dialog.emit(
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
    let webview = WebView::from_native(webview).unwrap();
    let mut result = HandleResult::default();
    webview.delegates().on_dialog.emit(
        &mut super::DialogType::Prompt(&mut result),
        &from_wkestring(msg),
    );
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
    let webview = WebView::from_native(webview).unwrap();

    let mut result = HandleResult::default();
    webview.delegates().on_navigation.emit(
        NavigationType::from_native(navigation_type),
        &from_wkestring(url),
        &mut result,
    );
    result.unwrap_or(false)
}

pub(crate) extern "C" fn on_create_view(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
    navigation_type: wkeNavigationType,
    url: wkeString,
    features: *const wkeWindowFeatures,
) -> wkeWebView {
    unsafe {
        let webview = WebView::from_native(webview).unwrap();
        let mut result = HandleResult::default();

        webview.delegates().on_create_view.emit(
            NavigationType::from_native(navigation_type),
            &from_wkestring(url),
            WindowFeature::from_native(features.as_ref().unwrap()),
            &mut result,
        );

        if let Some(result) = result.value() {
            return match result {
                super::NavigationResult::Redirect => {
                    webview.load_url(&from_wkestring(url));
                    null_mut()
                }
                super::NavigationResult::NewWindow(new_webview) => new_webview.native(),
                super::NavigationResult::Abort => null_mut(),
            };
        };

        webview.load_url(&from_wkestring(url));
        null_mut()
    }
}

pub(crate) extern "C" fn on_document_ready(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
) {
    let webview = WebView::from_native(webview).unwrap();
    webview.delegates().on_document_ready.emit();
}

pub(crate) extern "C" fn on_frame_document_ready(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
    frame_id: wkeWebFrameHandle,
) {
    let frame = WebFrame::from_native(webview, frame_id);
    let webview = WebView::from_native(webview).unwrap();
    webview
        .delegates()
        .on_frame_document_ready
        .emit(&frame);
}

#[allow(non_upper_case_globals)]
pub(crate) extern "C" fn on_loading_finish(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
    url: wkeString,
    result: wkeLoadingResult,
    reason: wkeString,
) {
    let webview = WebView::from_native(webview).unwrap();
    let loading_result = match result {
        _wkeLoadingResult_WKE_LOADING_SUCCEEDED => LoadingResult::Succeeded,
        _wkeLoadingResult_WKE_LOADING_CANCELED => LoadingResult::Cancelled,
        _ => LoadingResult::Failed(from_wkestring(reason)),
    };
    webview
        .delegates()
        .on_loading_finish
        .emit(&from_wkestring(url), loading_result);
}

pub(crate) extern "C" fn on_console(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
    level: wkeConsoleLevel,
    message: wkeString,
    source_name: wkeString,
    source_line: ::std::os::raw::c_uint,
    stack_trace: wkeString,
) {
    let webview = WebView::from_native(webview).unwrap();
    let level = ConsoleLevel::from_native(level);
    webview
        .delegates()
        .on_console
        .emit(&super::ConsoleMessage {
            level,
            message: from_wkestring(message),
            source_name: from_wkestring(source_name),
            source_line: source_line,
            stack_trace: from_wkestring(stack_trace),
        });
}

pub(crate) extern "C" fn on_load_url_begin(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
    url: *const utf8,
    job: wkeNetJob,
) -> bool {
    unsafe {
        let webview = WebView::from_native(webview).unwrap();
        let url = from_cstr_ptr(url).unwrap_or("".to_owned());
        let mut result = HandleResult::default();
        webview
            .delegates()
            .on_load_url_begin
            .emit(&url, &Job::from_native(job), &mut result);

        result.unwrap_or(false)
    }
}

pub(crate) extern "C" fn on_load_url_end(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
    url: *const utf8,
    job: wkeNetJob,
    buf: *mut ::std::os::raw::c_void,
    len: ::std::os::raw::c_int,
) {
    unsafe {
        let webview = WebView::from_native(webview).unwrap();
        let mut job_buf = JobBuf::from_native(buf, len as usize);
        webview.delegates().on_load_url_end.emit(
            &from_cstr_ptr(url).unwrap_or("".to_owned()),
            &Job::from_native(job),
            &mut job_buf,
        );
    }
}

pub(crate) extern "C" fn on_load_url_headers_received(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
    url: *const utf8,
    job: wkeNetJob,
) {
    unsafe {
        let webview = WebView::from_native(webview).unwrap();
        let job = Job::from_native(job);
        webview
            .delegates()
            .on_load_url_headers_received
            .emit(&from_cstr_ptr(url).unwrap_or("".to_owned()), &job);
    }
}

pub(crate) extern "C" fn on_load_url_finish(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
    url: *const utf8,
    job: wkeNetJob,
    len: ::std::os::raw::c_int,
) {
    unsafe {
        let webview = WebView::from_native(webview).unwrap();
        let job = Job::from_native(job);
        webview.delegates().on_load_url_finish.emit(
            &from_cstr_ptr(url).unwrap_or("".to_owned()),
            &job,
            len,
        );
    }
}

pub(crate) extern "C" fn on_load_url_fail(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
    url: *const utf8,
    job: wkeNetJob,
) {
    unsafe {
        let webview = WebView::from_native(webview).unwrap();
        let job = Job::from_native(job);
        webview
            .delegates()
            .on_load_url_fail
            .emit(&from_cstr_ptr(url).unwrap_or("".to_owned()), &job);
    }
}

pub(crate) extern "C" fn on_did_create_script_context(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
    frame_id: wkeWebFrameHandle,
    _context: *mut ::std::os::raw::c_void,
    _extension_group: ::std::os::raw::c_int,
    _world_id: ::std::os::raw::c_int,
) {
    let frame = WebFrame::from_native(webview, frame_id);
    let webview = WebView::from_native(webview).unwrap();
    webview
        .delegates()
        .on_did_create_script_context
        .emit(&frame);
}

pub(crate) extern "C" fn on_will_release_script_context(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
    frame_id: wkeWebFrameHandle,
    _context: *mut ::std::os::raw::c_void,
    _world_id: ::std::os::raw::c_int,
) {
    let frame = WebFrame::from_native(webview, frame_id);
    let webview = WebView::from_native(webview).unwrap();

    webview
        .delegates()
        .on_will_release_script_context
        .emit(&frame);
}

pub(crate) extern "C" fn on_window_closing(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
) -> bool {
    let webview = WebView::from_native(webview).unwrap();
    let mut result = HandleResult::default();

    webview
        .delegates()
        .on_window_closing
        .emit(&mut result);

    result.unwrap_or(true)
}

pub(crate) extern "C" fn on_draggable_regions_changed(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
    rects: *const wkeDraggableRegion,
    count: ::std::os::raw::c_int,
) {
    unsafe {
        let webview = WebView::from_native(webview).unwrap();
        let mut regions = Vec::new();
        for index in 0..count as usize {
            let rc = rects.add(index).as_ref().unwrap();
            regions.push(DraggableRegion::from_native(rc));
        }

        webview
            .delegates()
            .on_draggable_regions_changed
            .emit(&regions);
    }
}

pub(crate) extern "C" fn on_will_media_load(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
    url: *const ::std::os::raw::c_char,
    info: *mut wkeMediaLoadInfo,
) {
    unsafe {
        let webview = WebView::from_native(webview).unwrap();
        webview.delegates().on_will_media_load.emit(
            &from_cstr_ptr(url).unwrap_or("".to_owned()),
            MediaInfo::from_native(info.read()),
        );
    }
}

pub(crate) extern "C" fn on_print(
    webview: wkeWebView,
    _param: *mut ::std::os::raw::c_void,
    frame_id: wkeWebFrameHandle,
    _params: *mut ::std::os::raw::c_void,
) {
    let frame = WebFrame::from_native(webview, frame_id);
    let webview = WebView::from_native(webview).unwrap();
    webview.delegates().on_print.emit(&frame);
}
