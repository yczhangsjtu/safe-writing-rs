use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use hmac::{Hmac, Mac};
use rand::{rngs::StdRng, RngCore, SeedableRng};

use crate::{
    data_structures::{Passage, PlainText, IMAGE_SEP, WORKSPACE_SEP, SESSION_SEP},
    encode::{base64_decode, base64_decode_to_bytes, base64_encode},
    error::Error,
};
use crate::agent::{Workspace, Session};

pub fn key_derive(password: &str) -> [u8; 16] {
    let mut out = [0u8; 16];
    pbkdf2::pbkdf2_hmac::<sha2::Sha256>(password.as_bytes(), b"safe_write", 100, &mut out);
    out
}

pub fn encrypt(password: &str, data: &PlainText) -> String {
    let data = data.encode();
    let key = key_derive(password);

    let mut iv = [0u8; 16];
    StdRng::from_os_rng().fill_bytes(&mut iv);

    let encrypted = cbc::Encryptor::<aes::Aes128>::new(&key.into(), &iv.into())
        .encrypt_padded_vec_mut::<Pkcs7>(&data);
    let mut mac =
        Hmac::<sha2::Sha256>::new_from_slice(&key).expect("HMAC can take key of any size");
    mac.update(encrypted.as_slice());

    base64_encode(iv)
        + "\n"
        + &base64_encode(encrypted)
        + "\n"
        + &base64_encode(mac.finalize().into_bytes())
}

pub fn decrypt(password: &str, iv: &str, data: &str, mac: &str) -> Result<PlainText, Error> {
    let key = key_derive(password);
    let iv = base64_decode_to_bytes(iv)?;
    let data = base64_decode_to_bytes(data)?;
    let mac = base64_decode_to_bytes(mac)?;
    let mut mac_calculated =
        Hmac::<sha2::Sha256>::new_from_slice(&key).expect("HMAC can take key of any size");
    mac_calculated.update(data.as_slice());
    mac_calculated
        .verify_slice(&mac)
        .map_err(|err| Error::MacFail(err))?;

    let (plaintext, images) = cbc::Decryptor::<aes::Aes128>::new(&key.into(), iv.as_slice().into())
        .decrypt_padded_vec_mut::<Pkcs7>(&data)
        .map_err(|_| Error::DecryptionFail)
        .and_then(|s| {
            let (plaintext, images) = if let Some((i, _)) = s
                .iter()
                .enumerate()
                .filter(|(_, b)| *b == &IMAGE_SEP)
                .next()
            {
                let (plaintext, images) = s.split_at(i);
                let images = &images[1..];
                (plaintext.to_vec(), images.to_vec())
            } else {
                (s, vec![])
            };
            let plaintext = String::from_utf8(plaintext).map_err(|_| Error::InvalidUTF8)?;
            Ok((plaintext, images))
        })?;

    let images = if images.is_empty() {
        vec![]
    } else {
        if images.len() < size_of::<u32>() {
            return Err(Error::InvalidImageFormat);
        }
        let num_images = u32::from_le_bytes(
            images[0..4]
                .try_into()
                .map_err(|_| Error::InvalidImageFormat)?,
        );

        let mut splitted_images = Vec::with_capacity(num_images as usize);

        let mut images = &images[4..];
        for _ in 0..num_images {
            if images.len() < size_of::<u32>() {
                return Err(Error::InvalidImageFormat);
            }
            let image_size = u32::from_le_bytes(
                images[0..4]
                    .try_into()
                    .map_err(|_| Error::InvalidImageFormat)?,
            );
            images = &images[4..];
            if images.len() < image_size as usize {
                return Err(Error::InvalidImageFormat);
            }
            splitted_images.push(images[0..image_size as usize].to_vec());
            images = &images[image_size as usize..];
        }
        splitted_images
    };

    // Parse workspace and session from plaintext
    let (passages_text, workspace, session) = parse_workspace_session(&plaintext)?;

    if passages_text.is_empty() {
        return Ok(PlainText::new_with_workspace_session(0, vec![], images, workspace, session));
    };

    let plaintext_encodings: Vec<_> = passages_text.split("|").collect();

    let passages = plaintext_encodings
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let contents: Vec<_> = s.split("-").collect();
            if contents.len() < 2 {
                return Err(Error::InvalidPlaintextFormat);
            }
            let title = contents[0];
            let content = contents[1];
            Ok(Passage::new(
                i,
                base64_decode(title)?,
                base64_decode(content)?,
            ))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(PlainText::new_with_workspace_session(passages.len(), passages, images, workspace, session))
}

