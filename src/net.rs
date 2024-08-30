use std::os::raw::c_void;
use wke_sys::wkeNetJob;

pub struct Job {
    job: wkeNetJob,
}

pub struct JobBuf {
    buf: *mut c_void,
    len: usize,
}

impl Job {
    pub fn set_mime_type() {}
}
