use crate::error::{Error, Result};
use std::{
    ffi::{c_char, CStr, CString},
    os::raw::c_void,
    sync::atomic::AtomicUsize,
};
use wke_sys::*;

const ID_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

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

    pub(crate) fn len(&self) -> usize {
        self.0.as_c_str().count_bytes()
    }

    pub(crate) fn copy_to(&self, buf: &mut [i8]) {
        unsafe {
            let cstr = self.to_utf8();
            for index in 0..self.len().min(buf.len()) {
                buf[index] = cstr.add(index).read();
            }
        }
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

pub(crate) fn from_wkestring(str: wkeString) -> String {
    unsafe {
        match from_cstr_ptr(wkeToString.unwrap()(str)) {
            Ok(str) => str,
            Err(err) => {
                log::error!("str convert rust failed: {}", err);
                "".to_owned()
            }
        }
    }
}

pub(crate) fn set_wkestring(wke: wkeString, str: &str) {
    unsafe {
        let str = match to_cstr_ptr(str) {
            Ok(str) => str,
            Err(err) => {
                log::error!("str convert c string failed: {}", err);
                to_cstr_ptr("").unwrap()
            }
        };
        wkeSetString.unwrap()(wke, str.to_utf8(), str.len());
    }
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

pub(crate) fn from_mem(mem: *const wkeMemBuf) -> Vec<u8> {
    unsafe {
        let info = mem.read();
        let data = Vec::from_raw_parts(info.data as *mut u8, info.length, info.length);
        data
    }
}

pub(crate) fn from_ptr(ptr: *const c_void, size: usize) -> Vec<u8> {
    unsafe { Vec::from_raw_parts(ptr as *mut u8, size, size) }
}

#[allow(dead_code)]
pub(crate) fn check_ptr<T>(value: *const T) -> Result<()> {
    if value.is_null() {
        return Err(Error::InvalidReference);
    }
    Ok(())
}

pub(crate) fn next_id() -> usize {
    ID_SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::AcqRel)
}

pub(crate) struct OnDrop(Option<Box<dyn FnOnce() + 'static>>);

impl OnDrop {
    pub fn new<FN>(func: FN) -> Self
    where
        FN: FnOnce() + 'static,
    {
        OnDrop(Some(Box::new(func)))
    }

    #[allow(dead_code)]
    pub fn abort(&mut self) {
        self.0.take();
    }
}

impl Drop for OnDrop {
    fn drop(&mut self) {
        match self.0.take() {
            Some(cb) => cb(),
            None => todo!(),
        }
    }
}
