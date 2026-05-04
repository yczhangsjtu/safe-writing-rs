use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum Error {
    FailedToOpenFile(String),
    Base64DecodeFail,
    DecryptionFail,
    MacFail(hmac::digest::MacError),
    InvalidUTF8,
    InvalidPlaintextFormat,
    FailedToParseJson(String),
    InvalidImageFormat,
}

impl Serialize for Error {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&format!("{:?}", self))
    }
}

impl<'de> Deserialize<'de> for Error {
    fn deserialize<D: serde::Deserializer<'de>>(_deserializer: D) -> Result<Self, D::Error> {
        Ok(Error::DecryptionFail)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::FailedToOpenFile(s) => write!(f, "Failed to open file: {}", s),
            Error::Base64DecodeFail => write!(f, "Base64 decode failed"),
            Error::DecryptionFail => write!(f, "Decryption failed"),
            Error::MacFail(_) => write!(f, "MAC verification failed"),
            Error::InvalidUTF8 => write!(f, "Invalid UTF-8"),
            Error::InvalidPlaintextFormat => write!(f, "Invalid plaintext format"),
            Error::FailedToParseJson(s) => write!(f, "Failed to parse JSON: {}", s),
            Error::InvalidImageFormat => write!(f, "Invalid image format"),
        }
    }
}

impl std::error::Error for Error {}