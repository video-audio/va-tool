use std::fmt;

use failure::{Backtrace, Context, Fail};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    Logger,
    Config,
    URLParse(String),
    Signal,

    SourceSpawn,
    SourceInputLock(String),
    SourceStop,
    SourceJoin(String),
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

    pub(crate) fn signal<E: Fail>(err: E) -> Error {
        Error::from(err.context(ErrorKind::Signal))
    }

    pub(crate) fn source_spawn<E: Fail>(err: E) -> Error {
        Error::from(err.context(ErrorKind::SourceSpawn))
    }

    pub(crate) fn source_join<S: AsRef<str>>(reason: S) -> Error {
        Error::from(ErrorKind::SourceJoin(reason.as_ref().to_string()))
    }

    pub(crate) fn source_input_lock<S: AsRef<str>>(reason: S) -> Error {
        Error::from(ErrorKind::SourceInputLock(reason.as_ref().to_string()))
    }

    pub(crate) fn source_stop<E: Fail>(err: E) -> Error {
        Error::from(err.context(ErrorKind::SourceStop))
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
            ErrorKind::Signal => write!(f, "subscription to signals failed"),

            ErrorKind::SourceSpawn => write!(f, "source-spawn thread error"),
            ErrorKind::SourceInputLock(reason) => write!(
                f,
                "lock input inside source to read data failed (:reason {})",
                reason
            ),
            ErrorKind::SourceStop => write!(f, "source stop error"),
            ErrorKind::SourceJoin(reason) => write!(f, "source-join error (:reason {})", reason),
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
