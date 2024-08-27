#[derive(Debug)]
pub enum Error {
    Inited,
    InitFailed,
    OutOfBounds,
    TypeMismatch(String),
    JsCallException,
    Other(Box<dyn std::error::Error>),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl std::error::Error for Error {}
pub type Result<T> = core::result::Result<T, Error>;

impl Error {
    pub fn other<ERR>(err: ERR) -> Self
    where
        ERR: std::error::Error + 'static,
    {
        return Self::Other(Box::new(err));
    }
}
