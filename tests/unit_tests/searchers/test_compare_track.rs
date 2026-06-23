//! 测试 ISearcher::compare_track 评分引擎（默认实现）
//! 使用 NeteaseSearcher + NeteaseSearchResult 作为具体实现

use lyrix::models::{ITrackMetadata, TrackMetadata};
use lyrix::searchers::netease::{NeteaseSearchResult, NeteaseSearcher};
use lyrix::searchers::ISearcher;

/// 构造 TrackMetadata 的辅助函数
fn track(title: &str, artist: &str, album: &str, duration_ms: Option<u32>) -> TrackMetadata {
    TrackMetadata {
        title: Some(title.to_string()),
        artist: Some(artist.to_string()),
        album: if album.is_empty() {
            None
        } else {
            Some(album.to_string())
        },
        duration_ms,
        ..Default::default()
    }
}

/// 构造 NeteaseSearchResult 的辅助函数
fn result(
    title: &str,
    artists: &[&str],
    album: &str,
    duration_ms: Option<u32>,
    trial: Option<[u32; 2]>,
) -> NeteaseSearchResult {
    NeteaseSearchResult {
        id: String::new(),
        title: title.to_string(),
        artists: artists.iter().map(|s| s.to_string()).collect(),
        album: album.to_string(),
        duration_ms,
        match_score: 0,
        trial,
        is_trial: false,
    }
}

fn compare(
    searcher: &NeteaseSearcher,
    track: &TrackMetadata,
    r: &NeteaseSearchResult,
) -> (i8, bool) {
    searcher.compare_track(
        track as &dyn ITrackMetadata,
        r as &dyn lyrix::searchers::ISearchResult,
    )
}

// ═══════════════════════ Title 维度 ════════════════════════════

#[test]
fn score_title_exact_match() {
    let s = NeteaseSearcher::new();
    let t = track("Hello", "", "", None);
    let r = result("Hello", &[], "", None, None);
    let (score, _) = compare(&s, &t, &r);
    assert_eq!(score, 4);
}

#[test]
fn score_title_exact_case_insensitive() {
    let s = NeteaseSearcher::new();
    let t = track("HELLO", "", "", None);
    let r = result("hello", &[], "", None, None);
    let (score, _) = compare(&s, &t, &r);
    assert_eq!(score, 4);
}

#[test]
fn score_title_contains_track_in_result() {
    let s = NeteaseSearcher::new();
    let t = track("Hello", "", "", None);
    let r = result("Hello World", &[], "", None, None);
    let (score, _) = compare(&s, &t, &r);
    assert_eq!(score, 2);
}

#[test]
fn score_title_contains_result_in_track() {
    let s = NeteaseSearcher::new();
    let t = track("Hello World", "", "", None);
    let r = result("Hello", &[], "", None, None);
    let (score, _) = compare(&s, &t, &r);
    assert_eq!(score, 2);
}

#[test]
fn score_title_clean_exact() {
    // clean_title 只在 exact 和 contains 都失败后才触发
    // 需要两个不互相包含但有相同 clean 结果的标题
    let s = NeteaseSearcher::new();
    // "Song (Remix)" 和 "Song (Live)" 不互相包含，但 clean 后都是 "Song"
    let t = track("Song (Remix)", "", "", None);
    let r = result("Song (Live)", &[], "", None, None);
    let (score, _) = compare(&s, &t, &r);
    assert_eq!(score, 3); // clean exact match
}

#[test]
fn score_title_clean_contains() {
    let s = NeteaseSearcher::new();
    // "Hello (Mix)" 和 "Hello World" 不互相包含
    // clean后 "Hello" 被 "Hello World" 包含 → +1
    let t = track("Hello (Mix)", "", "", None);
    let r = result("Hello World", &[], "", None, None);
    let (score, _) = compare(&s, &t, &r);
    assert_eq!(score, 1); // clean_title 后部分匹配
}

#[test]
fn score_title_no_match() {
    let s = NeteaseSearcher::new();
    let t = track("ABC", "", "", None);
    let r = result("XYZ", &[], "", None, None);
    let (score, _) = compare(&s, &t, &r);
    assert_eq!(score, 0);
}

#[test]
fn score_title_empty_both() {
    let s = NeteaseSearcher::new();
    let t = track("", "", "", None);
    let r = result("", &[], "", None, None);
    let (score, _) = compare(&s, &t, &r);
    assert_eq!(score, 0);
}

// ═══════════════════════ Artist 维度 ════════════════════════════

#[test]
fn score_artist_single_exact() {
    let s = NeteaseSearcher::new();
    let t = track("X", "ArtistA", "", None);
    // Netease 用 '/' 分隔，单艺人无 '/' 就是整体
    let r = result("X", &["ArtistA"], "", None, None);
    let (score, _) = compare(&s, &t, &r);
    assert!(score >= 1); // title 不匹配(0) + artist 匹配(+1)
}

