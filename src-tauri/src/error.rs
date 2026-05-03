use serde::{Deserialize, Serialize};

#[derive(Debug)]
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
