use std::str::Utf8Error;

use thiserror::Error;

use crate::id::{ENCODED_LENGTH, PREFIX_LENGTH, XID_ENCODED_LENGTH};

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum DecodeError {
    /// Failed to retrieve the prefix from the provided encoded PXID.
    /// This could happen if the `_` is not present.
    #[error("Failed to retrieve the prefix from the provided encoded PXID {0}")]
    MissingPrefix(String),

    /// The provided `String` has an invalid length and cannot be decoded
    /// into an instance of PXID
    #[error("String cannot be decoded into a PXID instance. {0} length is not valid. Expected length {ENCODED_LENGTH}, but received {1}")]
    InvalidLength(String, usize),

    /// The provided `String` has an invalid length and cannot be decoded
    /// into an instance of PXID
    #[error("String cannot be decoded into a PXID instance. {0} length is not valid. Expected length {PREFIX_LENGTH}, but received {1}")]
    InvalidPrefixLength(String, usize),

    /// The provided `String` contains an invalid character and cannot be decoded
    /// into an instance of PXID
    #[error("String cannot be decoded into a PXID instance. {0} length is not valid. Found invalid char {1}.")]
    InvalidChar(String, char),

    /// Invalid UTF-8 character encountered
    #[error("Invalid UTF-8 character encountered")]
    InvalidUtf8(Utf8Error),

    /// The provided `String` has an invalid length and cannot be decoded
    /// into an instance of PXID
    #[error("String cannot be decoded into a PXID instance. {0} XID length is not valid. Expected length {XID_ENCODED_LENGTH}, but received {1}")]
    InvalidXidLength(String, usize),
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum Error {
    /// An error ocurred decoding a value into an instance of XID
    #[error("Failed to decode into a XID. {0}")]
    Decode(DecodeError),

    /// Failed to retrieve Machine ID
    #[error("Failed to retrieve Machine ID. {0}")]
    MachineID(String),

    /// Prefix is too long
    #[error("Provided prefix: {0} is too long. Max allowed characters are 4.")]
    PrefixExceedsMaxLength(String),
}
