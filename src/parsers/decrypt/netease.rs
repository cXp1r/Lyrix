use crate::error::LyrixResult;
use aes::cipher::{block_padding::Pkcs7, BlockEncryptMut, KeyInit};
use aes::Aes128;

type Aes128EcbEnc = ecb::Encryptor<Aes128>;

const EAPI_KEY: &[u8; 16] = b"e82ckenh8dichen8";

pub fn eapi_encrypt(url: &str, body: &str) -> LyrixResult<String> {
    let message = format!("nobody{url}use{body}md5forencrypt");
    let digest = format!("{:x}", md5::compute(message.as_bytes()));
    let data = format!("{url}-36cd479b6b5-{body}-36cd479b6b5-{digest}");
    aes_ecb_encode_hex(&data)
}

fn aes_ecb_encode_hex(data: &str) -> LyrixResult<String> {
    let enc = Aes128EcbEnc::new(EAPI_KEY.into());
    let encrypted = enc.encrypt_padded_vec_mut::<Pkcs7>(data.as_bytes());
    Ok(hex::encode_upper(encrypted))
}
