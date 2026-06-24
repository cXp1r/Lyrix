use lyrix::error::parser::parse::ParseError;

#[test]
fn invalid_structure_display() {
    let e = ParseError::InvalidStructure {
        detail: "missing '[' bracket".into(),
    };
    let msg = e.to_string();
    assert!(msg.contains("invalid lyrics structure"));
    assert!(msg.contains("missing '[' bracket"));
}

#[test]
fn timestamp_parse_display() {
    let e = ParseError::TimestampParse {
        field: "start_time".into(),
        raw: "abc".into(),
    };
    let msg = e.to_string();
    assert!(msg.contains("timestamp parse error"));
    assert!(msg.contains("start_time"));
    assert!(msg.contains("abc"));
}

#[test]
fn offset_overflow_display() {
    let e = ParseError::OffsetOverflow { t1: 5000, t2: 3000 };
    let msg = e.to_string();
    assert!(msg.contains("5000"));
    assert!(msg.contains("3000"));
}

#[test]
fn syllable_parse_display() {
    let e = ParseError::SyllableParse {
        detail: "s1 parse error".into(),
    };
    assert!(e.to_string().contains("s1 parse error"));
}

#[test]
fn empty_content_display() {
    let e = ParseError::EmptyContent;
    assert!(e.to_string().contains("empty lyrics content"));
}

#[test]
fn invalid_lrc_format_display() {
    let e = ParseError::InvalidLrcFormat {
        detail: "时间标签缺少 ':'".into(),
    };
    assert!(e.to_string().contains("invalid LRC format"));
    assert!(e.to_string().contains(":"));
}

#[test]
fn unknown_sync_type_display() {
    let e = ParseError::UnknownSyncType;
    assert!(e.to_string().contains("unknown lyrics sync type"));
}

#[test]
fn lyrics_parse_error_debug() {
    let e = ParseError::EmptyContent;
    let dbg = format!("{:?}", e);
    assert!(dbg.contains("EmptyContent"));
}
