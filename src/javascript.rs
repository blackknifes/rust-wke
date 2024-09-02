use super::webview::WebView;
use crate::error::{Error, Result};
use crate::utils::{from_bool_int, from_cstr_ptr, from_mem, to_cstr16_ptr, to_cstr_ptr};
use wke_sys::*;

pub struct ExecState {
    state: jsExecState,
}

pub struct JsValue {
    state: Option<jsExecState>,
    value: jsValue,
}

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum JsType {
    Number = _jsType_JSTYPE_NUMBER,
    String = _jsType_JSTYPE_STRING,
    Boolean = _jsType_JSTYPE_BOOLEAN,
    Object = _jsType_JSTYPE_OBJECT,
    Function = _jsType_JSTYPE_FUNCTION,
    Undefined = _jsType_JSTYPE_UNDEFINED,
    Array = _jsType_JSTYPE_ARRAY,
    Null = _jsType_JSTYPE_NULL,
}

impl ExecState {
    pub(crate) fn from_native(state: jsExecState) -> Self {
        Self { state }
    }
    pub(crate) fn new_value(&self, value: jsValue) -> JsValue {
        JsValue {
            state: Some(self.state),
            value,
        }
    }

    pub fn get_webview(&self) -> Option<WebView> {
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

    pub fn has_exception(&self) -> bool {
        unsafe { !jsGetLastErrorIfException.unwrap()(self.state).is_null() }
    }

    pub fn eval(&self, script: &str) -> Result<JsValue> {
        unsafe {
            let value = jsEval.unwrap()(self.state, to_cstr_ptr(script)?.to_utf8());

            Ok(self.new_value(value))
        }
    }

    pub fn eval_in_closure(&self, script: &str) -> Result<JsValue> {
        unsafe {
            let value = jsEvalExW.unwrap()(self.state, (&to_cstr16_ptr(script)).as_ptr(), true);

            Ok(self.new_value(value))
        }
    }

    pub fn arg_count(&self) -> i32 {
        unsafe { jsArgCount.unwrap()(self.state) }
    }

    pub fn arg(&self, index: i32) -> JsValue {
        unsafe { self.new_value(jsArg.unwrap()(self.state, index)) }
    }

    pub fn global(&self) -> JsValue {
        unsafe { self.new_value(jsGlobalObject.unwrap()(self.state)) }
    }

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

    pub fn throw(&self, exception: &str) -> Result<JsValue> {
        unsafe {
            let value = jsThrowException.unwrap()(self.state, to_cstr_ptr(exception)?.to_utf8());
            Ok(self.new_value(value))
        }
    }
}

impl JsValue {
    pub(crate) fn from_native_with_state(state: jsExecState, value: jsValue) -> Self {
        Self {
            state: Some(state),
            value,
        }
    }
    pub(crate) fn from_native(value: jsValue) -> Self {
        Self { state: None, value }
    }

    pub fn from_int(value: i32) -> Self {
        unsafe { Self::from_native(jsInt.unwrap()(value)) }
    }

    pub fn from_double(value: f64) -> Self {
        unsafe { Self::from_native(jsDouble.unwrap()(value)) }
    }

    pub fn from_str(state: &ExecState, str: &str) -> Result<Self> {
        unsafe {
            Ok(Self::from_native(jsString.unwrap()(
                state.state,
                to_cstr_ptr(str)?.to_utf8(),
            )))
        }
    }

    pub fn from_data(state: &ExecState, data: &[u8]) -> Self {
        unsafe {
            Self::from_native(jsArrayBuffer.unwrap()(
                state.state,
                data.as_ptr() as *const i8,
                data.len(),
            ))
        }
    }

    pub fn object(state: &ExecState) -> Self {
        unsafe { Self::from_native(jsEmptyObject.unwrap()(state.state)) }
    }

    pub fn array(state: &ExecState) -> Self {
        unsafe { Self::from_native(jsEmptyArray.unwrap()(state.state)) }
    }

    pub fn is_valid(&self, state: &ExecState) -> bool {
        unsafe { from_bool_int(jsIsJsValueValid.unwrap()(state.state, self.value)) }
    }

    pub fn is_boolean(&self) -> bool {
        unsafe { from_bool_int(jsIsBoolean.unwrap()(self.value)) }
    }

    pub fn is_number(&self) -> bool {
        unsafe { from_bool_int(jsIsNumber.unwrap()(self.value)) }
    }

    pub fn is_string(&self) -> bool {
        unsafe { from_bool_int(jsIsString.unwrap()(self.value)) }
    }

    pub fn is_object(&self) -> bool {
        unsafe { from_bool_int(jsIsObject.unwrap()(self.value)) }
    }

    pub fn is_true(&self) -> bool {
        unsafe { from_bool_int(jsIsTrue.unwrap()(self.value)) }
    }

    pub fn is_false(&self) -> bool {
        unsafe { from_bool_int(jsIsFalse.unwrap()(self.value)) }
    }

    pub fn is_null(&self) -> bool {
        unsafe { from_bool_int(jsIsNull.unwrap()(self.value)) }
    }

