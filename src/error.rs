use std::fmt;

use failure::{Backtrace, Context, Fail};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    Logger,
    Config,
    URLParse(String),
}

#[derive(Debug)]
pub struct Error {
    ctx: Context<ErrorKind>,
}

impl Error {
    pub(crate) fn config<E: Fail>(err: E) -> Error {
        Error::from(err.context(ErrorKind::Config))
    }

    pub(crate) fn url_parse<E: Fail, S: AsRef<str>>(err: E, url_raw: S) -> Error {
        Error::from(err.context(ErrorKind::URLParse(url_raw.as_ref().to_string())))
    }

    pub(crate) fn logger<E: Fail>(err: E) -> Error {
        Error::from(err.context(ErrorKind::Logger))
    }
}

impl Fail for Error {
    fn cause(&self) -> Option<&dyn Fail> {
        self.ctx.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.ctx.backtrace()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.cause() {
            // pretty-print nested errors
            Some(err) => write!(f, "{}: ({})", self.ctx, err),
            None => self.ctx.fmt(f),
        }
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            ErrorKind::Logger => write!(f, "logger error"),
            ErrorKind::Config => write!(f, "config parse error"),
            ErrorKind::URLParse(url_raw) => write!(f, "url-parse error (:url-raw {})", url_raw),
        }
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error::from(Context::new(kind))
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(ctx: Context<ErrorKind>) -> Error {
        Error { ctx }
    }
}
