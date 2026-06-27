use core::fmt;

/// Errors returned by the xuid library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The parsed prefix did not match the one expected by [`Xuid::parse_prefix`].
    ///
    /// [`Xuid::parse_prefix`]: crate::Xuid::parse_prefix
    BadPrefix {
        /// The prefix the caller expected.
        expected: String,
        /// The prefix actually found in the input.
        found: String,
    },
    /// The base32 body of an XUID could not be decoded.
    InvalidEncoding,
    /// The input could not be parsed as either an XUID or a standard UUID.
    InvalidUuid,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::BadPrefix { expected, found } => {
                write!(f, "xuid: bad prefix {found:?}, expected {expected:?}")
            }
            Error::InvalidEncoding => write!(f, "xuid: invalid base32 encoding"),
            Error::InvalidUuid => write!(f, "xuid: invalid uuid"),
        }
    }
}

impl std::error::Error for Error {}

impl From<uuid::Error> for Error {
    fn from(_: uuid::Error) -> Self {
        Error::InvalidUuid
    }
}

impl From<data_encoding::DecodeError> for Error {
    fn from(_: data_encoding::DecodeError) -> Self {
        Error::InvalidEncoding
    }
}