    pub fn is_undefined(&self) -> bool {
        unsafe { from_bool_int(jsIsUndefined.unwrap()(self.value)) }
    }

    pub fn is_function(&self) -> bool {
        unsafe { from_bool_int(jsIsFunction.unwrap()(self.value)) }
    }

    pub fn is_array(&self) -> bool {
        unsafe { from_bool_int(jsIsArray.unwrap()(self.value)) }
    }

    #[allow(non_upper_case_globals)]
    #[allow(non_snake_case)]
    pub fn get_type(&self) -> JsType {
        unsafe {
            match jsTypeOf.unwrap()(self.value) {
                _jsType_JSTYPE_NUMBER => JsType::Number,
                _jsType_JSTYPE_STRING => JsType::String,
                _jsType_JSTYPE_BOOLEAN => JsType::Boolean,
                _jsType_JSTYPE_OBJECT => JsType::Object,
                _jsType_JSTYPE_FUNCTION => JsType::Function,
                _jsType_JSTYPE_ARRAY => JsType::Array,
                _jsType_JSTYPE_NULL => JsType::Null,
                _ => JsType::Undefined,
            }
        }
    }

    pub fn to_int(&self, state: &ExecState) -> i32 {
        unsafe { jsToInt.unwrap()(state.state, self.value) }
    }

    pub fn to_double(&self, state: &ExecState) -> f64 {
        unsafe { jsToDouble.unwrap()(state.state, self.value) }
    }

    pub fn to_boolean(&self, state: &ExecState) -> bool {
        unsafe { from_bool_int(jsToBoolean.unwrap()(state.state, self.value)) }
    }

    pub fn get_array_buffer(&self, state: &ExecState) -> Result<Vec<u8>> {
        unsafe {
            let mem = jsGetArrayBuffer.unwrap()(state.state, self.value);
            if mem.is_null() {
                return Err(Error::TypeMismatch);
            }

            let data = from_mem(mem);
            wkeFreeMemBuf.unwrap()(mem);

            Ok(data)
        }
    }

    pub fn to_string(&self, state: &ExecState) -> Result<String> {
        unsafe { from_cstr_ptr(jsToTempString.unwrap()(state.state, self.value)) }
    }

    pub fn to_data(&self, state: &ExecState) -> Vec<u8> {
        unsafe {
            let mem = jsGetArrayBuffer.unwrap()(state.state, self.value);
            from_mem(mem)
        }
    }

    pub fn len(&self, state: &ExecState) -> i32 {
        unsafe { jsGetLength.unwrap()(state.state, self.value) }
    }

    pub fn set_len(&self, state: &ExecState, len: i32) {
        unsafe { jsSetLength.unwrap()(state.state, self.value, len as i32) }
    }

    pub fn keys(&self, state: &ExecState) -> Result<Vec<String>> {
        unsafe {
            let keys = jsGetKeys.unwrap()(state.state, self.value).read();
            let mut values = Vec::with_capacity(keys.length as usize);
            for i in 0..keys.length {
                let str = from_cstr_ptr(*keys.keys.add(i as usize))?;
                values.push(str);
            }

            Ok(values)
        }
    }

    pub fn set_at(&self, state: &ExecState, index: i32, value: &JsValue) {
        unsafe {
            jsSetAt.unwrap()(state.state, self.value, index, value.value);
        }
    }

    pub fn get_at(&self, state: &ExecState, index: i32) -> JsValue {
        unsafe {
            let value = jsGetAt.unwrap()(state.state, self.value, index);
            JsValue { value }
        }
    }

    pub fn set(&self, state: &ExecState, name: &str, value: &JsValue) -> Result<()> {
        unsafe {
            jsSet.unwrap()(
                state.state,
                self.value,
                to_cstr_ptr(name)?.to_utf8(),
                value.value,
            );
            Ok(())
        }
    }

    pub fn get(&self, state: &ExecState, name: &str) -> Result<JsValue> {
        unsafe {
            let value = jsGet.unwrap()(state.state, self.value, to_cstr_ptr(name)?.to_utf8());
            Ok(JsValue { value })
        }
    }

    pub fn delete(&self, state: &ExecState, key: &str) -> Result<()> {
        unsafe {
            jsDeleteObjectProp.unwrap()(state.state, self.value, to_cstr_ptr(key)?.to_utf8());
            Ok(())
        }
    }

    pub fn call(
        &self,
        state: &ExecState,
        thiz: Option<&JsValue>,
        args: &[&JsValue],
    ) -> Result<JsValue> {
        unsafe {
            let thiz = match thiz {
                Some(val) => val.value,
                None => jsUndefined.unwrap()(),
            };
            let mut args: Vec<jsValue> = args.iter().map(|val| val.value).collect();
            let value = jsCall.unwrap()(
                state.state,
                self.value,
                thiz,
                (&mut args).as_mut_ptr(),
                args.len() as i32,
            );

            if state.has_exception() {
                return Err(Error::JsCallException);
            }

            Ok(JsValue { value })
        }
    }
}

impl Drop for JsValue {
    fn drop(&mut self) {
        unsafe {
            if let Some(state) = self.state {
                jsReleaseRef.unwrap()(state, self.value);
            }
        }
    }
}
