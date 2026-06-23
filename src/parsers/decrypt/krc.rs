use crate::error::parser::decrypt::DecryptError;
use crate::error::LyrixResult;

pub fn krc_decrypt(encoded: &str) -> LyrixResult<String> {
    use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
    use flate2::read::{DeflateDecoder, ZlibDecoder};
    use std::io::Read;

    const KEY: &[u8] = &[
        0x40, 0x47, 0x61, 0x77, 0x5e, 0x32, 0x74, 0x47, 0x51, 0x36, 0x31, 0x2d, 0xce, 0xd2, 0x6e,
        0x69,
    ];

    let clean: String = encoded.chars().filter(|c| !c.is_whitespace()).collect();
    let decoded = B64.decode(&clean).map_err(|e| DecryptError::Base64Decode {
        detail: e.to_string(),
        len: clean.len(),
    })?;
    if decoded.len() <= 4 {
        return Err(DecryptError::Deflate {
            detail: format!("decoded data too short: {} bytes", decoded.len()),
        }
        .into());
    }
    let mut data = decoded[4..].to_vec();
    for (i, byte) in data.iter_mut().enumerate() {
        *byte ^= KEY[i % KEY.len()];
    }
    let head4: Vec<String> = data[..4.min(data.len())]
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect();
    let inflated = {
        let mut out = Vec::new();
        if ZlibDecoder::new(&data[..]).read_to_end(&mut out).is_ok() && !out.is_empty() {
            out
        } else {
            let mut out2 = Vec::new();
            DeflateDecoder::new(&data[..])
                .read_to_end(&mut out2)
                .map_err(|e| DecryptError::Deflate {
                    detail: format!("inflate failed (xor_head=[{}]): {}", head4.join(","), e),
                })?;
            if out2.is_empty() {
                return Err(DecryptError::Deflate {
                    detail: format!(
                        "inflate produced empty output (xor_head=[{}])",
                        head4.join(",")
                    ),
                }
                .into());
            }
            out2
        }
    };
    let skip = if inflated.starts_with(&[0xEF, 0xBB, 0xBF]) {
        3
    } else {
        1
    };
    if inflated.len() <= skip {
        return Err(DecryptError::Deflate {
            detail: format!(
                "inflated data too short after BOM skip({}): {} bytes",
                skip,
                inflated.len()
            ),
        }
        .into());
    }
    String::from_utf8(inflated[skip..].to_vec()).map_err(|e| {
        DecryptError::Utf8Decode {
            detail: e.to_string(),
        }
        .into()
    })
}
