use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct StdError(pub String);
impl std::fmt::Display for StdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}
impl std::error::Error for StdError {}

impl StdError {
    pub fn new(msg: String) -> Self {
        Self(msg)
    }
}

#[derive(Debug)]
pub enum Error {
    Other(Box<dyn std::error::Error + Send + 'static>),
    StdError(String),
    TypeMismatch,
    Inited,
    InitFailed,
    InvalidReference,
    InvalidEnum,
    OutOfBounds,
    JsCallException,
    JsContextNotEntered,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Other(err) => Debug::fmt(err.as_ref(), f),
            _ => Debug::fmt(self, f),
        }
    }
}
pub type Result<T> = core::result::Result<T, Error>;

impl<ERR> From<ERR> for Error
where
    ERR: std::error::Error + Send + 'static,
{
    fn from(value: ERR) -> Self {
        Error::other(value)
    }
}

impl From<Error> for Box<dyn std::error::Error> {
    fn from(value: Error) -> Self {
        match value {
            Error::Other(err) => err,
            Error::StdError(msg) => Box::new(StdError::new(msg)),
            Error::TypeMismatch => Box::new(StdError::new("TypeMismatch".to_owned())),
            Error::Inited => Box::new(StdError::new("Inited".to_owned())),
            Error::InitFailed => Box::new(StdError::new("InitFailed".to_owned())),
            Error::InvalidReference => Box::new(StdError::new("InvalidReference".to_owned())),
            Error::OutOfBounds => Box::new(StdError::new("OutOfBounds".to_owned())),
            Error::JsCallException => Box::new(StdError::new("JsCallException".to_owned())),
            Error::InvalidEnum => Box::new(StdError::new("InvalidEnum".to_owned())),
            Error::JsContextNotEntered => Box::new(StdError::new("JsContextNotEntered".to_owned())),
        }
    }
}

impl Error {
    pub fn msg(str: impl Into<String>) -> Self {
        return Error::StdError(str.into());
    }

    pub fn other<ERR>(err: ERR) -> Self
    where
        ERR: std::error::Error + Send + 'static,
    {
        return Self::Other(Box::new(err));
    }

    pub fn downcast<ERR>(self) -> Result<ERR>
    where
        ERR: std::error::Error + 'static,
    {
        match self {
            Error::Other(err) => err
                .downcast::<ERR>()
                .map(|err| *err)
                .map_err(|err| Error::Other(err)),
            _ => Err(Self::TypeMismatch),
        }
    }

    pub fn downcast_ref<ERR>(&self) -> Result<&ERR>
    where
        ERR: std::error::Error + 'static,
    {
        match self {
            Error::Other(err) => err.downcast_ref::<ERR>().ok_or_else(|| Self::TypeMismatch),
            _ => Err(Self::TypeMismatch),
        }
    }

    pub fn downcast_mut<ERR>(&mut self) -> Result<&mut ERR>
    where
        ERR: std::error::Error + 'static,
    {
        match self {
            Error::Other(err) => err.downcast_mut::<ERR>().ok_or_else(|| Self::TypeMismatch),
            _ => Err(Self::TypeMismatch),
        }
    }
}
