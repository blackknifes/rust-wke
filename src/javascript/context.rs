use crate::{error::{Error, Result}, webview::WebView};
use std::{cell::RefCell, rc::Rc};
use wke_sys::*;

thread_local! {
    static ENTERED_CONTEXT: RefCell<Option<Rc<ContextHolderInner>>> = RefCell::new(None);
}

struct ContextHolderInner {
    inner: Rc<ContextInner>,
    pre: Option<Rc<ContextHolderInner>>,
}

/// 上下文环境维持对象, 在该对象的生命周期类，该线程都将在该环境下，除非有新的Holder替代当前Holder
pub struct ContextHolder {
    inner: Rc<ContextHolderInner>,
}

impl Drop for ContextHolder {
    fn drop(&mut self) {
        match self.inner.pre.clone() {
            Some(pre) => ENTERED_CONTEXT.with_borrow_mut(move |context| {
                context.replace(pre);
            }),
            None => ENTERED_CONTEXT.with_borrow_mut(move |context| {
                context.take();
            }),
        };
    }
}

impl ContextHolder {
    pub(crate) fn new(inner: Rc<ContextInner>) -> Self {
        let inner = ENTERED_CONTEXT.with_borrow_mut(|context| {
            let mut inner = ContextHolderInner {
                inner,
                pre: Default::default(),
            };
            inner.pre = context.clone();
            let inner = Rc::new(inner);
            context.replace(inner.clone());
            inner
        });
        Self { inner }
    }
}

struct ContextInner(RefCell<Option<jsExecState>>);

impl ContextInner {
    fn is_valid(&self) -> bool {
        self.0.borrow().is_some()
    }

    fn invalid(&self) {
        self.0.borrow_mut().take();
    }
}

/// js上下文环境
#[derive(Clone)]
pub struct Context(Rc<ContextInner>);

impl Context {
    /// 从原生类型创建
    pub(crate) fn from_native(state: jsExecState) -> Self {
        Self(Rc::new(ContextInner(RefCell::new(Some(state)))))
    }

    /// 获取当前环境
    pub fn current() -> Result<Context> {
        ENTERED_CONTEXT.with_borrow(|holder| {
            let mut holder = holder.clone().ok_or_else(|| Error::JsContextNotEntered)?;

            loop {
                if holder.inner.is_valid() {
                    return Ok(Context(holder.inner.clone()));
                }

                match &holder.pre {
                    Some(pre) => holder = pre.clone(),
                    None => return Err(Error::JsContextNotEntered),
                }
            }
        })
    }

    /// 进入环境
    pub fn enter(&self) -> ContextHolder {
        ContextHolder::new(self.0.clone())
    }

    /// 是否已进入该环境
    pub fn is_entered(&self) -> bool {
        ENTERED_CONTEXT.with_borrow(|context| {
            let mut cur = context.clone();
            while let Some(ctx) = cur {
                if Rc::ptr_eq(&self.0, &ctx.inner) {
                    return true;
                }
                cur = ctx.pre.clone();
            }
            false
        })
    }

    /// 是否为当前环境
    pub fn is_current(&self) -> bool {
        ENTERED_CONTEXT.with_borrow(|context| {
            if let Some(ctx) = context {
                Rc::ptr_eq(&self.0, &ctx.inner)
            } else {
                false
            }
        })
    }

    fn state(&self) -> Result<jsExecState> {
        if let Some(state) = self.0 .0.borrow().as_ref() {
            return Ok(state.clone());
        }
        Err(Error::InvalidReference)
    }

    /// 获取当前webview窗口
    pub fn webview(&self) -> Option<WebView> {
        unsafe {
            match self.state() {
                Ok(state) => {
                    let webview = jsGetWebView.unwrap()(state);
                    if webview.is_null() {
                        return None;
                    }
                    match WebView::from_native(webview) {
                        Ok(webview) => Some(webview),
                        Err(_) => None,
                    }
                }
                Err(_) => None,
            }
        }
    }

    /// 查询是否有异常
    pub fn has_exception(&self) -> bool {
        unsafe {
            match self.state() {
                Ok(state) => !jsGetLastErrorIfException.unwrap()(state).is_null(),
                Err(_) => false,
            }
        }
    }

    /// 使用eval执行脚本
    pub fn eval(&self, script: &str) -> Result<JsValue> {
        unsafe {
            let value = jsEval.unwrap()(self.state()?, to_cstr_ptr(script)?.to_utf8());
            if self.has_exception() {
                return Err(Error::JsCallException);
            }
            Ok(JsValue::from_native(value))
        }
    }

    /// 在包裹的函数内执行eval脚本
    pub fn eval_in_closure(&self, script: &str) -> Result<JsValue> {
        unsafe {
            let value = jsEvalExW.unwrap()(self.state()?, (&to_cstr16_ptr(script)).as_ptr(), true);

            Ok(JsValue::from_native(value))
        }
    }

    /// 获取当前参数个数
    pub fn arg_count(&self) -> i32 {
        unsafe {
            match self.state() {
                Ok(state) => jsArgCount.unwrap()(state),
                Err(_) => 0,
            }
        }
    }

    /// 获取指定索引参数
    pub fn arg(&self, index: i32) -> Result<JsValue> {
        unsafe {
            let state = self.state()?;
            let value = jsArg.unwrap()(state, index);
            check_js_value(state, value)?;
            Ok(JsValue::from_native(value))
        }
    }

    /// 获取全局对象
    pub fn global(&self) -> Result<JsValue> {
        unsafe {
            let state = self.state()?;
            let value = jsGlobalObject.unwrap()(state);
            check_js_value(state, value)?;
            Ok(JsValue::from_native(value))
        }
    }

    /// 获取调用堆栈
    pub fn callstack(&self) -> Option<String> {
        unsafe {
            match self.state() {
                Ok(state) => {
                    let str = jsGetCallstack.unwrap()(state);
                    if str.is_null() {
                        return None;
                    }
                    if let Ok(str) = from_cstr_ptr(str) {
                        return Some(str);
                    }

                    None
                }
                Err(_) => None,
            }
        }
    }

    /// 抛出异常
    pub fn throw(&self, exception: &str) -> Result<JsValue> {
        unsafe {
            let state = self.state()?;
            let value = jsThrowException.unwrap()(state, to_cstr_ptr(exception)?.to_utf8());
            check_js_value(state, value)?;
            Ok(JsValue::from_native(value))
        }
    }
}
