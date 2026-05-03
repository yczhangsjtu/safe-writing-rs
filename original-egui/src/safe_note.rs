use std::path::PathBuf;

use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyIvInit};
use serde::{Deserialize, Serialize};
use sha2::Digest;

use crate::{
    data_structures::{Passage, PlainText},
    encode::base64_decode_to_bytes,
    error::Error,
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SafeNoteFile {
    records: Vec<SafeNoteRecord>,
}

impl SafeNoteFile {
    pub fn into_passages(&self) -> Vec<Passage> {
        self.records.iter().map(|p| p.into_passage()).collect()
    }

    pub fn into_plaintext(&self) -> PlainText {
        PlainText::from_passages(self.into_passages())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SafeNoteRecord {
    title: String,
    description: String,
}

impl SafeNoteRecord {
    pub fn into_passage(&self) -> Passage {
        Passage::new(0, self.title.clone(), self.description.clone())
    }
}

pub fn load_safe_note_file(password: &str, file_path: &PathBuf) -> Result<SafeNoteFile, Error> {
    let contents = std::fs::read_to_string(file_path)
        .map_err(|err| Error::FailedToOpenFile(format!("{:?}", err)))?;
    let mut safenote: SafeNoteFile = serde_json::from_str(&contents)
        .map_err(|err| Error::FailedToParseJson(format!("{:?}", err)))?;
    for record in safenote.records.iter_mut() {
        record.title = decrypt_safe_notes_ciphertext(password, &record.title)?;
        record.description = decrypt_safe_notes_ciphertext(password, &record.description)?;
    }
    Ok(safenote)
}

pub fn decrypt_safe_notes_ciphertext(password: &str, ciphertext: &str) -> Result<String, Error> {
    let data = base64_decode_to_bytes(ciphertext)?;
    let salt = data[8..16].to_vec();
    let data = data[16..].to_vec();
    let password = password.as_bytes();
    let mut concatenated_hashes = Vec::<u8>::new();
    let mut current_hash = Vec::<u8>::new();
    let mut pre_hash: Vec<u8>;

    for _ in 0..32 {
        if current_hash.len() > 0 {
            pre_hash = current_hash.clone();
            pre_hash.extend_from_slice(password);
            pre_hash.extend_from_slice(&salt);
        } else {
            pre_hash = password.to_vec();
            pre_hash.extend_from_slice(&salt);
        }
        let mut hasher = sha2::Sha256::new();
        hasher.update(&pre_hash);
        current_hash = hasher.finalize().to_vec();
        concatenated_hashes.extend_from_slice(&current_hash);
        if concatenated_hashes.len() > 48 {
            break;
        }
    }
    let key = concatenated_hashes[0..32].to_vec();
    let iv = concatenated_hashes[32..48].to_vec();

    cbc::Decryptor::<aes::Aes256>::new(key.as_slice().into(), iv.as_slice().into())
        .decrypt_padded_vec_mut::<Pkcs7>(&data)
        .map_err(|_| Error::DecryptionFail)
        .and_then(|s| String::from_utf8(s).map_err(|_| Error::InvalidUTF8))
}