/// Parse workspace and session from the plaintext string
/// Returns (passages_text, workspace, session)
fn parse_workspace_session(plaintext: &str) -> Result<(String, Workspace, Session), Error> {
    // Find :FontSize=24 to determine the end
    let font_size_idx = plaintext.find(":FontSize=24");

    // Check if this is a new format file (has :WORKSPACE and :SESSION)
    let workspace_idx = plaintext.find(WORKSPACE_SEP);
    let session_idx = plaintext.find(SESSION_SEP);

    // Determine the end of passages section
    let passages_end = match (workspace_idx, session_idx) {
        (Some(w), _) => w,  // Workspace marker starts after passages
        (_, Some(s)) => s,  // Session marker (legacy: no workspace but has session)
        _ => {
            // Old format - no workspace or session
            match font_size_idx {
                Some(idx) => idx,
                None => plaintext.len(),
            }
        }
    };

    let passages_text = plaintext[..passages_end].to_string();

    // Parse workspace if present
    let workspace = if let Some(w_idx) = workspace_idx {
        let workspace_start = w_idx + WORKSPACE_SEP.len();
        let workspace_end = session_idx.unwrap_or_else(|| {
            font_size_idx.unwrap_or(plaintext.len())
        });
        if workspace_start < workspace_end {
            let workspace_b64 = &plaintext[workspace_start..workspace_end];
            // Try base64 decode first (new format), fall back to raw JSON (old format)
            let workspace_json = base64_decode(workspace_b64)
                .or_else(|_| Ok(workspace_b64.to_string()))?;
            Workspace::from_json(&workspace_json).unwrap_or_default()
        } else {
            Workspace::default()
        }
    } else {
        Workspace::default()
    };

    // Parse session if present
    let session = if let Some(s_idx) = session_idx {
        let session_start = s_idx + SESSION_SEP.len();
        let session_end = font_size_idx.unwrap_or(plaintext.len());
        if session_start < session_end {
            let session_b64 = &plaintext[session_start..session_end];
            // Try base64 decode first (new format), fall back to raw JSON (old format)
            let session_json = base64_decode(session_b64)
                .or_else(|_| Ok(session_b64.to_string()))?;
            Session::from_json(&session_json).unwrap_or_default()
        } else {
            Session::default()
        }
    } else {
        Session::default()
    };

    Ok((passages_text, workspace, session))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_derive_consistency() {
        let key1 = key_derive("test_password");
        let key2 = key_derive("test_password");
        assert_eq!(key1, key2);

        let key3 = key_derive("different_password");
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let plaintext = PlainText::from_passages(vec![
            Passage::new(0, "Test Title".to_string(), "Test Content".to_string()),
        ]);
        let password = "my_password";
        let encrypted = encrypt(password, &plaintext);
        let decrypted = decrypt(password, &encrypted.split("\n").collect::<Vec<_>>()[0], &encrypted.split("\n").collect::<Vec<_>>()[1], &encrypted.split("\n").collect::<Vec<_>>()[2]).unwrap();
        assert_eq!(plaintext.num_passages(), decrypted.num_passages());
        assert_eq!(plaintext.title_of_passage(0), decrypted.title_of_passage(0));
        assert_eq!(plaintext.content_of_passage(0), decrypted.content_of_passage(0));
    }

    #[test]
    fn test_decrypt_wrong_password() {
        let plaintext = PlainText::from_passages(vec![
            Passage::new(0, "Title".to_string(), "Content".to_string()),
        ]);
        let encrypted = encrypt("correct_password", &plaintext);
        let parts: Vec<_> = encrypted.split("\n").collect();
        let result = decrypt("wrong_password", parts[0], parts[1], parts[2]);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_corrupted_mac() {
        let plaintext = PlainText::from_passages(vec![
            Passage::new(0, "Title".to_string(), "Content".to_string()),
        ]);
        let password = "password";
        let encrypted = encrypt(password, &plaintext);
        let parts: Vec<_> = encrypted.split("\n").collect();
        // Completely corrupt the MAC by replacing all characters
        let corrupted_mac = "AAAAAAAA";  // Invalid base64 or wrong MAC
        let result = decrypt(password, parts[0], parts[1], corrupted_mac);
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_decrypt_empty() {
        let plaintext = PlainText::empty();
        let password = "password";
        let encrypted = encrypt(password, &plaintext);
        let parts: Vec<_> = encrypted.split("\n").collect();
        let decrypted = decrypt(password, parts[0], parts[1], parts[2]).unwrap();
        assert!(decrypted.is_empty());
    }
}