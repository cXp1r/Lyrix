use lyrix::error::parser::parse::ParseError;
use lyrix::error::LyrixError;
use lyrix::parsers::lrc::LrcParser;
use lyrix::parsers::netease::{NeteaseLrcParser, NeteaseParser};
use lyrix::parsers::IParsers;

// ── NeteaseLrcParser ──────────────────────────────────────────

#[test]
fn netease_lrc_v3_format_colon_separator() {
    // v3: 第二个分隔符是 ':', centis*10
    let parser = NeteaseLrcParser { version: 3 };
    let ms = parser.parse_lrc_time("01:02:03").unwrap();
    assert_eq!(ms, 1 * 60000 + 2 * 1000 + 3 * 10); // 62030
}

#[test]
fn netease_lrc_v4_format_dot_separator() {
    // v4: 第二个分隔符是 '.', centis 原样
    let parser = NeteaseLrcParser { version: 4 };
    let ms = parser.parse_lrc_time("01:02.03").unwrap();
    assert_eq!(ms, 1 * 60000 + 2 * 1000 + 3); // 62003
}

#[test]
fn netease_lrc_zero_time() {
    let parser = NeteaseLrcParser { version: 4 };
    assert_eq!(parser.parse_lrc_time("00:00.00").unwrap(), 0);
}

#[test]
fn netease_lrc_max_values() {
    let parser = NeteaseLrcParser { version: 4 };
    let ms = parser.parse_lrc_time("59:59.99").unwrap();
    assert_eq!(ms, 59 * 60000 + 59 * 1000 + 99);
}

#[test]
fn netease_lrc_no_colon() {
    let parser = NeteaseLrcParser { version: 3 };
    let err = parser.parse_lrc_time("not-a-time").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("':'"));
}

#[test]
fn netease_lrc_no_second_separator_v3() {
    let parser = NeteaseLrcParser { version: 3 };
    // "01:0230" — 在 col+1 之后没有 ':' 或 '.'
    let err = parser.parse_lrc_time("01:0230").unwrap_err();
    assert!(matches!(
        err,
        LyrixError::Parser(lyrix::error::ParserError::Parse(
            ParseError::InvalidLrcFormat { .. }
        ))
    ));
}

#[test]
fn netease_lrc_no_second_separator_v4() {
    let parser = NeteaseLrcParser { version: 4 };
    let err = parser.parse_lrc_time("01:0230").unwrap_err();
    assert!(matches!(
        err,
        LyrixError::Parser(lyrix::error::ParserError::Parse(
            ParseError::InvalidLrcFormat { .. }
        ))
    ));
}

#[test]
fn netease_lrc_non_numeric_minutes() {
    let parser = NeteaseLrcParser { version: 3 };
    let err = parser.parse_lrc_time("xx:02.03").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("minutes"));
}

#[test]
fn netease_lrc_non_numeric_seconds() {
    let parser = NeteaseLrcParser { version: 3 };
    let err = parser.parse_lrc_time("01:xx.03").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("seconds"));
}

#[test]
fn netease_lrc_non_numeric_centis() {
    let parser = NeteaseLrcParser { version: 3 };
    let err = parser.parse_lrc_time("01:02.xx").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("centis"));
}

#[test]
fn netease_lrc_empty_tag() {
    let parser = NeteaseLrcParser { version: 4 };
    assert!(parser.parse_lrc_time("").is_err());
}

// ── NeteaseParser::parse_syllables ─────────────────────────────
// Netease 格式: (s,d)text — 文字在 ')' 之后
// s 参数是行 start_time, s1 是括号里的起始时间

#[test]
fn netease_syllable_single() {
    let parser = NeteaseParser;
    // 行 start=1000, 音节 (1500,300)Hello — offset = 1500-1000=500
    let result = parser.parse_syllables(1000, "(1500,300)Hello").unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].start_time, 500);
    assert_eq!(result[0].duration, 300);
    assert_eq!(result[0].text, "Hello");
}

#[test]
fn netease_syllable_multiple() {
    let parser = NeteaseParser;
    let result = parser
        .parse_syllables(1000, "(1000,200)A(1200,300)B")
        .unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].text, "A");
    assert_eq!(result[0].start_time, 0); // 1000-1000
    assert_eq!(result[0].duration, 200);
    assert_eq!(result[1].text, "B");
    assert_eq!(result[1].start_time, 200); // 1200-1000
    assert_eq!(result[1].duration, 300);
}

#[test]
fn netease_syllable_triplet_format() {
    let parser = NeteaseParser;
    // (s,d,x) — 第三个字段被忽略
    let result = parser.parse_syllables(0, "(1000,500,0)Text").unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].duration, 500);
    assert_eq!(result[0].text, "Text");
}

#[test]
fn netease_syllable_empty() {
    let parser = NeteaseParser;
    let result = parser.parse_syllables(0, "").unwrap();
    assert!(result.is_empty());
}

#[test]
fn netease_syllable_no_paren() {
    let parser = NeteaseParser;
    let result = parser
        .parse_syllables(0, "plain text without times")
        .unwrap();
    assert!(result.is_empty());
}

#[test]
fn netease_syllable_unclosed_paren() {
    let parser = NeteaseParser;
    // (1000,500text without closing paren → 找不到 ) 后 break
    let result = parser.parse_syllables(0, "(1000,500text").unwrap();
    assert!(result.is_empty());
}

#[test]
fn netease_syllable_non_digit_after_paren() {
    let parser = NeteaseParser;
    // ( 后不是数字 → skip
    let result = parser.parse_syllables(0, "(abc,500)Text").unwrap();
    assert!(result.is_empty());
}

#[test]
fn netease_syllable_text_between_tags() {
    let parser = NeteaseParser;
    // 验证 text 在 ) 和 ( 之间
    let result = parser
        .parse_syllables(0, "(0,100)First(100,200)Second(300,100)Third")
        .unwrap();
    assert_eq!(result.len(), 3);
    assert_eq!(result[0].text, "First");
    assert_eq!(result[1].text, "Second");
    assert_eq!(result[2].text, "Third");
}

#[test]
fn netease_syllable_zero_offset() {
    let parser = NeteaseParser;
    // s=500, s1=500 → offset=0
    let result = parser.parse_syllables(500, "(500,100)Zero").unwrap();
    assert_eq!(result[0].start_time, 0);
}
