mod test_decrypt;
mod test_lyrics_parse;
mod test_totp_gen;

use lyrix::error::parser::ParserError;

#[test]
fn parser_error_from_lyrics_parse() {
    let e = lyrix::error::parser::lyrics_parse::LyricsParseError::EmptyContent;
    let pe: ParserError = e.into();
    assert!(pe.to_string().contains("empty lyrics content"));
}

#[test]
fn parser_error_from_decrypt() {
    let e = lyrix::error::parser::decrypt::DecryptError::Base64Decode {
        detail: "bad padding".into(),
        len: 10,
    };
    let pe: ParserError = e.into();
    assert!(pe.to_string().contains("base64 decode failed"));
}

#[test]
fn parser_error_from_totp_gen() {
    let e = lyrix::error::parser::totp_gen::TotpGenError::InvalidIndex { index: 99 };
    let pe: ParserError = e.into();
    assert!(pe.to_string().contains("99"));
}

#[test]
fn parser_error_debug() {
    let e = ParserError::TotpGenerate(lyrix::error::parser::totp_gen::TotpGenError::ClockError);
    let dbg = format!("{:?}", e);
    assert!(dbg.contains("ClockError"));
}
