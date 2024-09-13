use hmac::digest::MacError;

#[derive(Debug)]
pub enum Error {
    FailedToOpenFile(String),
    Base64DecodeFail,
    DecryptionFail,
    MacFail(MacError),
    InvalidUTF8,
    InvalidPlaintextFormat,
    FailedToParseJson(String),
}
