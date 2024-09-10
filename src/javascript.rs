use crate::{
    error::{Error, Result},
    utils::{from_cstr_ptr, to_cstr_ptr, OnDrop},
    webframe::WebFrame,
};
use chrono::Utc;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    future::Future,
    mem::offset_of,
    pin::Pin,
    ptr::null_mut,
    rc::Rc,
};
use wke_sys::*;

const UNKOWN_ERROR_RESPONSE: &str = r#"{"code":-1,"message":"handle failed: unknown error"}"#;
const ENCODE_FAILED_RESPONSE: &str =
    r#"{"code":-1,"message":"handle failed: string encode failed"}"#;

#[derive(Debug, Deserialize)]
pub struct Request {
    /// 请求名称
    name: String,
    /// 请求数据
    #[serde(default)]
    data: Value,
}

#[derive(Debug, Serialize)]
struct QueryError {
    /// 错误代码
    code: i32,
    /// 响应数据
    message: String,
}

#[derive(Debug, Serialize)]
struct Event<S: Serialize> {
    /// 实践名
    name: String,
    /// 携带数据
    data: S,
    /// 时间戳
    timestamp: i64,
}

thread_local! {
    static GLOBAL_HANDLERS: Rc<Handlers> = Rc::new(Default::default());
}

pub type HandleCallback =
    Rc<dyn Fn(Request) -> Pin<Box<dyn Future<Output = Result<Value>> + 'static>> + 'static>;

#[derive(Default)]
pub struct Handlers {
    frames: RefCell<HashSet<WebFrame>>,
    handlers: RefCell<HashMap<String, HandleCallback>>,
}

#[repr(C)]
struct JsDataC {
    jsdatac: jsData,
    callback: Box<dyn Fn(jsExecState, jsValue, &[jsValue]) -> jsValue>,
}

impl JsDataC {
    unsafe extern "C" fn on_finalize(data: *mut tagjsData) {
        Self::from_raw(data);
    }

    unsafe extern "C" fn on_call(
        es: jsExecState,
        object: jsValue,
        args: *mut jsValue,
        arg_count: ::std::os::raw::c_int,
    ) -> jsValue {
        let jsdata = jsGetData.unwrap()(es, object);
        if jsdata.is_null() {
            return jsUndefined.unwrap()();
        }
        let jsdata = JsDataC::from_ptr(jsdata).as_ref().unwrap();

        let mut rargs = Vec::with_capacity(arg_count as usize);
        for index in 0..arg_count as usize {
            rargs.push(args.add(index).read());
        }

        return jsdata.callback.as_ref()(es, object, &rargs);
    }

    fn new<FN>(name: &str, cb: FN) -> Self
    where
        FN: Fn(jsExecState, jsValue, &[jsValue]) -> jsValue + 'static,
    {
        unsafe {
            let mut typename = [0; 100];
            to_cstr_ptr(name).unwrap().copy_to(&mut typename);

            Self {
                jsdatac: jsData {
                    typeName: typename,
                    propertyGet: None,
                    propertySet: None,
                    finalize: Some(Self::on_finalize),
                    callAsFunction: Some(Self::on_call),
                },
                callback: Box::new(cb),
            }
        }
    }

    fn into_raw(boxed: Box<Self>) -> *mut jsData {
        let ptr = Box::into_raw(boxed);
        ptr as *mut jsData
    }

    fn from_ptr(ptr: *mut jsData) -> *mut JsDataC {
        let offset = offset_of!(JsDataC, jsdatac);
        ptr.wrapping_sub(offset) as *mut JsDataC
    }

    fn from_raw(ptr: *mut jsData) -> Box<JsDataC> {
        unsafe {
            let jsdata = Self::from_ptr(ptr);
            Box::from_raw(jsdata)
        }
    }
}

impl Handlers {
    unsafe fn get_state(frame: WebFrame) -> jsExecState {
        let webview = frame.webview().native();
        let frame = match frame.native() {
            Ok(frame) => frame,
            Err(_) => return null_mut(),
        };
        wkeGetGlobalExecByFrame.unwrap()(webview, frame)
    }

    unsafe fn on_queryc(frame: WebFrame, state: jsExecState, args: &[jsValue]) -> Result<jsValue> {
        if args.len() < 3 {
            return Err(Error::msg("nativeQuery arguments mismatch"));
        }
        let content = args[0];
        let resolve = args[1];
        let reject = args[2];

        let content = from_cstr_ptr(jsToTempString.unwrap()(state, content))?;
        let request = serde_json::from_str::<Request>(&content)?;

        let handler = GLOBAL_HANDLERS.with(|handler| handler.clone());

        jsAddRef.unwrap()(state, resolve);
        jsAddRef.unwrap()(state, reject);
        tokio::task::spawn_local(async move {
            let frame_drop = frame.clone();
            let _ondrop = OnDrop::new(move || {
                let state = Self::get_state(frame_drop);
                if state.is_null() {
                    return;
                }
                jsReleaseRef.unwrap()(state, resolve);
                jsReleaseRef.unwrap()(state, reject);
            });

            let result = handler.on_query(request).await;
            let state = Self::get_state(frame);
            if state.is_null() {
                return;
            }

            // 处理结果封包
            let (func, json) = match result {
                Ok(json) => match to_cstr_ptr(&json) {
                    Ok(json) => (resolve, json),
                    Err(_) => (reject, to_cstr_ptr(ENCODE_FAILED_RESPONSE).unwrap()),
                },
                Err(err) => {
                    let message = format!("handle failed: {}", err);
                    let json = serde_json::to_string(&QueryError { code: -1, message })
                        .unwrap_or_else(|_| UNKOWN_ERROR_RESPONSE.to_string());
                    let json = to_cstr_ptr(&json)
                        .unwrap_or_else(|_| to_cstr_ptr(UNKOWN_ERROR_RESPONSE).unwrap());
                    (reject, json)
                }
            };

            // 准备参数
            let mut args = [jsString.unwrap()(state, json.to_utf8())];
            // 调用函数
            jsCall.unwrap()(state, func, jsUndefined.unwrap()(), args.as_mut_ptr(), 1);
        });

        Ok(jsUndefined.unwrap()())
    }

