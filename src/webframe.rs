use super::javascript::{Context, JsValue};
use crate::common::Size;
use crate::error::Error;
use crate::utils::{from_bool_int, from_mem, from_ptr, to_bool_int, to_cstr_ptr};
use crate::webview::WebView;
use crate::{error::Result, utils::from_cstr_ptr};
use std::ptr::null_mut;
use wke_sys::*;

/// 打印设置
#[derive(Clone, Copy)]
pub struct PrintSettings {
    /// 打印dpi
    pub dpi: i32,
    /// 纸张宽度
    pub width: u32,
    /// 纸张高度
    pub height: u32,

    /// 纸张上边距
    pub margin_top: i32,
    /// 纸张下边距
    pub margin_bottom: i32,
    /// 纸张左边距
    pub margin_left: i32,
    /// 纸张右边距
    pub margin_right: i32,
    /// 是否打印页头与页脚
    pub is_print_page_head_and_footer: bool,
    /// 是否打印背景
    pub is_print_backgroud: bool,
    /// 是否横屏
    pub is_landscape: bool,
    /// 是否打印多页
    pub is_print_to_multi_page: bool,
}

impl PrintSettings {
    /// 使用4k纸张来创建打印设置
    pub fn new_4k() -> Self {
        Self {
            dpi: 300,                            // 高质量打印常用的 DPI
            width: 4096,                         // 4K 纸张宽度（像素）
            height: 2160,                        // 4K 纸张高度（像素）
            margin_top: 50,                      // 50 像素上边距
            margin_bottom: 50,                   // 50 像素下边距
            margin_left: 50,                     // 50 像素左边距
            margin_right: 50,                    // 50 像素右边距
            is_print_page_head_and_footer: true, // 打印页眉和页脚
            is_print_backgroud: false,           // 不打印背景
            is_landscape: false,                 // 纵向打印
            is_print_to_multi_page: false,       // 不打印成多页
        }
    }

    pub(crate) fn into_native(&self) -> wkePrintSettings {
        wkePrintSettings {
            structSize: std::mem::size_of::<wkePrintSettings>() as i32,
            dpi: self.dpi,
            width: self.width as i32,
            height: self.height as i32,
            marginTop: self.margin_top,
            marginBottom: self.margin_bottom,
            marginLeft: self.margin_left,
            marginRight: self.margin_right,
            isPrintPageHeadAndFooter: to_bool_int(self.is_print_page_head_and_footer),
            isPrintBackgroud: to_bool_int(self.is_print_backgroud),
            isLandscape: to_bool_int(self.is_landscape),
            isPrintToMultiPage: to_bool_int(self.is_print_to_multi_page),
        }
    }
}

impl std::default::Default for PrintSettings {
    fn default() -> Self {
        Self::new_4k()
    }
}

/// frame对象，主要代表页面主框架与iframe
pub struct WebFrame {
    pub(crate) webview: wkeWebView,
    pub(crate) frame: wkeWebFrameHandle,
}

impl WebFrame {
    pub(crate) fn from_native(webview: wkeWebView, frame: wkeWebFrameHandle) -> Self {
        Self { webview, frame }
    }

    pub fn webview(&self) -> Result<WebView> {
        WebView::from_native(self.webview)
    }

    /// 获取url
    pub fn get_url(&self) -> Result<String> {
        unsafe { from_cstr_ptr(wkeGetFrameUrl.unwrap()(self.webview, self.frame)) }
    }

    /// 获取document完整url
    pub fn get_document_complete_url(&self, partial_url: &str) -> Result<String> {
        unsafe {
            from_cstr_ptr(wkeGetDocumentCompleteURL.unwrap()(
                self.webview,
                self.frame,
                to_cstr_ptr(partial_url)?.to_utf8(),
            ))
        }
    }

    /// 获取环境
    pub fn get_context(&self) -> Context {
        unsafe { Context::from_native(wkeGetGlobalExecByFrame.unwrap()(self.webview, self.frame)) }
    }

    /// 是否为主框架
    pub fn is_main(&self) -> bool {
        unsafe { from_bool_int(wkeIsMainFrame.unwrap()(self.webview, self.frame)) }
    }

    /// 是否为远程框架
    pub fn is_remote_frame(&self) -> bool {
        unsafe { from_bool_int(wkeIsWebRemoteFrame.unwrap()(self.webview, self.frame)) }
    }

    /// 执行js脚本
    pub fn run_js(&self, script: &str, is_in_closure: bool) -> Result<JsValue> {
        unsafe {
            let val = wkeRunJsByFrame.unwrap()(
                self.webview,
                self.frame,
                to_cstr_ptr(script)?.to_utf8(),
                is_in_closure,
            );
            Ok(JsValue::from_native(val))
        }
    }

    /// 插入css
    pub fn insert_css_by_frame(&self, css: &str) -> Result<()> {
        unsafe {
            wkeInsertCSSByFrame.unwrap()(self.webview, self.frame, to_cstr_ptr(css)?.to_utf8());
            Ok(())
        }
    }

    /// 打印到pdf
    pub fn print_to_pdf(&self, settings: PrintSettings) -> Result<Vec<Vec<u8>>> {
        unsafe {
            let settings = settings.into_native();
            let ptr = wkeUtilPrintToPdf.unwrap()(self.webview, self.frame, &settings);
            if ptr.is_null() {
                return Err(Error::InvalidReference);
            }

            let datas = ptr.as_ref().unwrap();
            let mut datas_ret = Vec::with_capacity(datas.count.min(0).max(0x1000) as usize);

            for index in 0..datas.count as usize {
                let data = datas.datas.add(index).read();
                let size = datas.sizes.add(index).read();
                datas_ret.push(from_ptr(data, size));
            }

            wkeUtilRelasePrintPdfDatas.unwrap()(ptr);
            Ok(datas_ret)
        }
    }

    /// 打印为位图
    pub fn print_to_bitmap(&self, size: Size) -> Result<Vec<u8>> {
        unsafe {
            let settings = wkeScreenshotSettings {
                structSize: std::mem::size_of::<wkeScreenshotSettings>() as i32,
                width: size.width,
                height: size.height,
            };
            let mem = wkePrintToBitmap.unwrap()(self.webview, self.frame, &settings);
            if mem.is_null() {
                return Err(Error::InvalidReference);
            }
            let data = from_mem(mem);

            Ok(data)
        }
    }

    /// 获取内容
    pub fn get_content_as_markup(&self) -> Result<String> {
        unsafe {
            from_cstr_ptr(wkeGetContentAsMarkup.unwrap()(
                self.webview,
                self.frame,
                null_mut(),
            ))
        }
    }

    // pub async fn popup_dialog_and_download()
    // {
    //     todo!()
    // }

    // pub async fn get_pdf_page_data(&self) -> Vec<u8> {}
}