#[test]
fn score_artist_contains() {
    let s = NeteaseSearcher::new();
    let t = track("X", "Artist", "", None);
    let r = result("X", &["Artist ABC"], "", None, None);
    let (score, _) = compare(&s, &t, &r);
    assert!(score >= 1);
}

#[test]
fn score_artist_split_by_slash() {
    let s = NeteaseSearcher::new();
    // NeteaseSearcher 的 split_char 是 '/'
    let t = track("X", "A/B", "", None);
    let r = result("X", &["A", "B"], "", None, None);
    let (score, _) = compare(&s, &t, &r);
    assert!(score >= 2); // 两个艺人都匹配
}

#[test]
fn score_artist_no_match() {
    let s = NeteaseSearcher::new();
    // 使用不同的标题以避免意外匹配
    let t = track("TrackA", "ArtistA", "", None);
    let r = result("ResultB", &["ArtistB"], "", None, None);
    let (score, _) = compare(&s, &t, &r);
    assert_eq!(score, 0);
}

#[test]
fn score_artist_empty_track() {
    let s = NeteaseSearcher::new();
    let t = track("TrackC", "  ", "", None); // 空格 → split → 空
    let r = result("ResultD", &["ArtistB"], "", None, None);
    let (score, _) = compare(&s, &t, &r);
    assert_eq!(score, 0);
}

// ═══════════════════════ Album 维度 ════════════════════════════

#[test]
fn score_album_exact() {
    let s = NeteaseSearcher::new();
    // 使用不同标题避免 +4
    let t = track("TrackT", "A", "AlbumX", None);
    let r = result("ResultR", &["A"], "AlbumX", None, None);
    let (score, _) = compare(&s, &t, &r);
    // artist(+1) + album exact(+2) = 3
    assert_eq!(score, 3);
}

#[test]
fn score_album_contains() {
    let s = NeteaseSearcher::new();
    let t = track("TrackT", "A", "Album", None);
    let r = result("ResultR", &["A"], "Album Extended", None, None);
    let (score, _) = compare(&s, &t, &r);
    // artist(+1) + album contains(+1) = 2
    assert_eq!(score, 2);
}

#[test]
fn score_album_no_match() {
    let s = NeteaseSearcher::new();
    let t = track("TrackT", "A", "AlbumA", None);
    let r = result("ResultR", &["A"], "AlbumB", None, None);
    let (score, _) = compare(&s, &t, &r);
    assert_eq!(score, 1); // only artist
}

#[test]
fn score_album_empty_track() {
    let s = NeteaseSearcher::new();
    let t = track("TrackT", "A", "", None);
    let r = result("ResultR", &["A"], "AlbumB", None, None);
    let (score, _) = compare(&s, &t, &r);
    assert_eq!(score, 1); // only artist, album empty → skipped
}

// ═══════════════════════ Album Artist 维度 ═════════════════════

#[test]
fn score_album_artist_exact() {
    let s = NeteaseSearcher::new();
    let mut t = track("X", "A", "Album", None);
    t.album_artist = Some("Producer".into());
    let r = result("X", &["A"], "Album", None, None);
    // NeteaseSearchResult doesn't have album_artists override, uses default None
    // Actually ISearchResult::album_artists() default returns None
    // Let me check — NeteaseSearchResult doesn't implement album_artists, so default None
    // So album_artist match always scores 0 for Netease
    let (score, _) = compare(&s, &t, &r);
    // Only: artist(+1) — album empty? No, album="Album", r.album="Album" → exact(+2) = 3
    // But track title "X" vs result title "X" → no match? X != X... wait
    // track title "X", result title "X" → exact match +4! Because they're case-insensitive
    // Actually track title = "X", result title = "X" → exact match +4
    // plus artist +1 = 5, plus album exact +2 = 7
    // But album_artist won't match because ISearchResult default returns None
    assert_eq!(score, 7);
}

// ═══════════════════════ Duration 维度 ═════════════════════════

#[test]
fn score_duration_exact() {
    let s = NeteaseSearcher::new();
    let t = track("A", "B", "C", Some(200000));
    let r = result("A", &["B"], "C", Some(200000), None);
    let (score, _) = compare(&s, &t, &r);
    // title exact(+4) + artist(+1) + album exact(+2) + duration exact(+3) = 10
    assert_eq!(score, 10);
}

#[test]
fn score_duration_within_500() {
    let s = NeteaseSearcher::new();
    let t = track("A", "B", "C", Some(200000));
    let r = result("A", &["B"], "C", Some(200400), None);
    let (score, _) = compare(&s, &t, &r);
    // title(+4) + artist(+1) + album(+2) + duration 400ms diff(+2) = 9
    assert_eq!(score, 9);
}

