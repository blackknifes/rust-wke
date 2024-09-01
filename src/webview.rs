use super::common::{InvokeFuture, Size, UserValue};
use super::webframe::WebFrame;
use super::{common::Rect, Proxy};
use crate::common::handle::HandleResult;
use crate::common::lazy::Lazy;
use crate::error::{Error, Result};
use crate::net::{Job, JobBuf};
use crate::utils::{from_bool_int, from_cstr_ptr, to_bool_int, to_cstr16_ptr, to_cstr_ptr};
use crate::DefineMulticastDelegate;
use extern_c::{find_cookie_on_visit_all_cookie, FindCookie};
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::c_void;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::sync::Arc;
use std::{ffi::CStr, ptr::null_mut};
use wke_sys::*;
mod extern_c;

pub enum DebugConfig {
    ///开启开发者工具，此时param要填写开发者工具的资源路径，如file:///c:/miniblink-release/front_end/inspector.html。注意param此时必须是utf8编码
    ShowDevTools(String),
    ///设置帧率，默认值是10，值越大帧率越低
    WakeMinInterval(u32),
    ///设置帧率，默认值是3，值越大帧率越低
    DrawMinInterval(u32),
    ///设置抗锯齿渲染。param必须设置为"1"
    AntiAlias(bool),
    ///最小字体
    MinimumFontSize(u32),
    ///最小逻辑字体
    MinimumLogicalFontSize(u32),
    ///默认字体
    DefaultFontSize(u32),
    ///默认fixed字体
    DefaultFixedFontSize(u32),
}

impl DebugConfig {
    pub(crate) fn get_native_params(&self) -> (String, String) {
        match self {
            DebugConfig::ShowDevTools(path) => ("showDevTools".to_owned(), path.clone()),
            DebugConfig::WakeMinInterval(param) => {
                ("wakeMinInterval".to_owned(), param.to_string())
            }
            DebugConfig::DrawMinInterval(param) => {
                ("drawMinInterval".to_owned(), param.to_string())
            }
            DebugConfig::AntiAlias(param) => ("antiAlias".to_owned(), param.to_string()),
            DebugConfig::MinimumFontSize(param) => {
                ("minimumFontSize".to_owned(), param.to_string())
            }
            DebugConfig::MinimumLogicalFontSize(param) => {
                ("minimumLogicalFontSize".to_owned(), param.to_string())
            }
            DebugConfig::DefaultFontSize(param) => {
                ("defaultFontSize".to_owned(), param.to_string())
            }
            DebugConfig::DefaultFixedFontSize(param) => {
                ("defaultFixedFontSize".to_owned(), param.to_string())
            }
        }
    }
}

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MenuItemId {
    MenuSelectedAllId = _wkeMenuItemId_kWkeMenuSelectedAllId,
    MenuSelectedTextId = _wkeMenuItemId_kWkeMenuSelectedTextId,
    MenuUndoId = _wkeMenuItemId_kWkeMenuUndoId,
    MenuCopyImageId = _wkeMenuItemId_kWkeMenuCopyImageId,
    MenuInspectElementAtId = _wkeMenuItemId_kWkeMenuInspectElementAtId,
    MenuCutId = _wkeMenuItemId_kWkeMenuCutId,
    MenuPasteId = _wkeMenuItemId_kWkeMenuPasteId,
    MenuPrintId = _wkeMenuItemId_kWkeMenuPrintId,
    MenuGoForwardId = _wkeMenuItemId_kWkeMenuGoForwardId,
    MenuGoBackId = _wkeMenuItemId_kWkeMenuGoBackId,
    MenuReloadId = _wkeMenuItemId_kWkeMenuReloadId,
}

// wkeOnPaintUpdated
// wkeOnPaintBitUpdated

#[derive(Default)]
pub struct PromptResult {
    pub default: String,
    pub result: String,
}

