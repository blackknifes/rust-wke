use super::{JsDataC, JsValue};
use crate::{
    error::{Error, Result},
    utils::{from_bool_int, from_cstr_ptr},
};
use std::ffi::{c_char, c_int};
use wke_sys::*;

fn get_data(es: jsExecState, obj: jsValue) -> Result<*mut JsDataC> {
    unsafe {
        if !from_bool_int(jsIsValidExecState.unwrap()(es))
            || !from_bool_int(jsIsJsValueValid.unwrap()(es, obj))
        {
            return Err(Error::InvalidReference);
        }

        let data = jsGetData.unwrap()(es, obj);
        if data.is_null() {
            return Err(Error::InvalidReference);
        }

        Ok(JsDataC::from_ptr(data))
    }
}

fn get_name_and_data(
    es: jsExecState,
    obj: jsValue,
    name: *const c_char,
) -> Result<(String, *mut JsDataC)> {
    unsafe {
        let data = get_data(es, obj)?;
        let name = from_cstr_ptr(name)?;
        Ok((name, data))
    }
}

pub(crate) extern "C" fn on_get(es: jsExecState, object: jsValue, name: *const c_char) -> jsValue {
    unsafe {
        match get_name_and_data(es, object, name) {
            Ok((name, data)) => data
                .as_mut()
                .unwrap()
                .delegate
                .as_mut()
                .unwrap()
                .get(&name)
                .map(|val| {
                    jsAddRef.unwrap()(es, val.value.value);
                    val.value.value
                })
                .unwrap_or_else(|_| jsUndefined.unwrap()()),
            Err(_) => jsUndefined.unwrap()(),
        }
    }
}

pub(crate) extern "C" fn on_set(
    es: jsExecState,
    object: jsValue,
    name: *const c_char,
    value: jsValue,
) -> bool {
    unsafe {
        match get_name_and_data(es, object, name) {
            Ok((name, data)) => {
                let temp = JsValue::from_native(value);

                data.as_mut()
                    .unwrap()
                    .delegate
                    .as_mut()
                    .unwrap()
                    .set(&name, &temp)
                    .map(|_| true)
                    .unwrap_or(false)
            }
            Err(_) => false,
        }
    }
}

pub(crate) extern "C" fn on_call(
    es: jsExecState,
    object: jsValue,
    args: *mut jsValue,
    arg_count: c_int,
) -> jsValue {
    unsafe {
        match get_data(es, object) {
            Ok(data) => {
                let mut arg_vec = Vec::new();
                for index in 0..arg_count {
                    arg_vec.push(JsValue::from_native(args.add(index as usize).read()));
                }

                let args: Vec<&JsValue> = arg_vec.iter().collect();

                data.as_mut()
                    .unwrap()
                    .delegate
                    .as_mut()
                    .unwrap()
                    .call(&args)
                    .map(|val| {
                        jsAddRef.unwrap()(es, val.value.value);
                        val.value.value
                    })
                    .unwrap_or_else(|_| jsUndefined.unwrap()())
            }
            Err(_) => jsUndefined.unwrap()(),
        }
    }
}

pub(crate) extern "C" fn on_finalize(data: *mut jsData) {
    unsafe {
        let data = JsDataC::from_ptr(data);
        if let Err(err) = data.as_mut().unwrap().delegate.as_mut().unwrap().finalize() {
            log::error!("js data finalize failed: {}", err);
        }
        // 释放内存
        let _ = Box::from_raw(data);
    }
}
