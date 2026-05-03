use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use hmac::{Hmac, Mac};
use rand::{rngs::StdRng, RngCore, SeedableRng};

use crate::{
    data_structures::{Passage, PlainText, IMAGE_SEP},
    encode::{base64_decode, base64_decode_to_bytes, base64_encode},
    error::Error,
};

pub fn key_derive(password: &str) -> [u8; 16] {
    let mut out = [0u8; 16]; // We will use 128 bits key
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
        // The images are encoded as follows:
        // 1. Number of images, encoded as u32
        // 2. For each image, the image data is encoded as:
        //    2.1 Size of the image, encoded as u32
        //    2.2 The image data (in png format)
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

    let plaintexts: Vec<_> = plaintext.split(":").collect();
    if plaintexts.len() < 2 {
        return Err(Error::InvalidPlaintextFormat);
    }
    let plaintext_encodings = plaintexts[0];
    // let font_size = plaintexts[1];

    if plaintext_encodings.is_empty() {
        return Ok(PlainText::empty());
    };

    let plaintext_encodings: Vec<_> = plaintext_encodings.split("|").collect();

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

    Ok(PlainText::new(passages.len(), passages, images))
}
