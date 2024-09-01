pub mod handle;
pub mod lazy;

use crate::{
    error::{Error, Result},
    utils::{from_cstr_ptr, to_cstr_ptr},
};
use std::{
    any::TypeId,
    cell::RefCell,
    rc::Rc,
    sync::{Arc, RwLockReadGuard, RwLockWriteGuard},
    task::Waker,
};
use wke_sys::{
    wkeRect, wkeUtilBase64Decode, wkeUtilBase64Encode, wkeUtilDecodeURLEscape,
    wkeUtilEncodeURLEscape,
};

pub fn base64_encode(str: &str) -> Result<String> {
    unsafe {
        let encoded = wkeUtilBase64Encode.unwrap()(to_cstr_ptr(str)?.to_utf8());
        from_cstr_ptr(encoded)
    }
}

pub fn base64_decode(str: &str) -> Result<String> {
    unsafe {
        let encoded = wkeUtilBase64Decode.unwrap()(to_cstr_ptr(str)?.to_utf8());
        from_cstr_ptr(encoded)
    }
}

pub fn url_encode(str: &str) -> Result<String> {
    unsafe {
        let encoded = wkeUtilEncodeURLEscape.unwrap()(to_cstr_ptr(str)?.to_utf8());
        from_cstr_ptr(encoded)
    }
}

pub fn url_decode(str: &str) -> Result<String> {
    unsafe {
        let encoded = wkeUtilDecodeURLEscape.unwrap()(to_cstr_ptr(str)?.to_utf8());
        from_cstr_ptr(encoded)
    }
}

///位置
#[derive(Debug, Default, Clone, Copy)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

///尺寸
#[derive(Debug, Default, Clone, Copy)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}

///举行区域
#[derive(Debug, Default, Clone, Copy)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub(crate) fn from_native(rc: &wkeRect) -> Self {
        return Rect {
            x: rc.x,
            y: rc.y,
            width: rc.w,
            height: rc.h,
        };
    }

    ///获取位置
    pub fn pos(&self) -> Point {
        Point {
            x: self.x,
            y: self.y,
        }
    }

    ///获取尺寸
    pub fn size(&self) -> Size {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    ///中点位置
    pub fn center(&self) -> Point {
        Point {
            x: self.x + self.width / 2,
            y: self.y + self.height / 2,
        }
    }
}

pub struct UserValue<T: 'static> {
    typeid: TypeId,
    value: std::sync::RwLock<T>,
}

impl<T: 'static> UserValue<T> {
    ///创建一个新的用户变量
    pub(crate) fn new(value: T) -> Arc<Self> {
        Arc::new(Self {
            typeid: TypeId::of::<T>(),
            value: std::sync::RwLock::new(value),
        })
    }

    pub fn read(&'static self) -> Result<RwLockReadGuard<T>> {
        let val = self
            .value
            .read()
            .map_err(|_| Error::msg("uesr value read lock failed"))?;
        Ok(val)
    }

    pub fn write(&'static self) -> Result<RwLockWriteGuard<T>> {
        let val = self
            .value
            .write()
            .map_err(|_| Error::msg("uesr value write lock failed"))?;
        Ok(val)
    }

    ///转为指针
    pub(crate) fn into_raw(arc: Arc<Self>) -> *const Self {
        Arc::into_raw(arc)
    }

    ///从指针转为UserValue
    pub(crate) fn from_raw(ptr: *const UserValue<T>) -> Result<Arc<Self>> {
        unsafe {
            if ptr.is_null() {
                return Err(Error::TypeMismatch("ptr is null".to_owned()));
            }

            let arc = Arc::from_raw(ptr);
            let typeid: TypeId = TypeId::of::<T>();
            if arc.typeid != typeid {
                let err = Error::TypeMismatch(format!(
                    "type {:?} is mismatch to {:?} of stored",
                    arc.typeid, typeid
                ));
                return Err(err);
            }

            Ok(arc)
        }
    }
}

/***********   *************/
struct InvokeFutureInner<T: Unpin> {
    pub(crate) value: Option<T>,
    pub(crate) waker: Option<Waker>,
}

impl<T: Unpin> InvokeFutureInner<T> {
    pub(crate) fn ready(&mut self, value: T) {
        self.value.replace(value);
        if let Some(waker) = self.waker.take() {
            waker.wake();
        }
    }
}

#[derive(Clone)]
pub struct InvokeFuture<T: Unpin>(Rc<RefCell<InvokeFutureInner<T>>>);

impl<T: Unpin> std::default::Default for InvokeFuture<T> {
    fn default() -> Self {
        Self(Rc::new(RefCell::new(InvokeFutureInner {
            value: None,
            waker: None,
        })))
    }
}

impl<T: Unpin> std::future::Future for InvokeFuture<T> {
    type Output = T;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut self_mut = self.get_mut().0.borrow_mut();

        if let Some(value) = self_mut.value.take() {
            return std::task::Poll::Ready(value);
        } else {
            self_mut.waker.replace(cx.waker().clone());
        }
        return std::task::Poll::Pending;
    }
}

impl<T: Unpin> InvokeFuture<T> {
    pub(crate) unsafe fn from_raw<PTR>(ptr: *const PTR) -> Self {
        let inner = Rc::from_raw(ptr as *const RefCell<InvokeFutureInner<T>>);
        Self(inner)
    }

    pub(crate) fn into_raw<PTR>(&self) -> *mut PTR {
        Rc::into_raw(self.0.clone()) as *mut PTR
    }

    pub(crate) fn ready(&self, value: T) {
        self.0.borrow_mut().ready(value);
    }
}
