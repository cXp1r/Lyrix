use lyrix::parsers::lrc::LrcParser;
use lyrix::parsers::qqmusic::{QQMusicLrcParser, QQMusicParser};
use lyrix::parsers::IParsers;

// ── QQMusicLrcParser ──────────────────────────────────────────
// QQMusicLrcParser 使用默认 LrcParser 实现

#[test]
fn qqmusic_lrc_default_valid() {
    let parser = QQMusicLrcParser;
    // mm:ss.cc → minutes*60000 + seconds*1000 + centis*10
    let ms = parser.parse_lrc_time("01:02.03").unwrap();
    assert_eq!(ms, 1 * 60000 + 2 * 1000 + 3 * 10); // 62030
}

#[test]
fn qqmusic_lrc_default_invalid() {
    let parser = QQMusicLrcParser;
    assert!(parser.parse_lrc_time("bad").is_err());
}

// ── QQMusicParser::parse_syllables ────────────────────────────
// QQ Music 格式: text(s,d) — 文字在 '(' 之前（与 Netease 相反）

#[test]
fn qqmusic_syllable_text_before_time() {
    let parser = QQMusicParser;
    // text 在 ( 之前: "Hello" 是文字, (1000,500) 是时间标签
    let result = parser.parse_syllables(1000, "Hello(1500,500)").unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].text, "Hello");
    assert_eq!(result[0].start_time, 500); // 1500-1000
    assert_eq!(result[0].duration, 500);
}

#[test]
fn qqmusic_syllable_multiple() {
    let parser = QQMusicParser;
    let result = parser.parse_syllables(0, "A(0,200)B(200,300)").unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].text, "A");
    assert_eq!(result[0].duration, 200);
    assert_eq!(result[0].start_time, 0);
    assert_eq!(result[1].text, "B");
    assert_eq!(result[1].duration, 300);
    assert_eq!(result[1].start_time, 200);
}

#[test]
fn qqmusic_syllable_empty_text_before_paren() {
    let parser = QQMusicParser;
    // 如果 ( 前面没文字，text_raw 是空字符串
    let result = parser.parse_syllables(0, "(1000,500)").unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].text, "");
}

#[test]
fn qqmusic_syllable_triplet() {
    let parser = QQMusicParser;
    // (s,d,x) 三元组
    let result = parser.parse_syllables(0, "Text(1000,500,0)").unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].text, "Text");
    assert_eq!(result[0].duration, 500);
}

#[test]
fn qqmusic_syllable_empty() {
    let parser = QQMusicParser;
    let result = parser.parse_syllables(0, "").unwrap();
    assert!(result.is_empty());
}

#[test]
fn qqmusic_syllable_non_digit_after_paren() {
    let parser = QQMusicParser;
    // ( 后不是数字 → skip
    let result = parser.parse_syllables(0, "(abc,500)").unwrap();
    assert!(result.is_empty());
}

// ── QQMusicParser::decrypt_and_parse ─────────────────────────
// 需要 QRC 加密数据；无样本时标 #[ignore]

#[test]
#[ignore = "需要真实的 QRC 加密歌词数据"]
fn qqmusic_decrypt_and_parse_valid() {
    let parser = QQMusicParser;
    // 此测试需要从已知 API 响应中获取 QRC 密文
    let encrypted = ""; // TODO: 填入真实 QRC 加密字符串
    let result = parser.decrypt_and_parse(encrypted.into()).unwrap();
    assert!(!result.is_empty());
}

#[test]
fn qqmusic_decrypt_invalid_hex() {
    let parser = QQMusicParser;
    let result = parser.decrypt_and_parse("not-hex-string".into());
    assert!(result.is_err());
}
