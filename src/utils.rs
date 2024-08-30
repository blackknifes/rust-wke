use crate::error::{Error, Result};
use std::ffi::{c_char, CStr, CString};
use wke_sys::{wkeFreeMemBuf, wkeMemBuf, BOOL};

pub(crate) unsafe fn to_cstr16_ptr(str: &str) -> Vec<u16> {
    let mut str_u16 = str.encode_utf16().collect::<Vec<u16>>();
    str_u16.push(0);
    return str_u16;
}

pub(crate) struct Utf8(CString);
impl Utf8 {
    pub(crate) fn to_utf8(&self) -> *const c_char {
        self.0.as_c_str().as_ptr()
    }
}

pub(crate) unsafe fn to_cstr_ptr(str: &str) -> Result<Utf8> {
    Ok(Utf8(CString::new(str)?))
}

pub(crate) unsafe fn from_cstr_ptr(str: *const c_char) -> Result<String> {
    let str = CStr::from_ptr(str)
        .to_str()
        .map_err(|err| Error::other(err))?
        .to_owned();
    Ok(str)
}

#[allow(dead_code)]
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
