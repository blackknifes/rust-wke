use super::webview::WebView;
use crate::error::{Error, Result};
use crate::utils::{from_bool_int, from_cstr_ptr, from_mem, to_cstr16_ptr, to_cstr_ptr};
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use wke_sys::*;

thread_local! {
    static ENTERED_CONTEXT: RefCell<Option<Rc<ContextHolderInner>>> = RefCell::new(None);
}

struct ContextHolderInner {
    state: jsExecState,
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
    pub(crate) fn new(state: jsExecState) -> Self {
        let inner = ENTERED_CONTEXT.with_borrow_mut(|context| {
            let mut inner = ContextHolderInner {
                state,
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

/// js类型
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum JsType {
    Number,
    String,
    Boolean,
    Object,
    Function,
    Array,
    Null,
    Undefined,
}

impl JsType {
    #[allow(non_upper_case_globals)]
    pub(crate) fn from_native(js_type: jsType) -> Self {
        match js_type {
            _jsType_JSTYPE_NUMBER => Self::Number,
            _jsType_JSTYPE_STRING => Self::String,
            _jsType_JSTYPE_BOOLEAN => Self::Boolean,
            _jsType_JSTYPE_OBJECT => Self::Object,
            _jsType_JSTYPE_FUNCTION => Self::Function,
            _jsType_JSTYPE_ARRAY => Self::Array,
            _jsType_JSTYPE_NULL => Self::Null,
            _ => Self::Undefined,
        }
    }

    #[allow(unused)]
    pub(crate) fn into_native(&self) -> jsType {
        match self {
            JsType::Number => _jsType_JSTYPE_NUMBER,
            JsType::String => _jsType_JSTYPE_STRING,
            JsType::Boolean => _jsType_JSTYPE_BOOLEAN,
            JsType::Object => _jsType_JSTYPE_OBJECT,
            JsType::Function => _jsType_JSTYPE_FUNCTION,
            JsType::Array => _jsType_JSTYPE_ARRAY,
            JsType::Null => _jsType_JSTYPE_NULL,
            JsType::Undefined => _jsType_JSTYPE_UNDEFINED,
        }
    }
}

/// js上下文环境
pub struct Context {
    state: jsExecState,
}

impl Context {
    /// 从原生类型创建
    pub(crate) fn from_native(state: jsExecState) -> Self {
        Self { state }
    }

    /// 获取当前环境
    pub fn current() -> Result<Context> {
        ENTERED_CONTEXT.with_borrow(|context: &Option<Rc<ContextHolderInner>>| {
            context
                .clone()
                .map(|ctx| Self::from_native(ctx.state))
                .ok_or_else(|| Error::JsContextNotEntered)
        })
    }

    /// 进入环境
    pub fn enter(&self) -> ContextHolder {
        ContextHolder::new(self.state)
    }

    /// 是否已进入该环境
    pub fn is_entered(&self) -> bool {
        ENTERED_CONTEXT.with_borrow(|context| {
            let mut cur = context.clone();
            while let Some(ctx) = cur {
                if self.state == ctx.state {
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
                self.state == ctx.state
            } else {
                false
            }
        })
    }

    /// 获取当前webview窗口
    pub fn webview(&self) -> Option<WebView> {
        unsafe {
            let webview = jsGetWebView.unwrap()(self.state);
            if webview.is_null() {
                return None;
            }
            match WebView::from_native(webview) {
                Ok(webview) => Some(webview),
                Err(_) => None,
            }
        }
    }

    /// 查询是否有异常
    pub fn has_exception(&self) -> bool {
        unsafe { !jsGetLastErrorIfException.unwrap()(self.state).is_null() }
    }

    /// 使用eval执行脚本
    pub fn eval(&self, script: &str) -> Result<JsValue> {
        unsafe {
            let value = jsEval.unwrap()(self.state, to_cstr_ptr(script)?.to_utf8());

            Ok(JsValue::from_native(value))
        }
    }

    /// 在包裹的函数内执行eval脚本
    pub fn eval_in_closure(&self, script: &str) -> Result<JsValue> {
        unsafe {
            let value = jsEvalExW.unwrap()(self.state, (&to_cstr16_ptr(script)).as_ptr(), true);

            Ok(JsValue::from_native(value))
        }
    }

    /// 获取当前参数个数
    pub fn arg_count(&self) -> i32 {
        unsafe { jsArgCount.unwrap()(self.state) }
    }

    /// 获取指定索引参数
    pub fn arg(&self, index: i32) -> JsValue {
        unsafe { JsValue::from_native(jsArg.unwrap()(self.state, index)) }
    }

    /// 获取全局对象
    pub fn global(&self) -> JsValue {
        unsafe { JsValue::from_native(jsGlobalObject.unwrap()(self.state)) }
    }

    /// 获取调用堆栈
    pub fn callstack(&self) -> Option<String> {
        unsafe {
            let str = jsGetCallstack.unwrap()(self.state);
            if str.is_null() {
                return None;
            }
            if let Ok(str) = from_cstr_ptr(str) {
                return Some(str);
            }

            None
        }
    }

    /// 抛出异常
    pub fn throw(&self, exception: &str) -> Result<JsValue> {
        unsafe {
            let value = jsThrowException.unwrap()(self.state, to_cstr_ptr(exception)?.to_utf8());
            Ok(JsValue::from_native(value))
        }
    }
}

/// js变量
pub struct JsValue {
    value: jsValue,
}

impl JsValue {
    /// 从原生创建临时js变量
    pub(crate) fn from_native(value: jsValue) -> Self {
        Self { value }
    }

    /// 从原生创建持久对象
    pub(crate) fn from_native_with_entered(value: jsValue) -> Result<JsValuePerssist> {
        unsafe {
            let state = Context::current()?.state;
            jsAddRef.unwrap()(state, value);

            Ok(JsValuePerssist {
                state,
                value: Rc::new(Self::from_native(value)),
            })
        }
    }

    /// 维持对象
    pub fn perssist(self) -> Result<JsValuePerssist> {
        Self::from_native_with_entered(self.value)
    }

    /// 创建null对象
    pub fn null() -> Result<JsValuePerssist> {
        unsafe { Self::from_native_with_entered(jsNull.unwrap()()) }
    }

    /// 创建undefined对象
    pub fn undefined() -> Result<JsValuePerssist> {
        unsafe { Self::from_native_with_entered(jsUndefined.unwrap()()) }
    }

    /// 创建空对象
    pub fn object() -> Result<JsValuePerssist> {
        unsafe { Self::from_native_with_entered(jsEmptyObject.unwrap()(Context::current()?.state)) }
    }

    /// 创建空数组
    pub fn array() -> Result<JsValuePerssist> {
        unsafe { Self::from_native_with_entered(jsEmptyArray.unwrap()(Context::current()?.state)) }
    }

    /// 从bool值创建
    pub fn from_bool(val: bool) -> Result<JsValuePerssist> {
        unsafe {
            Self::from_native_with_entered(if val {
                jsTrue.unwrap()()
            } else {
                jsFalse.unwrap()()
            })
        }
    }

    /// 从int值创建
    pub fn from_int(value: i32) -> Result<JsValuePerssist> {
        unsafe { Self::from_native_with_entered(jsInt.unwrap()(value)) }
    }

    /// 从f64值创建
    pub fn from_f64(value: f64) -> Result<JsValuePerssist> {
        unsafe { Self::from_native_with_entered(jsDouble.unwrap()(value)) }
    }

    /// 从str创建
    pub fn from_str(str: &str) -> Result<JsValuePerssist> {
        unsafe {
            Self::from_native_with_entered(jsString.unwrap()(
                Context::current()?.state,
                to_cstr_ptr(str)?.to_utf8(),
            ))
        }
    }

    /// 从二进制数据创建
    pub fn from_data(data: &[u8]) -> Result<JsValuePerssist> {
        unsafe {
            Self::from_native_with_entered(jsArrayBuffer.unwrap()(
                Context::current()?.state,
                data.as_ptr() as *const i8,
                data.len(),
            ))
        }
    }

    /// 当前引用是否有效
    pub fn is_valid(&self) -> Result<bool> {
        unsafe {
            Ok(from_bool_int(jsIsJsValueValid.unwrap()(
                Context::current()?.state,
                self.value,
            )))
        }
    }

    /// 是否为bool类型
    pub fn is_boolean(&self) -> bool {
        unsafe { from_bool_int(jsIsBoolean.unwrap()(self.value)) }
    }

    /// 是否为number
    pub fn is_number(&self) -> bool {
        unsafe { from_bool_int(jsIsNumber.unwrap()(self.value)) }
    }

    /// 是否为string
    pub fn is_string(&self) -> bool {
        unsafe { from_bool_int(jsIsString.unwrap()(self.value)) }
    }

    /// 是否为object
    pub fn is_object(&self) -> bool {
        unsafe { from_bool_int(jsIsObject.unwrap()(self.value)) }
    }

    /// 是否为true
    pub fn is_true(&self) -> bool {
        unsafe { from_bool_int(jsIsTrue.unwrap()(self.value)) }
    }

    /// 是否为false
    pub fn is_false(&self) -> bool {
        unsafe { from_bool_int(jsIsFalse.unwrap()(self.value)) }
    }

    /// 是否为null
    pub fn is_null(&self) -> bool {
        unsafe { from_bool_int(jsIsNull.unwrap()(self.value)) }
    }

    /// 是否为undefined
    pub fn is_undefined(&self) -> bool {
        unsafe { from_bool_int(jsIsUndefined.unwrap()(self.value)) }
    }

    /// 是否为function
    pub fn is_function(&self) -> bool {
        unsafe { from_bool_int(jsIsFunction.unwrap()(self.value)) }
    }

    /// 是否为数组
    pub fn is_array(&self) -> bool {
        unsafe { from_bool_int(jsIsArray.unwrap()(self.value)) }
    }

    /// 获取类型
    pub fn get_type(&self) -> JsType {
        unsafe { JsType::from_native(jsTypeOf.unwrap()(self.value)) }
    }

    /// 转为int
    pub fn to_int(&self) -> Result<i32> {
        unsafe { Ok(jsToInt.unwrap()(Context::current()?.state, self.value)) }
    }

    /// 转为f64
    pub fn to_f64(&self) -> Result<f64> {
        unsafe { Ok(jsToDouble.unwrap()(Context::current()?.state, self.value)) }
    }

    /// 转为bool
    pub fn to_boolean(&self) -> Result<bool> {
        unsafe {
            Ok(from_bool_int(jsToBoolean.unwrap()(
                Context::current()?.state,
                self.value,
            )))
        }
    }

    /// 获取arrayBuffer中的数据
    pub fn get_array_buffer(&self) -> Result<Vec<u8>> {
        unsafe {
            let mem = jsGetArrayBuffer.unwrap()(Context::current()?.state, self.value);
            if mem.is_null() {
                return Err(Error::TypeMismatch);
            }

            let data = from_mem(mem);
            wkeFreeMemBuf.unwrap()(mem);

            Ok(data)
        }
    }

    /// 转为字符串
    pub fn to_string(&self) -> Result<String> {
        unsafe {
            from_cstr_ptr(jsToTempString.unwrap()(
                Context::current()?.state,
                self.value,
            ))
        }
    }

    /// 获取长度
    pub fn len(&self) -> Result<i32> {
        unsafe { Ok(jsGetLength.unwrap()(Context::current()?.state, self.value)) }
    }

    /// 设置数组长度
    pub fn set_len(&self, len: i32) -> Result<()> {
        unsafe {
            jsSetLength.unwrap()(Context::current()?.state, self.value, len as i32);
            Ok(())
        }
    }

    /// 获取object 所有键值
    pub fn keys(&self) -> Result<Vec<String>> {
        unsafe {
            let keys = jsGetKeys.unwrap()(Context::current()?.state, self.value).read();
            let mut values = Vec::with_capacity(keys.length as usize);
            for i in 0..keys.length {
                let str = from_cstr_ptr(*keys.keys.add(i as usize))?;
                values.push(str);
            }

            Ok(values)
        }
    }

    /// 设置数组元素
    pub fn set_at(&self, index: i32, value: &JsValue) -> Result<()> {
        unsafe {
            jsSetAt.unwrap()(Context::current()?.state, self.value, index, value.value);
            Ok(())
        }
    }

    /// 获取数组元素
    pub fn get_at(&self, index: i32) -> Result<JsValue> {
        unsafe {
            let value = jsGetAt.unwrap()(Context::current()?.state, self.value, index);
            Ok(JsValue::from_native(value))
        }
    }

    /// 设置对象元素
    pub fn set(&self, name: &str, value: &JsValue) -> Result<()> {
        unsafe {
            jsSet.unwrap()(
                Context::current()?.state,
                self.value,
                to_cstr_ptr(name)?.to_utf8(),
                value.value,
            );
            Ok(())
        }
    }

    /// 获取对象元素
    pub fn get(&self, name: &str) -> Result<JsValue> {
        unsafe {
            let value = jsGet.unwrap()(
                Context::current()?.state,
                self.value,
                to_cstr_ptr(name)?.to_utf8(),
            );
            Ok(JsValue::from_native(value))
        }
    }

    /// 删除对象元素
    pub fn delete(&self, key: &str) -> Result<()> {
        unsafe {
            jsDeleteObjectProp.unwrap()(
                Context::current()?.state,
                self.value,
                to_cstr_ptr(key)?.to_utf8(),
            );
            Ok(())
        }
    }

    /// 调用函数
    pub fn call(&self, thiz: Option<&JsValue>, args: &[&JsValue]) -> Result<JsValue> {
        unsafe {
            let ctx = Context::current()?;
            let thiz = match thiz {
                Some(val) => val.value,
                None => jsUndefined.unwrap()(),
            };
            let mut args: Vec<jsValue> = args.iter().map(|val| val.value).collect();
            let value = jsCall.unwrap()(
                ctx.state,
                self.value,
                thiz,
                (&mut args).as_mut_ptr(),
                args.len() as i32,
            );

            if ctx.has_exception() {
                return Err(Error::JsCallException);
            }

            Ok(JsValue::from_native(value))
        }
    }
}

#[derive(Clone)]
pub struct JsValuePerssist {
    value: Rc<JsValue>,
    state: jsExecState,
}

impl Deref for JsValuePerssist {
    type Target = JsValue;

    fn deref(&self) -> &Self::Target {
        self.value.as_ref()
    }
}

impl Drop for JsValuePerssist {
    fn drop(&mut self) {
        unsafe {
            if from_bool_int(jsIsValidExecState.unwrap()(self.state))
                && from_bool_int(jsIsJsValueValid.unwrap()(self.state, self.value.value))
            {
                jsReleaseRef.unwrap()(self.state, self.value.value);
            }
        }
    }
}
