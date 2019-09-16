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

    UdpUrlMissingHost,
    UdpSocketBind(String, u16),
    UdpJoinMulticastV4(String, u16, String),
    UdpJoinMulticastV6(String, u16, u32),
    UdpDomainToIpV4(String),
    UdpFifoNotInitialized,
    UdpFifoLock(String),
    UdpFifoCvarWait(String),
    UdpFifoPopEmpty,
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

    pub(crate) fn udp_url_missing_host() -> Error {
        Error::from(ErrorKind::UdpUrlMissingHost)
    }

    pub(crate) fn udp_socket_bind<E: Fail, S: AsRef<str>>(err: E, host: S, port: u16) -> Error {
        Error::from(err.context(ErrorKind::UdpSocketBind(host.as_ref().to_string(), port)))
    }

    pub(crate) fn udp_join_multicast_v4<E: Fail, S: AsRef<str>>(
        err: E,
        host: S,
        port: u16,
        group: S,
    ) -> Error {
        Error::from(err.context(ErrorKind::UdpJoinMulticastV4(
            host.as_ref().to_string(),
            port,
            group.as_ref().to_string(),
        )))
    }

    pub(crate) fn udp_join_multicast_v6<E: Fail, S: AsRef<str>>(
        err: E,
        host: S,
        port: u16,
        group: u32,
    ) -> Error {
        Error::from(err.context(ErrorKind::UdpJoinMulticastV6(
            host.as_ref().to_string(),
            port,
            group,
        )))
    }

    pub(crate) fn udp_domain_to_ipv4<E: Fail, S: AsRef<str>>(err: E, domain: S) -> Error {
        Error::from(err.context(ErrorKind::UdpDomainToIpV4(domain.as_ref().to_string())))
    }

    pub(crate) fn udp_fifo_not_initialized() -> Error {
        Error::from(ErrorKind::UdpFifoNotInitialized)
    }

    pub(crate) fn udp_fifo_lock<S: AsRef<str>>(reason: S) -> Error {
        Error::from(ErrorKind::UdpFifoLock(reason.as_ref().to_string()))
    }

    pub(crate) fn udp_fifo_cvar_wait<S: AsRef<str>>(reason: S) -> Error {
        Error::from(ErrorKind::UdpFifoCvarWait(reason.as_ref().to_string()))
    }

    pub(crate) fn udp_fifo_pop_empty() -> Error {
        Error::from(ErrorKind::UdpFifoPopEmpty)
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

            ErrorKind::UdpUrlMissingHost => write!(f, "source-udp - missing url host"),
            ErrorKind::UdpSocketBind(h, p) => {
                write!(f, "source-udp - bind error (:host {} :port {})", h, p)
            }
            ErrorKind::UdpJoinMulticastV4(h, p, g) => write!(
                f,
                "source-udp - join multicast v4 error (:host {} :port {} :group {})",
                h, p, g
            ),
            ErrorKind::UdpJoinMulticastV6(h, p, g) => write!(
                f,
                "source-udp - join multicast v6 error (:host {} :port {} :group {})",
                h, p, g
            ),
            ErrorKind::UdpDomainToIpV4(d) => write!(
                f,
                "source-udp - domain to ipv4 conversion error (:domain {})",
                d,
            ),
            ErrorKind::UdpFifoNotInitialized => {
                write!(f, "source-udp - fifo is not initializer. call open first")
            }
            ErrorKind::UdpFifoLock(reason) => {
                write!(f, "source-udp - fifo lock error (:reason {})", reason)
            }
            ErrorKind::UdpFifoCvarWait(reason) => {
                write!(f, "source-udp - condvar wait error (:reason {})", reason)
            }
            ErrorKind::UdpFifoPopEmpty => write!(f, "source-udp - no data after fifo pop"),
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
