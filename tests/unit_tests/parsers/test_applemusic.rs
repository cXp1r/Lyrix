use lyrix::parsers::applemusic::AppleMusicParser;

// ── AppleMusicParser::parse_time ──────────────────────────────
// 格式: H:MM:SS.cc
// 公式: hours*3_600_000 + minutes*60_000 + seconds*1_000 + centis*10

#[test]
fn am_parse_time_valid() {
    let parser = AppleMusicParser {};
    // 01:02:03.04 → 1*3_600_000 + 2*60_000 + 3*1_000 + 4*10
    assert_eq!(parser.parse_time("01:02:03.04").unwrap(), 3_723_040);
}

#[test]
fn am_parse_time_zero() {
    let parser = AppleMusicParser {};
    assert_eq!(parser.parse_time("00:00:00.00").unwrap(), 0);
}

#[test]
fn am_parse_time_no_colon() {
    let parser = AppleMusicParser {};
    assert!(parser.parse_time("bad").is_err());
}

#[test]
fn am_parse_time_one_colon_only() {
    let parser = AppleMusicParser {};
    // 只有第一个 ':', 缺少第二个
    assert!(parser.parse_time("01:02").is_err());
}

#[test]
fn am_parse_time_missing_second_colon() {
    let parser = AppleMusicParser {};
    // 有第一个 ':' 但没有第二个 ':'（rest 里没有 ':'）
    assert!(parser.parse_time("01:0203.10").is_err());
}

#[test]
fn am_parse_time_no_dot() {
    let parser = AppleMusicParser {};
    assert!(parser.parse_time("01:02:03").is_err());
}

#[test]
fn am_parse_time_non_numeric() {
    let parser = AppleMusicParser {};
    assert!(parser.parse_time("01:xx:03.10").is_err());
}

#[test]
fn am_parse_time_empty_field() {
    let parser = AppleMusicParser {};
    assert!(parser.parse_time("01::03.10").is_err());
}

// ── AppleMusicParser::parse_syllables_time ────────────────────
// 格式: MM:SS.cc（无小时）
// 公式: minutes*60_000 + seconds*1_000 + centis（不乘10）

#[test]
fn am_parse_syllables_time_valid() {
    let parser = AppleMusicParser {};
    assert_eq!(parser.parse_syllables_time("01:02.03").unwrap(), 62_003);
}

#[test]
fn am_parse_syllables_time_no_colon() {
    let parser = AppleMusicParser {};
    // 没有 ':' 时 minutes = 0
    let ms = parser.parse_syllables_time("02.03").unwrap();
    assert_eq!(ms, 2 * 1000 + 3); // seconds=2, centis=3
}

#[test]
fn am_parse_syllables_time_no_dot() {
    let parser = AppleMusicParser {};
    assert!(parser.parse_syllables_time("0102").is_err());
}

#[test]
fn am_parse_syllables_time_non_numeric() {
    let parser = AppleMusicParser {};
    assert!(parser.parse_syllables_time("xx:yy.zz").is_err());
}

// ════════════════════════════════════════════════════════════════
// Apple Music 的 get_offset_time 是 private fn，无法从外部直接测试。
// 其逻辑通过 parse_syllables_line / parse_w 间接覆盖：
//   - 合法 offset: parse_syllables_line 计算 start_time / duration 时用到
//   - underflow: end_time < start_time 时 parse_syllables_line 返回 OffsetOverflow

// ── AppleMusicParser::parse_without_st 分发逻辑 ───────────────

#[test]
fn am_parse_without_st_no_span_no_div() {
    let parser = AppleMusicParser {};
    // 不含 span → 走 parse_w，但需要 <div> 和 ="..." 属性
    let err = parser
        .parse_without_st("just plain text without any tags".into())
        .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("div") || msg.contains("body"));
}

#[test]
fn am_parse_syllables_line_missing_in_attr() {
    let parser = AppleMusicParser {};
    // 缺少 in="..." 属性，但有 nd= — 时间必须是 MM:SS.cc 格式
    let err = parser
        .parse_syllables_line(r#"<p nd="01:02.03">text</p>"#)
        .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("start_time"));
}

#[test]
fn am_parse_syllables_line_missing_nd_attr() {
    let parser = AppleMusicParser {};
    // 有 in= 但缺少 nd= — in= 的值需要是合法的 MM:SS.cc 格式
    let err = parser
        .parse_syllables_line(r#"<p in="01:02.03">text</p>"#)
        .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("end_time"));
}
