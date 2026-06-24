use lyrix::error::parser::parse::ParseError;
use lyrix::error::LyrixError;
use lyrix::parsers::IParsers;

// ── Dummy parser for testing default IParsers implementations ─
struct DummyParser;
impl IParsers for DummyParser {}

// ── get_offset_time ──────────────────────────────────────────

#[test]
fn iparsers_offset_valid_diff() {
    let p = DummyParser;
    assert_eq!(p.get_offset_time(1000, 1500).unwrap(), 500);
}

#[test]
fn iparsers_offset_zero_diff() {
    let p = DummyParser;
    assert_eq!(p.get_offset_time(100, 100).unwrap(), 0);
}

#[test]
fn iparsers_offset_t2_lt_t1_underflow() {
    let p = DummyParser;
    let err = p.get_offset_time(2000, 1000).unwrap_err();
    assert!(matches!(
        err,
        LyrixError::Parser(lyrix::error::ParserError::Parse(
            ParseError::OffsetOverflow { .. }
        ))
    ));
}

// ── parse_without_st ─────────────────────────────────────────

#[test]
fn iparsers_parse_empty() {
    let p = DummyParser;
    let result = p.parse(String::new()).unwrap();
    assert!(result.is_empty());
}

#[test]
fn iparsers_parse_single_line() {
    let p = DummyParser;
    let lyrics = "[1000,500]Hello".to_string();
    let result = p.parse(lyrics).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].start_time, 1000);
    assert_eq!(result[0].duration, 500);
}

#[test]
fn iparsers_parse_multiple_lines() {
    let p = DummyParser;
    let lyrics = "[0,1000]First[2000,500]Second".to_string();
    let result = p.parse(lyrics).unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].start_time, 0);
    assert_eq!(result[1].start_time, 2000);
}

#[test]
fn iparsers_parse_skips_non_digit_tags() {
    let p = DummyParser;
    let lyrics = "[ti:Title][0,500]Real".to_string();
    let result = p.parse(lyrics).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].start_time, 0);
}

#[test]
fn iparsers_parse_multiple_metadata_tags_skipped() {
    let p = DummyParser;
    let lyrics = "[ti:Title][ar:Artist][offset:0][100,200]Content".to_string();
    let result = p.parse(lyrics).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].start_time, 100);
}

#[test]
fn iparsers_parse_no_opening_bracket() {
    let p = DummyParser;
    let lyrics = "no brackets at all".to_string();
    let result = p.parse(lyrics).unwrap();
    assert!(result.is_empty());
}

#[test]
fn iparsers_parse_large_timestamps() {
    let p = DummyParser;
    let lyrics = "[3000000,500]Long".to_string();
    let result = p.parse(lyrics).unwrap();
    assert_eq!(result[0].start_time, 3000000);
}

// ── parse_syllables (default impl: <s,d> 格式) ────────────────

#[test]
fn iparsers_syllable_single() {
    let p = DummyParser;
    // s=0, 音节 <0,500>Hello — offset = 0-0 = 0
    let result = p.parse_syllables(0, "<0,500>Hello").unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].start_time, 0);
    assert_eq!(result[0].duration, 500);
    assert_eq!(result[0].text, "Hello");
}

#[test]
fn iparsers_syllable_multiple() {
    let p = DummyParser;
    let result = p.parse_syllables(1000, "<1000,200>A<1200,300>B").unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].text, "A");
    assert_eq!(result[0].start_time, 0); // 1000-1000
    assert_eq!(result[0].duration, 200);
    assert_eq!(result[1].text, "B");
    assert_eq!(result[1].start_time, 200); // 1200-1000
    assert_eq!(result[1].duration, 300);
}

#[test]
fn iparsers_syllable_triplet() {
    let p = DummyParser;
    let result = p.parse_syllables(0, "<0,500,0>Text").unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].duration, 500);
    assert_eq!(result[0].text, "Text");
}

#[test]
fn iparsers_syllable_empty() {
    let p = DummyParser;
    let result = p.parse_syllables(0, "").unwrap();
    assert!(result.is_empty());
}

#[test]
fn iparsers_syllable_no_angle_brackets() {
    let p = DummyParser;
    let result = p.parse_syllables(0, "plain text").unwrap();
    assert!(result.is_empty());
}

#[test]
fn iparsers_syllable_non_digit_after_angle() {
    let p = DummyParser;
    // < 后不是数字 → skip
    let result = p.parse_syllables(0, "<abc,500>Text").unwrap();
    assert!(result.is_empty());
}

#[test]
fn iparsers_syllable_with_spaces_in_text() {
    let p = DummyParser;
    let result = p.parse_syllables(0, "<0,100>Hello World<100,50>!").unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].text, "Hello World");
    assert_eq!(result[1].text, "!");
}
