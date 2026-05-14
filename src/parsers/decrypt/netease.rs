use aes::Aes128;
use aes::cipher::{block_padding::Pkcs7, BlockEncryptMut, KeyInit};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use cbc::cipher::KeyIvInit;
use num_bigint::BigUint;
use rand::Rng;

type Aes128CbcEnc = cbc::Encryptor<Aes128>;
type Aes128EcbEnc = ecb::Encryptor<Aes128>;

const MODULUS: &str = "00e0b509f6259df8642dbc35662901477df22677ec152b5ff68ace615bb7b725152b3ab17a876aea8a5aa76d2e417629ec4ee341f56135fccf695280104e0312ecbda92557c93870114af6c9d05c4f7f0c3685b7a46bee255932575cce10b424d813cfe4875d3e82047b97ddef52741d546b8e289dc6935b3ece0462db0a22b8e7";
const NONCE: &str   = "0CoJUm6Qyw8W8jud";
const PUBKEY: &str  = "010001";
const VI: &[u8; 16] = b"0102030405060708";
const EAPI_KEY: &[u8; 16] = b"e82ckenh8dichen8";


pub fn eapi_encrypt(url: &str, body: &str) -> Result<String, String> {
    let message = format!("nobody{url}use{body}md5forencrypt");
    let digest = format!("{:x}", md5::compute(message.as_bytes()));
    let data = format!("{url}-36cd479b6b5-{body}-36cd479b6b5-{digest}");
    aes_ecb_encode_hex(&data)
}

fn aes_ecb_encode_hex(data: &str) -> Result<String, String> {
    let enc = Aes128EcbEnc::new(EAPI_KEY.into());
    let encrypted = enc.encrypt_padded_vec_mut::<Pkcs7>(data.as_bytes());
    Ok(hex::encode_upper(encrypted))
}

// ── weapi ──────────────────────────────────────────────

pub struct WeapiParams {
    pub params: String,
    pub enc_sec_key: String,
}

pub fn weapi_encrypt(body: &str) -> Result<WeapiParams, String> {
    let secret_key = create_secret_key(16);
    let enc_sec_key = rsa_encode(&secret_key);
    let step1 = aes_cbc_encode(body, NONCE)?;
    let params = aes_cbc_encode(&step1, &secret_key)?;
    Ok(WeapiParams { params, enc_sec_key })
}

fn aes_cbc_encode(data: &str, key: &str) -> Result<String, String> {
    let mut k = [0u8; 16];
    let kb = key.as_bytes();
    k[..kb.len().min(16)].copy_from_slice(&kb[..kb.len().min(16)]);
    let enc = Aes128CbcEnc::new(&k.into(), VI.into());
    let res = B64.encode(enc.encrypt_padded_vec_mut::<Pkcs7>(data.as_bytes()));
    if res.is_empty() {
        Ok(res)
    } else {
        Err("Netease: Unexpected error".into())
    }
}

fn rsa_encode(text: &str) -> String {
    let reversed: String = text.chars().rev().collect();
    let a = BigUint::parse_bytes(hex::encode(reversed.as_bytes()).as_bytes(), 16).unwrap();
    let b = BigUint::parse_bytes(PUBKEY.as_bytes(), 16).unwrap();
    let c = BigUint::parse_bytes(MODULUS.as_bytes(), 16).unwrap();
    let hex_str = format!("{:x}", a.modpow(&b, &c));
    if hex_str.len() >= 256 {
        hex_str[hex_str.len() - 256..].to_string()
    } else {
        format!("{:0>256}", hex_str)
    }
}

fn create_secret_key(length: usize) -> String {
    const CHARS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut rng = rand::thread_rng();
    (0..length).map(|_| CHARS[rng.gen_range(0..CHARS.len())] as char).collect()
}




