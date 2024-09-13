use crate::error::Error;
use base64::{engine::general_purpose, Engine as _};

pub fn base64_decode_to_bytes(data: &str) -> Result<Vec<u8>, Error> {
    general_purpose::STANDARD
        .decode(data)
        .map_err(|_| Error::Base64DecodeFail)
}

pub fn base64_encode<T: AsRef<[u8]>>(data: T) -> String {
    general_purpose::STANDARD.encode(data)
}

pub fn base64_decode(data: &str) -> Result<String, Error> {
    String::from_utf8(
        general_purpose::STANDARD
            .decode(data)
            .map_err(|_| Error::Base64DecodeFail)?,
    )
    .map_err(|_| Error::InvalidUTF8)
}
