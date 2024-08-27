use crate::{
    error::{Error, Result},
    utils::{from_cstr_ptr, to_cstr_ptr},
};
use std::{
    any::TypeId,
    sync::{Arc, RwLockReadGuard, RwLockWriteGuard},
};
use wke_sys::{
    wkeRect, wkeUtilBase64Decode, wkeUtilBase64Encode, wkeUtilDecodeURLEscape,
    wkeUtilEncodeURLEscape,
};

pub fn base64_encode(str: &str) -> Result<String> {
    unsafe {
        let encoded = wkeUtilBase64Encode.unwrap()(to_cstr_ptr(str));
        from_cstr_ptr(encoded)
    }
}

pub fn base64_decode(str: &str) -> Result<String> {
    unsafe {
        let encoded = wkeUtilBase64Decode.unwrap()(to_cstr_ptr(str));
        from_cstr_ptr(encoded)
    }
}

pub fn url_encode(str: &str) -> Result<String> {
    unsafe {
        let encoded = wkeUtilEncodeURLEscape.unwrap()(to_cstr_ptr(str));
        from_cstr_ptr(encoded)
    }
}

pub fn url_decode(str: &str) -> Result<String> {
    unsafe {
        let encoded = wkeUtilDecodeURLEscape.unwrap()(to_cstr_ptr(str));
        from_cstr_ptr(encoded)
    }
}

///位置
#[derive(Clone, Copy)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

///尺寸
#[derive(Clone, Copy)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}

///举行区域
#[derive(Default, Clone, Copy)]
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
        let val = self.value.read().map_err(Error::other)?;
        Ok(val)
    }

    pub fn write(&'static self) -> Result<RwLockWriteGuard<T>> {
        let val = self.value.write().map_err(Error::other)?;
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
