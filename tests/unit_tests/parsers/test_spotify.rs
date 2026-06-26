use lyrix::error::parser::parse::ParseError;
use lyrix::error::LyrixError;
use lyrix::parsers::spotify::SpotifyParser;

// ── SpotifyParser::parse_without_st ───────────────────────────
// Spotify 是独立的 JSON 解析器，不实现 IParsers trait

fn spotify_json(sync_type: &str, lines: &str) -> String {
    format!(
        r#"{{"lyrics":{{"syncType":"{}","lines":{}}}}}"#,
        sync_type, lines
    )
}

#[test]
fn spotify_parse_valid_two_lines() {
    let parser = SpotifyParser;
    let json = spotify_json(
        "LINE_SYNCED",
        r#"[
            {"startTimeMs":"0","words":"Hello","endTimeMs":"2000"},
            {"startTimeMs":"2000","words":"World","endTimeMs":"4500"}
        ]"#,
    );
    let result = parser.parse_without_st(json).unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].text, "Hello");
    assert_eq!(result[0].start_time, 0);
    assert_eq!(result[0].duration, 2000);
    assert_eq!(result[1].text, "World");
    assert_eq!(result[1].start_time, 2000);
    assert_eq!(result[1].duration, 2500); // 4500-2000
}

#[test]
fn spotify_parse_unknown_sync_type() {
    let parser = SpotifyParser;
    let json = spotify_json("UNSYNCED", "[]");
    let err = parser.parse_without_st(json).unwrap_err();
    assert!(matches!(
        err,
        LyrixError::Parser(lyrix::error::ParserError::Parse(
            ParseError::UnknownSyncType
        ))
    ));
}

#[test]
fn spotify_parse_null_lines_is_empty_content() {
    let parser = SpotifyParser;
    let json = r#"{"lyrics":{"syncType":"LINE_SYNCED","lines":null}}"#;
    let err = parser.parse_without_st(json.into()).unwrap_err();
    assert!(matches!(
        err,
        LyrixError::Parser(lyrix::error::ParserError::Parse(ParseError::EmptyContent))
    ));
}

#[test]
fn spotify_parse_missing_lines_field() {
    let parser = SpotifyParser;
    let json = r#"{"lyrics":{"syncType":"LINE_SYNCED"}}"#;
    let err = parser.parse_without_st(json.into()).unwrap_err();
    // serde 把 missing Option field 当 None → EmptyContent
    assert!(matches!(
        err,
        LyrixError::Parser(lyrix::error::ParserError::Parse(ParseError::EmptyContent))
    ));
}

#[test]
fn spotify_parse_skip_empty_words() {
    let parser = SpotifyParser;
    let json = spotify_json(
        "LINE_SYNCED",
        r#"[
            {"startTimeMs":"0","words":"","endTimeMs":"1000"},
            {"startTimeMs":"1000","words":"Real","endTimeMs":"2000"}
        ]"#,
    );
    let result = parser.parse_without_st(json).unwrap();
    // 空 words 被跳过
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].text, "Real");
}

#[test]
fn spotify_parse_missing_start_time_defaults_zero() {
    let parser = SpotifyParser;
    let json =
        r#"{"lyrics":{"syncType":"LINE_SYNCED","lines":[{"words":"Hi","endTimeMs":"1000"}]}}"#;
    let result = parser.parse_without_st(json.into()).unwrap();
    assert_eq!(result[0].start_time, 0);
}

#[test]
fn spotify_parse_end_before_start_zero_duration() {
    let parser = SpotifyParser;
    let json = spotify_json(
        "LINE_SYNCED",
        r#"[{"startTimeMs":"2000","words":"Back","endTimeMs":"1000"}]"#,
    );
    let result = parser.parse_without_st(json).unwrap();
    assert_eq!(result[0].duration, 0);
}

#[test]
fn spotify_parse_invalid_json() {
    let parser = SpotifyParser;
    let err = parser.parse_without_st("not json".into()).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("JSON"));
}

#[test]
fn spotify_parse_invalid_timestamp() {
    let parser = SpotifyParser;
    let json = r#"{"lyrics":{"syncType":"LINE_SYNCED","lines":[{"startTimeMs":"abc","words":"Hi","endTimeMs":"1000"}]}}"#;
    let err = parser.parse_without_st(json.into()).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("timestamp") || msg.contains("startTimeMs"));
}
