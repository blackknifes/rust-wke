use crate::error::{Error, Result};
use std::ffi::{c_char, CStr};
use wke_sys::{wkeFreeMemBuf, wkeMemBuf, BOOL};

pub(crate) unsafe fn to_cstr16_ptr(str: &str) -> Vec<u16> {
    let mut str_u16 = str.encode_utf16().collect::<Vec<u16>>();
    str_u16.push(0);
    return str_u16;
}

pub(crate) unsafe fn to_cstr_ptr(str: &str) -> *const i8 {
    return str.as_ptr() as *const i8;
}

pub(crate) unsafe fn from_cstr_ptr(str: *const c_char) -> Result<String> {
    let str = CStr::from_ptr(str)
        .to_str()
        .map_err(|err| Error::other(err))?
        .to_owned();
    Ok(str)
}

pub(crate) fn to_bool_int(value: bool) -> BOOL {
    if value {
        1
    } else {
        0
    }
}

pub(crate) fn from_bool_int(value: BOOL) -> bool {
    if value == 0 {
        false
    } else {
        true
    }
}

pub(crate) fn from_mem(mem: *mut wkeMemBuf) -> Vec<u8> {
    unsafe {
        let info = mem.read();
        let data = Vec::from_raw_parts(info.data as *mut u8, info.length, info.length);
        wkeFreeMemBuf.unwrap()(mem);
        data
    }
}
