use lyrix::parsers::lrc::LrcParser;

struct Dummy;
impl LrcParser for Dummy {}

#[test]
fn parse_lrc_time_valid() {
    let parser = Dummy;
    // [01:02.03] → 1*60000 + 2*1000 + 3*10 = 62030
    assert_eq!(parser.parse_lrc_time("01:02.03").unwrap(), 62030);
    // [00:00.00]
    assert_eq!(parser.parse_lrc_time("00:00.00").unwrap(), 0);
    // [59:59.99]
    assert_eq!(
        parser.parse_lrc_time("59:59.99").unwrap(),
        59 * 60000 + 59 * 1000 + 99 * 10
    );
}

#[test]
fn parse_lrc_time_invalid() {
    let parser = Dummy;
    assert!(parser.parse_lrc_time("not-a-time").is_err());
    assert!(parser.parse_lrc_time("00:01").is_err()); // 缺少 .
    assert!(parser.parse_lrc_time("00:01.").is_err()); // 缺少 centis
    assert!(parser.parse_lrc_time("00:xx.10").is_err()); // 非数字
    assert!(parser.parse_lrc_time("xx:00.10").is_err()); // 非数字
    assert!(parser.parse_lrc_time("00:00.xx").is_err()); // 非数字
    assert!(parser.parse_lrc_time("").is_err());
}

#[test]
fn parse_lrc_empty_lyrics() {
    let parser = Dummy;
    let result = parser.parse(String::new()).unwrap();
    assert!(result.is_empty());
}

#[test]
fn parse_lrc_single_line() {
    let parser = Dummy;
    let lyrics = "[00:01.50]Hello World";
    let result = parser.parse(lyrics.into()).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].start_time, 1 * 1000 + 50 * 10);
    assert_eq!(result[0].text, "Hello World");
}

#[test]
fn parse_lrc_multiple_lines() {
    let parser = Dummy;
    let lyrics = "[00:10.00]Line 1\n[00:20.00]Line 2\n[00:30.00]Line 3";
    let result = parser.parse(lyrics.into()).unwrap();
    assert_eq!(result.len(), 3);
    assert_eq!(result[0].start_time, 10_000);
    assert_eq!(result[1].start_time, 20_000);
    assert_eq!(result[2].start_time, 30_000);
}

#[test]
fn parse_lrc_skips_metadata_tags() {
    let parser = Dummy;
    let lyrics = "[ti:Title]\n[ar:Artist]\n[00:05.00]Real line";
    let result = parser.parse(lyrics.into()).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].text, "Real line");
}

#[test]
fn parse_lrc_trims_trailing_newline() {
    let parser = Dummy;
    let lyrics = "[00:05.00]Text\r\n";
    let result = parser.parse(lyrics.into()).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].text, "Text");
}

#[test]
fn parse_lrc_offset_tags() {
    let parser = Dummy;
    let lyrics = "[offset:0]\n[00:05.00]Song start";
    let result = parser.parse(lyrics.into()).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].text, "Song start");
}