    async unsafe fn on_query(&self, request: Request) -> Result<String> {
        let handler = self.handlers.borrow().get(&request.name).cloned();
        let handler = handler.ok_or_else(|| Error::NotImplement)?;
        let result = handler(request).await?;
        Ok(serde_json::to_string(&result)?)
    }

    pub(crate) fn on_did_create_script_context(&self, frame: WebFrame) {
        unsafe {
            let webview = frame.webview().native();
            if let Ok(native_frame) = frame.native() {
                // 将wke对象注入的js全局环境中
                let state = wkeGetGlobalExecByFrame.unwrap()(webview, native_frame);
                if state.is_null() {
                    return;
                }
                let preload = include_str!("preload.js");
                let preload_utf8 = to_cstr_ptr(preload).unwrap();
                let js_preload =
                    wkeRunJsByFrame.unwrap()(webview, native_frame, preload_utf8.to_utf8(), true);
                let js_version = jsString.unwrap()(state, wkeVersionString.unwrap()());

                let frame_cb = frame.clone();
                let datac = Box::new(JsDataC::new("nativeQuery", move |state, _thiz, args| {
                    match Self::on_queryc(frame_cb.clone(), state, args) {
                        Ok(result) => result,
                        Err(err) => {
                            log::error!("nativeQuery failed: {}", err);
                            jsUndefined.unwrap()()
                        }
                    }
                }));
                let ptr = JsDataC::into_raw(datac);
                let js_query = jsFunction.unwrap()(state, ptr);

                let mut js_args = [js_version, js_query];
                jsCall.unwrap()(
                    state,
                    js_preload,
                    jsUndefined.unwrap()(),
                    js_args.as_mut_ptr(),
                    2,
                );

                self.frames.borrow_mut().insert(frame);
            }
        }
    }

    pub(crate) fn on_will_release_script_context(&self, frame: WebFrame) {
        self.frames.borrow_mut().remove(&frame);
    }

    pub fn emit<S: Serialize>(&self, name: impl Into<String>, data: S) -> Result<()> {
        unsafe {
            let content = serde_json::to_string(&Event {
                name: name.into(),
                data,
                timestamp: Utc::now().timestamp_millis(),
            })?;

            let script = format!("window.mb.emit(JSON.parse(`{}`))", content);
            let script_utf8 = to_cstr_ptr(&script)?;

            for frame in self.frames.borrow().iter() {
                let webview = frame.webview();
                if let Ok(frame) = frame.native() {
                    wkeRunJsByFrame.unwrap()(webview.native(), frame, script_utf8.to_utf8(), true);
                }
            }

            Ok(())
        }
    }

    pub fn register<NAME, FN, FUT, REQ, RET>(&self, name: NAME, cb: FN)
    where
        NAME: Into<String>,
        REQ: DeserializeOwned,
        RET: Serialize,
        FUT: Future<Output = Result<RET>> + 'static,
        FN: Fn(REQ) -> FUT + 'static,
    {
        let cb = Rc::new(cb);
        self.handlers
            .borrow_mut()
            .entry(name.into())
            .or_insert_with(move || {
                Rc::new(move |request| {
                    let cb = cb.clone();
                    let fut = async move {
                        let req = serde_json::from_value::<REQ>(request.data)?;
                        let result = cb(req).await?;
                        let value = serde_json::to_value(result)?;

                        Result::Ok(value)
                    };
                    Box::pin(fut)
                })
            });
    }
}

pub(crate) fn on_did_create_script_context(frame: WebFrame) {
    GLOBAL_HANDLERS
        .with(|handler| handler.clone())
        .on_did_create_script_context(frame);
}

pub(crate) fn on_will_release_script_context(frame: WebFrame) {
    GLOBAL_HANDLERS
        .with(|handler| handler.clone())
        .on_will_release_script_context(frame);
}

pub fn register<NAME, FN, FUT, REQ, RET>(name: NAME, cb: FN)
where
    NAME: Into<String>,
    REQ: DeserializeOwned,
    RET: Serialize,
    FUT: Future<Output = Result<RET>> + 'static,
    FN: Fn(REQ) -> FUT + 'static,
{
    GLOBAL_HANDLERS.with(move |handler| handler.register(name, cb));
}