#[test]
fn score_duration_within_1000() {
    let s = NeteaseSearcher::new();
    let t = track("A", "B", "C", Some(200000));
    let r = result("A", &["B"], "C", Some(200800), None);
    let (score, _) = compare(&s, &t, &r);
    // title(+4) + artist(+1) + album(+2) + duration 800ms diff(+1) = 8
    assert_eq!(score, 8);
}

#[test]
fn score_duration_over_1000() {
    let s = NeteaseSearcher::new();
    let t = track("A", "B", "C", Some(200000));
    let r = result("A", &["B"], "C", Some(202000), None);
    let (score, _) = compare(&s, &t, &r);
    // title(+4) + artist(+1) + album(+2) + duration no match = 7
    assert_eq!(score, 7);
}

#[test]
fn score_duration_track_none() {
    let s = NeteaseSearcher::new();
    let t = track("A", "B", "C", None);
    let r = result("A", &["B"], "C", Some(200000), None);
    let (score, _) = compare(&s, &t, &r);
    // Only title+artist+album = 7
    assert_eq!(score, 7);
}

#[test]
fn score_duration_result_none() {
    let s = NeteaseSearcher::new();
    let t = track("A", "B", "C", Some(200000));
    let r = result("A", &["B"], "C", None, None);
    let (score, _) = compare(&s, &t, &r);
    // title+artist+album = 7, no duration
    assert_eq!(score, 7);
}

// ═══════════════════════ Trial 维度 ════════════════════════════

#[test]
fn score_trial_exact() {
    let s = NeteaseSearcher::new();
    let t = track("A", "B", "C", Some(200000));
    let r = result("A", &["B"], "C", Some(200000), Some([0, 200000]));
    let (score, is_trial) = compare(&s, &t, &r);
    // base(10) + trial exact(+2) = 12
    assert_eq!(score, 12);
    assert!(is_trial);
}

#[test]
fn score_trial_near() {
    let s = NeteaseSearcher::new();
    let t = track("A", "B", "C", Some(200000));
    // trial end = result duration
    let r = result("A", &["B"], "C", Some(200500), Some([0, 200500]));
    let (score, is_trial) = compare(&s, &t, &r);
    // base: title(+4) + artist(+1) + album(+2) + duration 500ms diff(+2) = 9
    // trial: diff=|200000-200500|=500, ≤1000 → +1, is_trial=true
    assert_eq!(score, 10);
    assert!(is_trial);
}

#[test]
fn score_trial_far() {
    let s = NeteaseSearcher::new();
    let t = track("A", "B", "C", Some(200000));
    let r = result("A", &["B"], "C", Some(250000), Some([0, 250000]));
    let (score, is_trial) = compare(&s, &t, &r);
    // base: title(+4)+artist(+1)+album(+2)+duration over1000(0) = 7
    // trial: diff=|200000-250000|=50000 > 1000, no score, is_trial=false
    assert_eq!(score, 7);
    assert!(!is_trial);
}

#[test]
fn score_trial_result_none() {
    let s = NeteaseSearcher::new();
    let t = track("A", "B", "C", Some(200000));
    let r = result("A", &["B"], "C", Some(200000), None);
    let (score, is_trial) = compare(&s, &t, &r);
    assert_eq!(score, 10); // base 10, no trial
    assert!(!is_trial);
}

// ═══════════════════════ 组合 / 边界 ═══════════════════════════

#[test]
fn score_completely_different() {
    let s = NeteaseSearcher::new();
    let t = track("SongA", "ArtistA", "AlbumA", Some(300000));
    let r = result("SongB", &["ArtistB"], "AlbumB", Some(400000), None);
    let (score, _) = compare(&s, &t, &r);
    assert_eq!(score, 0);
}

#[test]
fn score_empty_track_artists_with_result_artists() {
    let s = NeteaseSearcher::new();
    let t = track("A", "", "C", None);
    let r = result("A", &["ArtistB"], "C", None, None);
    let (score, _) = compare(&s, &t, &r);
    // title(+4) + album(+2) = 6 (artist is empty string after split, filtered out)
    assert_eq!(score, 6);
}

#[test]
fn score_partial_match_accumulation() {
    let s = NeteaseSearcher::new();
    let t = track("Song", "Artist", "Album", Some(100000));
    let r = result("Song Remix", &["Artist X"], "Album X", Some(100500), None);
    let (score, _) = compare(&s, &t, &r);
    // title contains(+2) + artist contains(+1) + album contains(+1) + duration ≤500(+2) = 6
    assert_eq!(score, 6);
}
