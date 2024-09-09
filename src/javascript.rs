use crate::{
    error::{Error, Result},
    utils::to_cstr_ptr,
    webframe::WebFrame,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    future::Future,
    pin::Pin,
};
use wke_sys::{
    jsCall, jsString, jsUndefined, jsValue, wkeGetGlobalExecByFrame, wkeRunJsByFrame,
    wkeWebFrameHandle,
};

#[derive(Debug, Deserialize)]
struct Request {
    /// 请求名称
    name: String,
    /// 请求数据
    data: Option<Value>,
}

#[derive(Debug, Serialize)]
struct ResponseOk {
    /// 应为true
    successed: bool,
    /// 响应数据
    data: Value,
}

#[derive(Debug, Serialize)]
struct ResponseFail {
    /// 应为false
    successed: bool,
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

pub trait Handler {
    fn handle<'life0, 'life1>(
        &'life0 self,
        request: Request,
    ) -> Pin<Box<dyn Future<Output = Result<Value>> + 'life1>>
    where
        'life0: 'life1;
}

pub struct Handlers {
    frames: RefCell<HashSet<wkeWebFrameHandle>>,
    handlers: RefCell<HashMap<String, Box<dyn Handler>>>,
}

impl Handlers {
    pub(crate) fn on_did_create_script_context(&self, frame: wkeWebFrameHandle) {
        self.frames.borrow_mut().insert(frame);
    }

    pub(crate) fn on_will_release_script_context(&self, frame: wkeWebFrameHandle) {
        self.frames.borrow_mut().remove(&frame);
    }

    pub fn emit<S: Serialize>(name: impl Into<String>, data: S)
    {
        Event {
            name: name.into(),
            data,
            timestamp: Utc::now().to_rfc3339(),
        }
    }

    fn call(frame: WebFrame, caller: jsValue, content: &str) -> Result<()> {
        unsafe {
            let webview = frame.webview();
            if !webview.is_valid() {
                return Err(Error::InvalidReference);
            }
            let frame = frame.native()?;
            if frame.is_null() {
                return Err(Error::InvalidReference);
            }

            let state = wkeGetGlobalExecByFrame.unwrap()(webview.native(), frame);
            if state.is_null() {
                return Ok(());
            }

            let arg = jsString.unwrap()(state, to_cstr_ptr(content)?.to_utf8());
            let mut args = [arg];
            jsCall.unwrap()(state, caller, jsUndefined.unwrap()(), args.as_mut_ptr(), 1);

            Ok(())
        }
    }

    pub async fn handle(
        &self,
        request: Request,
        frame: WebFrame,
        resolve: jsValue,
        reject: jsValue,
    ) -> Result<()> {
        let content = if let Some(handler) = self.handlers.borrow().get(&request.name) {
            match handler.handle(request).await {
                Ok(data) => {
                    let content = serde_json::to_string(&ResponseOk {
                        successed: true,
                        data,
                    })?;
                    Self::call(frame, resolve, &content)?;
                    return Ok(());
                }
                Err(err) => serde_json::to_string(&ResponseFail {
                    successed: false,
                    code: -1,
                    message: format!("handle failed: {}", err),
                })?,
            }
        } else {
            serde_json::to_string(&ResponseFail {
                successed: false,
                code: -2,
                message: "Not Implemented".to_owned(),
            })?
        };
        Self::call(frame, reject, &content)?;

        Ok(())
    }
}