pub enum DialogType<'a> {
    Alert,
    Confirm(&'a mut HandleResult<bool>),
    Prompt(&'a mut HandleResult<(bool, PromptResult)>),
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationType {
    LinkClick = _wkeNavigationType_WKE_NAVIGATION_TYPE_LINKCLICK,
    FormSubmit = _wkeNavigationType_WKE_NAVIGATION_TYPE_FORMSUBMITTE,
    BackForward = _wkeNavigationType_WKE_NAVIGATION_TYPE_BACKFORWARD,
    Reload = _wkeNavigationType_WKE_NAVIGATION_TYPE_RELOAD,
    FormreSubmit = _wkeNavigationType_WKE_NAVIGATION_TYPE_FORMRESUBMITT,
    Other = _wkeNavigationType_WKE_NAVIGATION_TYPE_OTHER,
}

impl NavigationType {
    pub(crate) fn from_native(navigation: wkeNavigationType) -> Self {
        match navigation {
            _wkeNavigationType_WKE_NAVIGATION_TYPE_LINKCLICK => Self::LinkClick,
            _wkeNavigationType_WKE_NAVIGATION_TYPE_FORMSUBMITTE => Self::FormSubmit,
            _wkeNavigationType_WKE_NAVIGATION_TYPE_BACKFORWARD => Self::BackForward,
            _wkeNavigationType_WKE_NAVIGATION_TYPE_RELOAD => Self::Reload,
            _wkeNavigationType_WKE_NAVIGATION_TYPE_FORMRESUBMITT => Self::FormreSubmit,
            _ => Self::Other,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct WindowFeature {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub menu_bar_visible: bool,
    pub status_bar_visible: bool,
    pub tool_bar_visible: bool,
    pub location_bar_visible: bool,
    pub scrollbars_visible: bool,
    pub resizable: bool,
    pub fullscreen: bool,
}

// pub const _wkeLoadingResult_WKE_LOADING_SUCCEEDED: _wkeLoadingResult = 0;
// pub const _wkeLoadingResult_WKE_LOADING_FAILED: _wkeLoadingResult = 1;
// pub const _wkeLoadingResult_WKE_LOADING_CANCELED: _wkeLoadingResult = 2;
#[derive(Debug, Clone)]
pub enum LoadingResult {
    Succeeded,
    Failed(String),
    Cancelled,
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsoleLevel {
    Log = _wkeConsoleLevel_wkeLevelLog,
    Warning = _wkeConsoleLevel_wkeLevelWarning,
    Error = _wkeConsoleLevel_wkeLevelError,
    Debug = _wkeConsoleLevel_wkeLevelDebug,
    Info = _wkeConsoleLevel_wkeLevelInfo,
    RevokedError = _wkeConsoleLevel_wkeLevelRevokedError,
}

#[derive(Debug, Clone)]
pub struct ConsoleMessage {
    pub level: ConsoleLevel,
    pub message: String,
    pub source_name: String,
    pub source_line: u32,
    pub stack_trace: String,
}

#[derive(Debug, Clone, Copy)]
pub struct DraggableRegion {
    pub bounds: Rect,
    pub draggable: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct MediaInfo {
    pub width: i32,
    pub height: i32,
    pub duration: f64,
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadType {
    StartLoading = _wkeOtherLoadType_WKE_DID_START_LOADING,
    StopLoading = _wkeOtherLoadType_WKE_DID_STOP_LOADING,
    Navigate = _wkeOtherLoadType_WKE_DID_NAVIGATE,
    NavigateInPage = _wkeOtherLoadType_WKE_DID_NAVIGATE_IN_PAGE,
    GetResponseDetails = _wkeOtherLoadType_WKE_DID_GET_RESPONSE_DETAILS,
    GetRedirectRequest = _wkeOtherLoadType_WKE_DID_GET_REDIRECT_REQUEST,
    PostRequest = _wkeOtherLoadType_WKE_DID_POST_REQUEST,
}
DefineMulticastDelegate!(WebViewCaretChangedDelegate, (rc: Rect)); //wkeOnCaretChanged
DefineMulticastDelegate!(WebViewMouseOverUrlChangedDelegate, (url: &str)); //wkeOnMouseOverUrlChanged
DefineMulticastDelegate!(WebViewTitleChangedDelegate, (title: &str)); // wkeOnTitleChanged
DefineMulticastDelegate!(WebViewUrlChangedDelegate, (url: &str)); // wkeOnURLChanged
DefineMulticastDelegate!(WebViewFrameUrlChangedDelegate, (frame: &WebFrame, url: &str)); // wkeOnURLChanged2
DefineMulticastDelegate!(WebViewDialogDelegate, (dialog: &mut DialogType, message: &str)); // wkeOnAlertBox, wkeOnConfirmBox, wkeOnPromptBox
DefineMulticastDelegate!(WebViewNavigationDelegate, (navigation: NavigationType, url: &str, result: &mut HandleResult<bool>)); // wkeOnNavigation
DefineMulticastDelegate!(WebViewCreateViewDelegate, (navigation: NavigationType, url: &str, feature: WindowFeature, result: &mut HandleResult<WebView>)); // wkeOnCreateView
DefineMulticastDelegate!(WebViewDocumentReadyDelegate, ()); // wkeOnDocumentReady
DefineMulticastDelegate!(WebViewFrameDocumentReadyDelegate, (frame: &WebFrame)); // wkeOnDocumentReady2
DefineMulticastDelegate!(WebViewLoadingFinishDelegate, (url: &str, result: LoadingResult)); // wkeOnLoadingFinish
DefineMulticastDelegate!(WebViewConsoleDelegate, (msg: &ConsoleMessage)); // wkeOnConsole
DefineMulticastDelegate!(WebViewLoadUrlBeginDelegate, (url: &str, job: &Job)); // wkeOnLoadUrlBegin
DefineMulticastDelegate!(WebViewLoadUrlEndDelegate, (url: &str, job: &Job, buf: &mut JobBuf)); // wkeOnLoadUrlEnd
DefineMulticastDelegate!(WebViewLoadUrlHeadersReceivedDelegate, (url: &str, job: &Job)); // wkeOnLoadUrlHeadersReceived
DefineMulticastDelegate!(WebViewLoadUrlFinishDelegate, (url: &str, job: &Job, len: i32)); // wkeOnLoadUrlFinish
DefineMulticastDelegate!(WebViewLoadUrlFailDelegate, (url: &str, job: &Job)); // wkeOnLoadUrlFail
DefineMulticastDelegate!(WebViewDidCreateScriptContextDelegate, (frame: &WebFrame)); // wkeOnDidCreateScriptContext
DefineMulticastDelegate!(WebViewWillReleaseScriptContextDelegate, (frame: &WebFrame)); // wkeOnWillReleaseScriptContext
DefineMulticastDelegate!(WebViewWindowClosingDelegate, (result: &mut HandleResult<bool>)); // wkeOnWindowClosing
DefineMulticastDelegate!(WebViewWindowDestroyDelegate, ()); // wkeOnWindowDestroy
DefineMulticastDelegate!(WebViewDraggableRegionsChangedDelegate, (rects: &[DraggableRegion])); // wkeOnDraggableRegionsChanged
DefineMulticastDelegate!(WebViewWillMediaLoadDelegate, (url: &str, info: MediaInfo)); // wkeOnWillMediaLoad
DefineMulticastDelegate!(WebViewPrintDelegate, (frame: &WebFrame /* , params: */)); // wkeOnPrint

// 以下回调未实现
// wkeOnPluginFind
// wkeOnContextMenuItemClick
// wkeOnOtherLoad
// wkeOnStartDragging
// wkeOnDownload
// wkeOnDownload2

pub struct WebViewDelegates {
    ///光标改变
    pub caret_changed_delegate: Lazy<WebViewCaretChangedDelegate>,
    /// 鼠标划过url
    pub mouse_over_url_changed_delegate: Lazy<WebViewMouseOverUrlChangedDelegate>,
    /// 标题改变
    pub title_changed_delegate: Lazy<WebViewTitleChangedDelegate>,
    /// url改变
    pub url_changed_delegate: Lazy<WebViewUrlChangedDelegate>,
    /// 某个frame中的url改变
    pub frame_url_changed_delegate: Lazy<WebViewFrameUrlChangedDelegate>,
    /// 弹出对话框
    pub dialog_delegate: Lazy<WebViewDialogDelegate>,
    /// 发生导航
    pub navigation_delegate: Lazy<WebViewNavigationDelegate>,
    /// 创建新的webview
    pub create_view_delegate: Lazy<WebViewCreateViewDelegate>,
    /// document已准备
    pub document_ready_delegate: Lazy<WebViewDocumentReadyDelegate>,
    /// 某个frame的document已准备
    pub frame_document_ready_delegate: Lazy<WebViewFrameDocumentReadyDelegate>,
    /// 加载完成
    pub loading_finish_delegate: Lazy<WebViewLoadingFinishDelegate>,
    /// 控制台
    pub console_delegate: Lazy<WebViewConsoleDelegate>,
    /// url加载开始
    pub load_url_begin_delegate: Lazy<WebViewLoadUrlBeginDelegate>,
    /// url加载结束
    pub load_url_end_delegate: Lazy<WebViewLoadUrlEndDelegate>,
    /// url加载接受到http头
    pub load_url_headers_received_delegate: Lazy<WebViewLoadUrlHeadersReceivedDelegate>,
    /// url加载完成
    pub load_url_finish_delegate: Lazy<WebViewLoadUrlFinishDelegate>,
    /// url加载失败
    pub load_url_fail_delegate: Lazy<WebViewLoadUrlFailDelegate>,
    /// js环境创建
    pub did_create_script_context_delegate: Lazy<WebViewDidCreateScriptContextDelegate>,
    /// js环境销毁
    pub will_release_script_context_delegate: Lazy<WebViewWillReleaseScriptContextDelegate>,
    /// 收到窗口关闭请求
    pub window_closing_delegate: Lazy<WebViewWindowClosingDelegate>,
    /// 可拖拽区域改变
    pub draggable_regions_changed_delegate: Lazy<WebViewDraggableRegionsChangedDelegate>,
    /// 将发生媒体加载
    pub will_media_load_delegate: Lazy<WebViewWillMediaLoadDelegate>,
    /// 打印
    pub print_delegate: Lazy<WebViewPrintDelegate>,

    /// 窗口销毁
    pub window_destroy_delegate: WebViewWindowDestroyDelegate,
}

pub(crate) struct WebViewInner {
    webview: wkeWebView,
    values: HashMap<String, Box<dyn Any>>,
    delegates: WebViewDelegates,
}

macro_rules! LazyNew {
    ($webview: ident, $wke: ident, $cb: ident) => {
        Lazy::new(move || unsafe {
            $wke.unwrap()($webview, Some(extern_c::$cb), null_mut());
            Default::default()
        })
    };
}

impl WebViewInner {
    pub fn new(webview: wkeWebView) -> Self {
        Self {
            webview,
            values: Default::default(),
            delegates: WebViewDelegates {
                caret_changed_delegate: LazyNew!(webview, wkeOnCaretChanged, on_caret_changed),
                mouse_over_url_changed_delegate: LazyNew!(
                    webview,
                    wkeOnMouseOverUrlChanged,
                    on_mouse_over_url_changed
                ),
                title_changed_delegate: LazyNew!(webview, wkeOnTitleChanged, on_title_changed),
                url_changed_delegate: LazyNew!(webview, wkeOnURLChanged, on_url_changed),
                frame_url_changed_delegate: LazyNew!(
                    webview,
                    wkeOnURLChanged2,
                    on_frame_url_changed
                ),
                dialog_delegate: Lazy::new(move || unsafe {
                    wkeOnAlertBox.unwrap()(webview, Some(extern_c::on_alert_box), null_mut());
                    wkeOnConfirmBox.unwrap()(webview, Some(extern_c::on_confirm_box), null_mut());
                    wkeOnPromptBox.unwrap()(webview, Some(extern_c::on_prompt_box), null_mut());
                    Default::default()
                }),
                navigation_delegate: LazyNew!(webview, wkeOnNavigation, on_navigation),
                create_view_delegate: LazyNew!(webview, wkeOnCreateView, on_create_view),
                document_ready_delegate: LazyNew!(webview, wkeOnDocumentReady, on_document_ready),
                frame_document_ready_delegate: LazyNew!(
                    webview,
                    wkeOnDocumentReady2,
                    on_frame_document_ready
                ),
                loading_finish_delegate: LazyNew!(webview, wkeOnLoadingFinish, on_loading_finish),
                console_delegate: LazyNew!(webview, wkeOnConsole, on_console),
                load_url_begin_delegate: LazyNew!(webview, wkeOnLoadUrlBegin, on_load_url_begin),
                load_url_end_delegate: LazyNew!(webview, wkeOnLoadUrlEnd, on_load_url_end),
                load_url_headers_received_delegate: LazyNew!(
                    webview,
                    wkeOnLoadUrlHeadersReceived,
                    on_load_url_headers_received
                ),
                load_url_finish_delegate: LazyNew!(webview, wkeOnLoadUrlFinish, on_load_url_finish),
                load_url_fail_delegate: LazyNew!(webview, wkeOnLoadUrlFail, on_load_url_fail),
                did_create_script_context_delegate: LazyNew!(
                    webview,
                    wkeOnDidCreateScriptContext,
                    on_did_create_script_context
                ),
                will_release_script_context_delegate: LazyNew!(
                    webview,
                    wkeOnWillReleaseScriptContext,
                    on_will_release_script_context
                ),
                window_closing_delegate: LazyNew!(webview, wkeOnWindowClosing, on_window_closing),
                draggable_regions_changed_delegate: LazyNew!(
                    webview,
                    wkeOnDraggableRegionsChanged,
                    on_draggable_regions_changed
                ),
                will_media_load_delegate: LazyNew!(webview, wkeOnWillMediaLoad, on_will_media_load),
                print_delegate: LazyNew!(webview, wkeOnPrint, on_print),
                window_destroy_delegate: Default::default(),
            },
        }
    }
}

pub struct WebView {
    inner: Rc<RefCell<WebViewInner>>,
}

pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub secure: bool,
    pub http_only: bool,
    pub expires: Option<i32>,
}

impl Cookie {
    pub(crate) unsafe fn from_native(
        name: String,
        value: *const ::std::os::raw::c_char,
        domain: *const ::std::os::raw::c_char,
        path: *const ::std::os::raw::c_char,
        secure: ::std::os::raw::c_int,
        http_only: ::std::os::raw::c_int,
        expires: *mut ::std::os::raw::c_int,
    ) -> Result<Self> {
        let value = from_cstr_ptr(value)?;
        let domain = from_cstr_ptr(domain)?;
        let path = from_cstr_ptr(path)?;
        let secure = from_bool_int(secure);
        let http_only = from_bool_int(http_only);
        let expires = if expires.is_null() {
            None
        } else {
            Some(expires.read())
        };
        return Ok(Self {
            name,
            value,
            domain,
            path,
            secure,
            http_only,
            expires,
        });
    }
}

extern "C" fn on_visit_all_cookie(
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
        let callback: &Box<dyn Fn(Cookie) -> bool> = std::mem::transmute(params);
        let expires = if expires.is_null() {
            None
        } else {
            Some(expires.read())
        };

        callback(Cookie {
            name: from_cstr_ptr(name).unwrap_or("".to_owned()),
            value: from_cstr_ptr(value).unwrap_or("".to_owned()),
            domain: from_cstr_ptr(domain).unwrap_or("".to_owned()),
            path: from_cstr_ptr(path).unwrap_or("".to_owned()),
            secure: from_bool_int(secure),
            http_only: from_bool_int(http_only),
            expires,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DefaultPrinterSettings {
    pub is_landscape: bool,
    pub is_print_head_footer: bool,
    pub is_print_backgroud: bool,
    pub edge_distance_left: i32,
    pub edge_distance_top: i32,
    pub edge_distance_right: i32,
    pub edge_distance_bottom: i32,
    pub copies: i32,
    pub paper_type: i32,
}

impl DefaultPrinterSettings {
    #[allow(dead_code)]
    pub(crate) fn into_wke(&self) -> wkeDefaultPrinterSettings {
        wkeDefaultPrinterSettings {
            structSize: std::mem::size_of::<wkeDefaultPrinterSettings>() as i32,
            isLandscape: to_bool_int(self.is_landscape),
            isPrintHeadFooter: to_bool_int(self.is_print_head_footer),
            isPrintBackgroud: to_bool_int(self.is_print_backgroud),
            edgeDistanceLeft: self.edge_distance_left,
            edgeDistanceTop: self.edge_distance_top,
            edgeDistanceRight: self.edge_distance_right,
            edgeDistanceBottom: self.edge_distance_bottom,
            copies: self.copies,
            paperType: self.paper_type,
        }
    }
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CookieCommand {
    ClearAllCookies = _wkeCookieCommand_wkeCookieCommandClearAllCookies,
    ClearSessionCookies = _wkeCookieCommand_wkeCookieCommandClearSessionCookies,
    FlushCookiesToFile = _wkeCookieCommand_wkeCookieCommandFlushCookiesToFile,
    ReloadCookiesFromFile = _wkeCookieCommand_wkeCookieCommandReloadCookiesFromFile,
}

pub struct WebViewDelegatesMut<'a>(std::cell::RefMut<'a, WebViewInner>);
impl<'a> Deref for WebViewDelegatesMut<'a> {
    type Target = WebViewDelegates;

    fn deref(&self) -> &Self::Target {
        &self.0.delegates
    }
}
impl<'a> DerefMut for WebViewDelegatesMut<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0.delegates
    }
}

impl WebView {
    pub(crate) fn get_webview(&self) -> wkeWebView {
        self.inner.borrow().webview
    }

    pub(crate) fn attach_webview(webview: wkeWebView) -> Self {
        unsafe {
            if let Ok(webview) = Self::from_native(webview) {
                return webview;
            }

            let inner = Rc::new(RefCell::new(WebViewInner::new(webview)));
            let ptr = Rc::into_raw(inner.clone());

            wkeSetUserKeyValue.unwrap()(
                webview,
                to_cstr_ptr("rust").unwrap().to_utf8(),
                ptr as *mut c_void,
            );

            wkeOnWindowDestroy.unwrap()(webview, Some(extern_c::on_window_destroy), null_mut());

            Self { inner }
        }
    }

    pub(crate) fn detach_webview(webview: wkeWebView) -> Result<Self> {
        unsafe {
            if !from_bool_int(wkeIsWebviewValid.unwrap()(webview)) {
                return Err(Error::InvalidReference);
            }

            let ptr = wkeGetUserKeyValue.unwrap()(webview, to_cstr_ptr("rust").unwrap().to_utf8());
            if ptr.is_null() {
                return Err(Error::InvalidReference);
            }

            let inner = Rc::from_raw(ptr as *const RefCell<WebViewInner>);

            Ok(Self { inner })
        }
    }

    pub(crate) fn from_native(webview: wkeWebView) -> Result<Self> {
        unsafe {
            if !from_bool_int(wkeIsWebviewValid.unwrap()(webview)) {
                return Err(Error::InvalidReference);
            }

            let ptr = wkeGetUserKeyValue.unwrap()(webview, to_cstr_ptr("rust").unwrap().to_utf8());
            if ptr.is_null() {
                return Err(Error::InvalidReference);
            }

            Rc::increment_strong_count(ptr);
            let inner = Rc::from_raw(ptr as *const RefCell<WebViewInner>);

            Ok(Self { inner })
        }
    }

    pub fn popup(rc: Rect) -> Self {
        unsafe {
            let webview = wkeCreateWebWindow.unwrap()(
                _wkeWindowType_WKE_WINDOW_TYPE_POPUP,
                null_mut(),
                rc.x,
                rc.y,
                rc.width,
                rc.height,
            );

            Self::attach_webview(webview)
        }
    }

    pub fn from_current_context() -> Option<Self> {
        unsafe {
            let webview = wkeGetWebViewForCurrentContext.unwrap()();
            if !from_bool_int(wkeIsWebviewValid.unwrap()(webview)) {
                return None;
            }

            Some(Self::attach_webview(webview))
        }
    }

    pub fn delegates(&self) -> WebViewDelegatesMut {
        WebViewDelegatesMut(self.inner.borrow_mut())
    }

    pub fn close(&self) {
        unsafe {
            wkeDestroyWebView.unwrap()(self.get_webview());
        }
    }

    pub fn is_valid(&self) -> bool {
        unsafe { from_bool_int(wkeIsWebviewValid.unwrap()(self.get_webview())) }
    }

    pub fn get_name(&self) -> Result<String> {
        unsafe { from_cstr_ptr(wkeWebViewName.unwrap()(self.get_webview())) }
    }

    pub fn set_name(&self, name: &str) -> Result<()> {
        unsafe {
            wkeSetWebViewName.unwrap()(self.get_webview(), to_cstr_ptr(name)?.to_utf8());
            Ok(())
        }
    }

    pub fn is_loaded(&self) -> bool {
        unsafe { from_bool_int(wkeIsLoaded.unwrap()(self.get_webview())) }
    }

    pub fn is_load_failed(&self) -> bool {
        unsafe { from_bool_int(wkeIsLoadFailed.unwrap()(self.get_webview())) }
    }

    pub fn is_load_complete(&self) -> bool {
        unsafe { from_bool_int(wkeIsLoadComplete.unwrap()(self.get_webview())) }
    }

    pub fn is_loading(&self) -> bool {
        unsafe { from_bool_int(wkeIsLoading.unwrap()(self.get_webview())) }
    }

    pub fn is_loading_succeeded(&self) -> bool {
        unsafe { from_bool_int(wkeIsLoadingSucceeded.unwrap()(self.get_webview())) }
    }

    pub fn is_loading_failed(&self) -> bool {
        unsafe { from_bool_int(wkeIsLoadingFailed.unwrap()(self.get_webview())) }
    }

    pub fn is_loading_completed(&self) -> bool {
        unsafe { from_bool_int(wkeIsLoadingCompleted.unwrap()(self.get_webview())) }
    }

    pub fn is_document_ready(&self) -> bool {
        unsafe { from_bool_int(wkeIsDocumentReady.unwrap()(self.get_webview())) }
    }

    pub fn get_source(&self) -> Result<String> {
        unsafe { from_cstr_ptr(wkeGetSource.unwrap()(self.get_webview())) }
    }

    pub fn move_to(&self, rc: Rect) {
        unsafe {
            wkeMoveWindow.unwrap()(self.get_webview(), rc.x, rc.y, rc.width, rc.height);
        }
    }

    pub fn move_to_center(&self) {
        unsafe {
            wkeMoveToCenter.unwrap()(self.get_webview());
        }
    }

    pub fn get_caret_rect(&self) -> Rect {
        unsafe {
            let rc = wkeGetCaretRect.unwrap()(self.get_webview());
            return Rect::from_native(&rc);
        }
    }

    pub fn set_media_volume(&self, volume: f32) {
        unsafe {
            wkeSetMediaVolume.unwrap()(self.get_webview(), volume);
        }
    }

    pub fn get_media_volume(&self) -> f32 {
        unsafe { wkeGetMediaVolume.unwrap()(self.get_webview()) }
    }

    pub fn set_proxy(&self, proxy: Proxy) -> crate::error::Result<()> {
        unsafe {
            let mut wke_proxy = proxy.into_native()?;
            wkeSetViewProxy.unwrap()(self.get_webview(), &mut wke_proxy);
            Ok(())
        }
    }

    pub fn set_debug_config(&self, config: DebugConfig) -> Result<()> {
        unsafe {
            let (debug_str, param) = config.get_native_params();
            wkeSetDebugConfig.unwrap()(
                self.get_webview(),
                to_cstr_ptr(&debug_str)?.to_utf8(),
                to_cstr_ptr(&param)?.to_utf8(),
            );
            Ok(())
        }
    }

    pub fn set_mouse_enabled(&self, enable: bool) {
        unsafe {
            wkeSetMouseEnabled.unwrap()(self.get_webview(), enable);
        }
    }

    pub fn set_touch_enabled(&self, enable: bool) {
        unsafe {
            wkeSetTouchEnabled.unwrap()(self.get_webview(), enable);
        }
    }

    pub fn set_system_touch_enabled(&self, enable: bool) {
        unsafe {
            wkeSetSystemTouchEnabled.unwrap()(self.get_webview(), enable);
        }
    }

    pub fn set_context_menu_enabled(&self, enable: bool) {
        unsafe {
            wkeSetContextMenuEnabled.unwrap()(self.get_webview(), enable);
        }
    }

    pub fn set_navigation_to_new_window_enabled(&self, enable: bool) {
        unsafe {
            wkeSetNavigationToNewWindowEnable.unwrap()(self.get_webview(), enable);
        }
    }

    pub fn set_headless_enabled(&self, enable: bool) {
        unsafe {
            wkeSetHeadlessEnabled.unwrap()(self.get_webview(), enable);
        }
    }

    pub fn set_drag_drop_enabled(&self, enable: bool) {
        unsafe {
            wkeSetDragDropEnable.unwrap()(self.get_webview(), enable);
        }
    }

    pub fn set_drag_enabled(&self, enable: bool) {
        unsafe {
            wkeSetDragEnable.unwrap()(self.get_webview(), enable);
        }
    }

    pub fn set_context_menu_item_show(&self, menu_item_id: MenuItemId, show: bool) {
        unsafe {
            wkeSetContextMenuItemShow.unwrap()(self.get_webview(), menu_item_id as i32, show);
        }
    }

    pub fn set_language(&self, language: &str) -> Result<()> {
        unsafe {
            wkeSetLanguage.unwrap()(self.get_webview(), to_cstr_ptr(language)?.to_utf8());
            Ok(())
        }
    }

    pub fn set_handle(&self, hwnd: HWND) {
        unsafe {
            wkeSetHandle.unwrap()(self.get_webview(), hwnd);
        }
    }

    pub fn set_handle_offset(&self, x: i32, y: i32) {
        unsafe {
            wkeSetHandleOffset.unwrap()(self.get_webview(), x, y);
        }
    }

    pub fn get_host_hwnd(&self) -> HWND {
        unsafe {
            return wkeGetHostHWND.unwrap()(self.get_webview());
        }
    }

    pub fn set_transparent(&self, transparent: bool) {
        unsafe {
            wkeSetTransparent.unwrap()(self.get_webview(), transparent);
        }
    }

    pub fn set_csp_check_enabled(&self, enable: bool) {
        unsafe {
            wkeSetCspCheckEnable.unwrap()(self.get_webview(), enable);
        }
    }

    pub fn set_npapi_plugins_enabled(&self, enable: bool) {
        unsafe {
            wkeSetNpapiPluginsEnabled.unwrap()(self.get_webview(), enable);
        }
    }

    pub fn set_memory_cache_enabled(&self, enable: bool) {
        unsafe {
            wkeSetMemoryCacheEnable.unwrap()(self.get_webview(), enable);
        }
    }

    pub fn set_cookie_enabled(&self, enable: bool) {
        unsafe {
            wkeSetCookieEnabled.unwrap()(self.get_webview(), enable);
        }
    }

    pub fn is_cookie_enabled(&self) -> bool {
        unsafe { from_bool_int(wkeIsCookieEnabled.unwrap()(self.get_webview())) }
    }

    pub fn set_cookie(&self, url: &str, cookie: &str) -> Result<()> {
        unsafe {
            wkeSetCookie.unwrap()(
                self.get_webview(),
                to_cstr_ptr(url)?.to_utf8(),
                to_cstr_ptr(cookie)?.to_utf8(),
            );
            Ok(())
        }
    }

    pub fn set_cookie_jar_path(&self, path: &str) {
        unsafe {
            wkeSetCookieJarPath.unwrap()(self.get_webview(), (&to_cstr16_ptr(path)).as_ptr());
        }
    }

    pub fn set_cookie_jar_full_path(&self, path: &str) {
        unsafe {
            wkeSetCookieJarFullPath.unwrap()(self.get_webview(), (&to_cstr16_ptr(path)).as_ptr());
        }
    }

    pub fn set_local_storage_full_path(&self, path: &str) {
        unsafe {
            wkeSetLocalStorageFullPath.unwrap()(
                self.get_webview(),
                (&to_cstr16_ptr(path)).as_ptr(),
            );
        }
    }

    pub fn get_title(&self) -> Result<String> {
        unsafe {
            let title = CStr::from_ptr(wkeGetTitle.unwrap()(self.get_webview()))
                .to_str()
                .map_err(Error::other)?
                .to_owned();
            Ok(title)
        }
    }

    pub fn set_title(&self, title: &str) -> Result<()> {
        unsafe {
            wkeSetWindowTitle.unwrap()(self.get_webview(), to_cstr_ptr(title)?.to_utf8());
            Ok(())
        }
    }

    pub fn get_url(&self) -> Result<String> {
        self.get_main_frame()
            .ok_or_else(|| Error::InvalidReference)?
            .get_url()
    }

    pub fn get_cursor_info_type(&self) -> i32 {
        unsafe {
            return wkeGetCursorInfoType.unwrap()(self.get_webview());
        }
    }

    pub fn add_plugin_directory(&self, path: &str) {
        unsafe {
            wkeAddPluginDirectory.unwrap()(self.get_webview(), (&to_cstr16_ptr(path)).as_ptr());
        }
    }

    pub fn set_user_agent(&self, user_agent: &str) -> Result<()> {
        unsafe {
            wkeSetUserAgent.unwrap()(self.get_webview(), to_cstr_ptr(user_agent)?.to_utf8());
            Ok(())
        }
    }

    pub fn get_user_agent(&self) -> Result<String> {
        unsafe { from_cstr_ptr(wkeGetUserAgent.unwrap()(self.get_webview())) }
    }

    pub fn show_devtools(&self, path: &str) -> InvokeFuture<Result<WebView>> {
        unsafe {
            let path_u16 = to_cstr16_ptr(path);
            let future = InvokeFuture::default();
            wkeShowDevtools.unwrap()(
                self.get_webview(),
                (&path_u16).as_ptr(),
                Some(extern_c::on_show_dev_tools),
                future.into_raw(),
            );
            future
        }
    }

    pub fn set_zoom_factor(&self, factor: f32) {
        unsafe {
            wkeSetZoomFactor.unwrap()(self.get_webview(), factor);
        }
    }

    pub fn get_zoom_factor(&self) -> f32 {
        unsafe { wkeGetZoomFactor.unwrap()(self.get_webview()) }
    }

    pub fn gc(&self, interval_seconds: i32) {
        unsafe {
            wkeGC.unwrap()(self.get_webview(), interval_seconds);
        }
    }

    pub fn set_resource_gc(&self, interval_seconds: i32) {
        unsafe {
            wkeSetResourceGc.unwrap()(self.get_webview(), interval_seconds);
        }
    }

    pub fn can_go_forward(&self) -> bool {
        unsafe { from_bool_int(wkeCanGoForward.unwrap()(self.get_webview())) }
    }

    pub fn can_go_back(&self) -> bool {
        unsafe { from_bool_int(wkeCanGoBack.unwrap()(self.get_webview())) }
    }

    pub fn get_cookie(&self) -> Result<String> {
        unsafe { from_cstr_ptr(wkeGetCookie.unwrap()(self.get_webview())) }
    }

    pub fn find_cookie(&self, name: &str) -> Option<Cookie> {
        unsafe {
            let mut find_cookie = FindCookie::new(name.to_owned());
            wkeVisitAllCookie.unwrap()(
                self.get_webview(),
                &mut find_cookie as *mut FindCookie as *mut c_void,
                Some(find_cookie_on_visit_all_cookie),
            );
            return find_cookie.cookie;
        }
    }

    pub fn visit_all_cookies(&self, callback: impl Fn(Cookie) -> bool) {
        unsafe {
            let mut boxed = Box::new(callback);
            let ptr: *mut Box<dyn Fn(Cookie) -> bool> = std::mem::transmute(&mut boxed);
            wkeVisitAllCookie.unwrap()(
                self.get_webview(),
                ptr as *mut c_void,
                Some(on_visit_all_cookie),
            );
        }
    }

    pub fn clear_cookie(&self) {
        unsafe {
            wkeClearCookie.unwrap()(self.get_webview());
        }
    }

    pub fn resize(&self, size: Size) {
        unsafe { wkeResize.unwrap()(self.get_webview(), size.width, size.height) }
    }

    pub fn get_size(&self) -> Size {
        unsafe {
            let width = wkeGetWidth.unwrap()(self.get_webview());
            let height = wkeGetHeight.unwrap()(self.get_webview());
            Size { width, height }
        }
    }

    pub fn go_back(&self) {
        unsafe {
            wkeGoBack.unwrap()(self.get_webview());
        }
    }

    pub fn go_forward(&self) {
        unsafe {
            wkeGoForward.unwrap()(self.get_webview());
        }
    }

    pub fn navigate_at_index(&self, index: i32) {
        unsafe {
            wkeNavigateAtIndex.unwrap()(self.get_webview(), index);
        }
    }

    pub fn get_navigate_index(&self) -> i32 {
        unsafe { wkeGetNavigateIndex.unwrap()(self.get_webview()) }
    }

    pub fn stop_loading(&self) {
        unsafe {
            wkeStopLoading.unwrap()(self.get_webview());
        }
    }

    pub fn reload(&self) {
        unsafe {
            wkeReload.unwrap()(self.get_webview());
        }
    }

    pub fn perform_cookie_command(&self, command: CookieCommand) {
        unsafe {
            wkePerformCookieCommand.unwrap()(self.get_webview(), command as i32);
        }
    }

    pub fn select_all(&self) {
        unsafe {
            wkeEditorSelectAll.unwrap()(self.get_webview());
        }
    }

    pub fn unselect(&self) {
        unsafe {
            wkeEditorUnSelect.unwrap()(self.get_webview());
        }
    }

    pub fn copy(&self) {
        unsafe {
            wkeEditorCopy.unwrap()(self.get_webview());
        }
    }

    pub fn cut(&self) {
        unsafe {
            wkeEditorCut.unwrap()(self.get_webview());
        }
    }
    pub fn paste(&self) {
        unsafe {
            wkeEditorPaste.unwrap()(self.get_webview());
        }
    }
    pub fn delete(&self) {
        unsafe {
            wkeEditorDelete.unwrap()(self.get_webview());
        }
    }
    pub fn undo(&self) {
        unsafe {
            wkeEditorUndo.unwrap()(self.get_webview());
        }
    }

    pub fn redo(&self) {
        unsafe {
            wkeEditorRedo.unwrap()(self.get_webview());
        }
    }

    pub fn set_focus(&self) {
        unsafe {
            wkeSetFocus.unwrap()(self.get_webview());
        }
    }

    pub fn kill_focus(&self) {
        unsafe {
            wkeKillFocus.unwrap()(self.get_webview());
        }
    }

    pub fn show(&self) {
        unsafe {
            wkeShowWindow.unwrap()(self.get_webview(), true);
        }
    }

    pub fn hide(&self) {
        unsafe {
            wkeShowWindow.unwrap()(self.get_webview(), false);
        }
    }

    pub fn load_html(&self, html: &str) -> Result<()> {
        unsafe {
            wkeLoadHTML.unwrap()(self.get_webview(), to_cstr_ptr(html)?.to_utf8());
            Ok(())
        }
    }

    pub fn load_url(&self, url: &str) {
        unsafe {
            let url = to_cstr16_ptr(url);
            wkeLoadURLW.unwrap()(self.get_webview(), (&url).as_ptr());
        }
    }

    pub fn load_html_with_base_url(&self, html: &str, base_url: &str) -> Result<()> {
        unsafe {
            wkeLoadHtmlWithBaseUrl.unwrap()(
                self.get_webview(),
                to_cstr_ptr(html)?.to_utf8(),
                to_cstr_ptr(base_url)?.to_utf8(),
            );
            Ok(())
        }
    }

    pub fn post_url(&self, url: &str, post_data: &str) -> Result<()> {
        unsafe {
            wkePostURL.unwrap()(
                self.get_webview(),
                to_cstr_ptr(url)?.to_utf8(),
                to_cstr_ptr(post_data)?.to_utf8(),
                post_data.len() as i32,
            );
            Ok(())
        }
    }
    pub fn unlock_view_dc(&self) {
        unsafe {
            wkeUnlockViewDC.unwrap()(self.get_webview());
        }
    }

    pub fn get_main_frame(&self) -> Option<WebFrame> {
        unsafe {
            let frame = wkeWebFrameGetMainFrame.unwrap()(self.get_webview());
            if frame.is_null() {
                return None;
            }
            Some(WebFrame::from_native(self.get_webview(), frame))
        }
    }

    pub fn is_main(&self, frame: &WebFrame) -> bool {
        unsafe { from_bool_int(wkeIsMainFrame.unwrap()(self.get_webview(), frame.frame)) }
    }

    pub async fn get_content_as_markup(&self) -> String {
        todo!()
    }

    pub async fn util_serialize_to_mhtml(&self) -> String {
        todo!()
    }

    pub fn get_content_size(&self) -> Size {
        unsafe {
            Size {
                width: wkeGetContentWidth.unwrap()(self.get_webview()),
                height: wkeGetContentHeight.unwrap()(self.get_webview()),
            }
        }
    }

    pub fn set_user_value<T: 'static>(&self, key: &str, value: T) -> Result<()> {
        unsafe {
            let ptr = UserValue::into_raw(UserValue::new(value));
            wkeSetUserKeyValue.unwrap()(
                self.get_webview(),
                to_cstr_ptr(key)?.to_utf8(),
                ptr as *mut c_void,
            );
            Ok(())
        }
    }

    pub fn get_user_value<T: 'static>(&self, key: &str) -> Result<Arc<UserValue<T>>> {
        unsafe {
            let ptr = wkeGetUserKeyValue.unwrap()(self.get_webview(), to_cstr_ptr(key)?.to_utf8());
            UserValue::from_raw(ptr as *const UserValue<T>)
        }
    }

    pub fn go_to_offset(&self, offset: i32) {
        unsafe {
            wkeGoToOffset.unwrap()(self.get_webview(), offset);
        }
    }

    pub fn go_to_index(&self, index: i32) {
        unsafe {
            wkeGoToIndex.unwrap()(self.get_webview(), index);
        }
    }

    pub fn set_editable(&self, editable: bool) {
        unsafe {
            wkeSetEditable.unwrap()(self.get_webview(), editable);
        }
    }

    pub fn awake(&self) {
        unsafe { wkeWake.unwrap()(self.get_webview()) }
    }

    pub fn sleep(&self) {
        unsafe { wkeSleep.unwrap()(self.get_webview()) }
    }

    pub fn is_awake(&self) -> bool {
        unsafe { from_bool_int(wkeIsAwake.unwrap()(self.get_webview())) }
    }

    pub fn is_transparent(&self) -> bool {
        unsafe { from_bool_int(wkeIsTransparent.unwrap()(self.get_webview())) }
    }
}
