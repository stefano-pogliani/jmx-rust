use std::fmt;

use failure::Backtrace;
use failure::Context;
use failure::Fail;
use j4rs::errors::J4RsError;


/// Error information returned by functions in case of errors.
#[derive(Debug)]
pub struct Error(Context<ErrorKind>);

impl Error {
    pub fn kind(&self) -> &ErrorKind {
        self.0.get_context()
    }
}

impl Fail for Error {
    fn cause(&self) -> Option<&Fail> {
        self.0.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.0.backtrace()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error(inner)
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error(Context::new(kind))
    }
}

impl From<J4RsError> for Error {
    fn from(e: J4RsError) -> Error {
        Error(Context::new(ErrorKind::J4RsError(e)))
    }
}


/// Exhaustive list of possible errors emitted by this crate.
#[derive(Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "could not cast java object to class '{}'", _0)]
    JavaCast(String),

    #[fail(display = "could not clone java object")]
    JavaClone,

    #[fail(display = "could not instantiate java object with class '{}'", _0)]
    JavaCreateInstance(&'static str),

    #[fail(display = "could not invoke instance method '{}.{}'", _0, _1)]
    JavaInvoke(String, &'static str),

    #[fail(display = "could not parse invocation arg '{}'", _0)]
    J4RsError(J4RsError),

    #[fail(display = "could not invoke static method '{}.{}'", _0, _1)]
    JavaInvokeStatic(&'static str, &'static str),

    #[fail(display = "could not initialise JVM instance")]
    JvmInit,

    #[fail(display = "the JMX client is not connected")]
    NotConnected,

    #[fail(display = "could not cast java object to rust '{}' type", _0)]
    RustCast(&'static str),

    #[cfg(feature = "thread-support")]
    #[fail(display = "could not decode mbean attribute value")]
    WorkerDecode,

    #[cfg(feature = "thread-support")]
    #[fail(display = "background worker did not send a response")]
    WorkerNoResponse,

    #[cfg(feature = "thread-support")]
    #[fail(display = "could not send request to background worker")]
    WorkerNoSend,

    #[cfg(feature = "thread-support")]
    #[fail(display = "could not spawn background worker thread")]
    WorkerSpawn,
}


/// Short form alias for functions returning `Error`s.
pub type Result<T> = ::std::result::Result<T, Error>;
