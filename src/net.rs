use crate::utils::{from_bool_int, from_mem, from_wkestring, to_cstr16_ptr, to_cstr_ptr};
use crate::{
    error::{Error, Result},
    utils::from_cstr_ptr,
};
use std::io::Write;
use std::ops::Range;
use std::{os::raw::c_void, ptr::null_mut};
use wke_sys::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
    Put,
}

impl ToString for Method {
    fn to_string(&self) -> String {
        match self {
            Method::Get => "Get".to_owned(),
            Method::Post => "Post".to_owned(),
            Method::Put => "Put".to_owned(),
        }
    }
}

impl Method {
    #[allow(non_upper_case_globals)]
    pub(crate) fn from_native(method: _wkeRequestType) -> Result<Self> {
        match method {
            _wkeRequestType_kWkeRequestTypeGet => Ok(Self::Get),
            _wkeRequestType_kWkeRequestTypePost => Ok(Self::Post),
            _wkeRequestType_kWkeRequestTypePut => Ok(Self::Put),
            _ => Err(Error::InvalidEnum),
        }
    }
}

pub enum PostDataElement {
    Data(Vec<u8>),
    File(String),
    FileRange((String, Range<usize>)),
}

pub struct PostData {
    pub datas: Vec<PostDataElement>,
    pub dirty: bool,
}

pub struct Job {
    job: wkeNetJob,
}

impl Job {
    pub(crate) fn from_native(job: wkeNetJob) -> Self {
        Self { job }
    }
    pub fn set_mime_type(&self, mime_type: &str) -> Result<()> {
        unsafe {
            wkeNetSetMIMEType.unwrap()(self.job, to_cstr_ptr(mime_type)?.to_utf8());
            Ok(())
        }
    }

    pub fn get_mime_type(&self) -> Result<String> {
        unsafe { from_cstr_ptr(wkeNetGetMIMEType.unwrap()(self.job, null_mut())) }
    }

    pub fn get_referrer(&self) -> Result<String> {
        unsafe { from_cstr_ptr(wkeNetGetReferrer.unwrap()(self.job)) }
    }

    pub fn set_http_request_header(&self, key: &str, val: &str) {
        unsafe {
            wkeNetSetHTTPHeaderField.unwrap()(
                self.job,
                to_cstr16_ptr(key).as_ptr(),
                to_cstr16_ptr(val).as_ptr(),
                false,
            );
        }
    }

    pub fn set_http_response_header(&self, key: &str, val: &str) {
        unsafe {
            wkeNetSetHTTPHeaderField.unwrap()(
                self.job,
                to_cstr16_ptr(key).as_ptr(),
                to_cstr16_ptr(val).as_ptr(),
                true,
            );
        }
    }

    pub fn get_http_request_header(&self, key: &str) -> Result<String> {
        unsafe {
            from_cstr_ptr(wkeNetGetHTTPHeaderField.unwrap()(
                self.job,
                to_cstr_ptr(key)?.to_utf8(),
            ))
        }
    }

    pub fn get_http_response_header(&self, key: &str) -> Result<String> {
        unsafe {
            from_cstr_ptr(wkeNetGetHTTPHeaderFieldFromResponse.unwrap()(
                self.job,
                to_cstr_ptr(key)?.to_utf8(),
            ))
        }
    }

    pub fn set_data(&self, data: &[u8]) {
        unsafe {
            wkeNetSetData.unwrap()(self.job, data.as_ptr() as *mut c_void, data.len() as i32);
        }
    }

    pub fn hook(&self) {
        unsafe {
            wkeNetHookRequest.unwrap()(self.job);
        }
    }

    pub fn get_method(&self) -> Result<Method> {
        unsafe { Method::from_native(wkeNetGetRequestMethod.unwrap()(self.job)) }
    }

    pub fn continue_process(&self) {
        unsafe { wkeNetContinueJob.unwrap()(self.job) }
    }

    pub fn get_url(&self) -> Result<String> {
        unsafe { from_cstr_ptr(wkeNetGetUrlByJob.unwrap()(self.job)) }
    }

    pub fn get_request_raw_http_head(&self) -> Vec<String> {
        unsafe {
            let mut list = wkeNetGetRawHttpHead.unwrap()(self.job) as *mut _wkeSlist;
            let mut headers = Vec::new();

            while !list.is_null() {
                let info = list.as_ref().unwrap();
                if let Ok(str) = from_cstr_ptr(info.data) {
                    headers.push(str);
                } else {
                    log::error!("get_request_raw_http_head: convert str failed");
                }
                list = info.next;
            }
            headers
        }
    }

    pub fn get_response_raw_http_head(&self) -> Vec<String> {
        unsafe {
            let mut list = wkeNetGetRawResponseHead.unwrap()(self.job) as *mut _wkeSlist;
            let mut headers = Vec::new();

            while !list.is_null() {
                let info = list.as_ref().unwrap();
                if let Ok(str) = from_cstr_ptr(info.data) {
                    headers.push(str);
                } else {
                    log::error!("get_request_raw_http_head: convert str failed");
                }
                list = info.next;
            }
            headers
        }
    }

    pub fn cancel(&self) {
        unsafe { wkeNetCancelRequest.unwrap()(self.job) }
    }

    pub fn asyn_commit(&self) -> bool {
        unsafe { from_bool_int(wkeNetHoldJobToAsynCommit.unwrap()(self.job)) }
    }

    pub fn set_url(&self, url: &str) -> Result<()> {
        unsafe {
            wkeNetChangeRequestUrl.unwrap()(self.job, to_cstr_ptr(url)?.to_utf8());
            Ok(())
        }
    }

    #[allow(non_upper_case_globals)]
    pub fn get_post_body(&self) -> Option<PostData> {
        unsafe {
            let ptr = wkeNetGetPostBody.unwrap()(self.job);
            if ptr.is_null() {
                return None;
            }
            let native_eles = ptr.as_ref().unwrap();
            // Vec<PostDataElement>,
            let dirty = native_eles.isDirty;
            let mut datas = Vec::with_capacity(native_eles.elementSize);
            for index in 0..native_eles.elementSize {
                let ele = native_eles.element.add(index).read().as_ref().unwrap();
                let data = match ele.type_ {
                    _wkeHttBodyElementType_wkeHttBodyElementTypeData => {
                        PostDataElement::Data(from_mem(ele.data))
                    }
                    _wkeHttBodyElementType_wkeHttBodyElementTypeFile => {
                        if ele.fileStart == 0 && ele.fileLength == -1 {
                            PostDataElement::File(from_wkestring(ele.filePath))
                        } else {
                            let start = ele.fileStart as usize;
                            let end = if ele.fileLength < 0 {
                                usize::MAX
                            } else {
                                start + ele.fileLength as usize
                            };
                            PostDataElement::FileRange((from_wkestring(ele.filePath), start..end))
                        }
                    }
                    _ => continue,
                };

                datas.push(data);
            }

            wkeNetFreePostBodyElements.unwrap()(ptr);
            Some(PostData { datas, dirty })
        }
    }
}

pub struct JobBuf {
    buf: *mut c_void,
    len: usize,
}

impl JobBuf {
    pub(crate) fn from_native(buf: *mut c_void, len: usize) -> Self {
        Self { buf, len }
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

impl Write for JobBuf {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        unsafe {
            let write_size = self.len.min(buf.len());
            std::ptr::copy_nonoverlapping(buf.as_ptr(), self.buf as *mut u8, write_size);
            Ok(write_size)
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
