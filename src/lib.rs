pub mod common;
pub mod delegate;
pub mod error;
pub mod javascript;
pub mod net;
pub mod webframe;
pub mod webview;
pub use wke_command::command;
pub use wke_main::main;

mod utils;

#[cfg(test)]
mod tests;

use crate::{
    error::{Error, Result},
    utils::{from_bool_int, from_cstr_ptr},
};
use wke_sys::*;

///代理设置
#[derive(Debug, Clone)]
pub struct ProxyOptions {
    ///代理域名或ip地址
    hostname: String,
    ///代理端口
    port: u16,
    ///用户名
    username: Option<String>,
    ///密码
    password: Option<String>,
}

fn encode_to_buf(str: &str, buf: &mut [i8]) -> Result<()> {
    let mut index = 0;
    let mut ch_buf = [0u8; 8];

    for ch in str.chars() {
        let ch_size = ch.encode_utf8(&mut ch_buf).len();
        for i in 0..ch_size {
            if index >= buf.len() - 1 {
                return Err(Error::OutOfBounds);
            }

            buf[index] = ch_buf[i] as i8;
            index = index + 1;
        }
    }

    return Ok(());
}

impl ProxyOptions {
    fn into_native(&self, proxy_type: wkeProxyType) -> Result<wkeProxy> {
        let mut hostname = [0i8; 100];
        let mut username = [0i8; 50];
        let mut password = [0i8; 50];

        encode_to_buf(&self.hostname, &mut hostname)?;
        if let Some(str) = &self.username {
            encode_to_buf(&str, &mut username)?;
        }

        if let Some(str) = &self.password {
            encode_to_buf(&str, &mut password)?;
        }

        Ok(wkeProxy {
            type_: proxy_type,
            hostname,
            port: self.port,
            username,
            password,
        })
    }
}

///代理
#[derive(Debug, Clone)]
pub enum Proxy {
    ///禁用代理
    None,
    ///HTTP代理
    Http(ProxyOptions),
    ///Sock4代理
    Sock4(ProxyOptions),
    ///Sock4A代理
    Sock4A(ProxyOptions),
    ///Sock5代理
    Sock5(ProxyOptions),
    ///Sock5Hostname代理
    Sock5Hostname(ProxyOptions),
}
impl PartialEq for Proxy {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}
impl Eq for Proxy {}

impl std::default::Default for Proxy {
    fn default() -> Self {
        Proxy::None
    }
}

impl Proxy {
    fn into_native(&self) -> Result<wkeProxy> {
        match self {
            Proxy::None => Ok(wkeProxy {
                type_: _wkeProxyType_WKE_PROXY_NONE,
                hostname: [0; 100],
                port: 0,
                username: [0; 50],
                password: [0; 50],
            }),
            Proxy::Http(option) => option.into_native(_wkeProxyType_WKE_PROXY_HTTP),
            Proxy::Sock4(option) => option.into_native(_wkeProxyType_WKE_PROXY_SOCKS4),
            Proxy::Sock4A(option) => option.into_native(_wkeProxyType_WKE_PROXY_SOCKS4A),
            Proxy::Sock5(option) => option.into_native(_wkeProxyType_WKE_PROXY_SOCKS5),
            Proxy::Sock5Hostname(option) => {
                option.into_native(_wkeProxyType_WKE_PROXY_SOCKS5HOSTNAME)
            }
        }
    }
}

///初始化设置
#[derive(Debug, Default, Clone)]
pub struct Settings {
    ///代理配置
    pub proxy: Proxy,
    pub enable_nodejs: bool,
    pub enable_disable_h5video: bool,
    pub enable_disable_pdfview: bool,
    pub enable_disable_cc: bool,
    pub enable_enable_eglgles2: bool,
    pub enable_enable_swiftshaer: bool,
}

///初始化miniblink
pub fn init(dll: &str) -> Result<()> {
    unsafe {
        if wkeIsInitialize.is_some() && from_bool_int(wkeIsInitialize.unwrap()()) {
            return Err(Error::Inited);
        }

        let mut dll_u16 = dll.encode_utf16().collect::<Vec<u16>>();
        dll_u16.push(0);
        wkeSetWkeDllPath((&dll_u16).as_ptr());
        if !from_bool_int(wkeInit()) {
            return Err(Error::InitFailed);
        }

        return Ok(());
    }
}

pub fn shutdown() {
    unsafe {
        wkeShutdown.unwrap()();
    }
}

pub fn set_proxy(proxy: Proxy) -> Result<()> {
    unsafe {
        let proxy_native = proxy.into_native()?;
        wkeSetProxy.unwrap()(&proxy_native);
        Ok(())
    }
}

pub fn version() -> u32 {
    unsafe { wkeVersion.unwrap()() }
}

pub fn version_str() -> Result<String> {
    unsafe { from_cstr_ptr(wkeVersionString.unwrap()()) }
}

pub fn run() {
    unsafe {
        win32RunLoop();
    }
}

pub enum RunOnceFlag {
    Idle,
    RunOnce,
    Exit,
}

pub fn run_once() -> RunOnceFlag {
    unsafe {
        let result = win32RunLoopOnce();
        match result {
            1 => RunOnceFlag::RunOnce,
            0 => RunOnceFlag::Idle,
            _ => RunOnceFlag::Exit,
        }
    }
}

pub fn exit() {
    unsafe {
        win32ExitLoop();
    }
}

pub fn enable_high_dpi_support() {
    unsafe { wkeEnableHighDPISupport.unwrap()() }
}

#[cfg(test)]
pub fn report() -> String {
    unsafe {
        let report = from_cstr_ptr(wkeReport()).unwrap();
        wkeReportFree();
        report
    }
}
