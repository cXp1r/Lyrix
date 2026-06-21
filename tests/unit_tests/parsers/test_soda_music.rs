use lyrix::parsers::soda_music::SodaParser;
use lyrix::parsers::IParsers;

#[test]
fn soda_parser_get_offset_time_returns_t2_as_u16() {
    let p = SodaParser;
    // 汽水音乐的 offset 直接就是 t2
    let result = p.get_offset_time(99999, 500).unwrap();
    assert_eq!(result, 500);
}

#[test]
fn soda_parser_get_offset_time_zero() {
    let p = SodaParser;
    let result = p.get_offset_time(123, 0).unwrap();
    assert_eq!(result, 0);
}

#[test]
fn soda_parser_get_offset_time_u16_max() {
    let p = SodaParser;
    let result = p.get_offset_time(0, 65535).unwrap();
    assert_eq!(result, 65535);
}

#[test]
fn soda_parser_get_offset_time_overflow() {
    let p = SodaParser;
    // t2 > u16::MAX should fail
    assert!(p.get_offset_time(0, 65536).is_err());
}

#[test]
fn soda_parser_parse_empty() {
    let p = SodaParser;
    let result = p.parse(String::new()).unwrap();
    assert!(result.is_empty());
}

#[test]
fn soda_parser_parse_basic_lyrics() {
    let p = SodaParser;
    // 汽水音乐歌词格式：[st,d]<s1,d1>text<s2,d2>text
    let lyrics = "[1000,500]<0,500,0>Hello";
    let result = p.parse(lyrics.into()).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].start_time, 1000);
    assert_eq!(result[0].duration, 500);
    assert!(!result[0].syllables.is_empty());
}

#[test]
fn soda_parser_parse_multiple_lines() {
    let p = SodaParser;
    let lyrics = "[0,1000]<0,500,0>First\n[1000,500]Second";
    let result = p.parse(lyrics.into()).unwrap();
    assert_eq!(result.len(), 2);
}
