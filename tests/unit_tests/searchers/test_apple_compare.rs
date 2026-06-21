//! 测试 Apple Music 自定义 compare_track
//! 与默认实现的差异:
//!   1. 艺人按 "—" 拆分，取第一段，再按 "、" 拆分
//!   2. 专辑从 "—" 第二段取，仅完全匹配 (+1 而非 +2)
//!   3. 无 trial 评分 (is_trial 永远 false)

use lyrix::models::{ITrackMetadata, TrackMetadata};
use lyrix::searchers::applemusic::{ApplemusicSearcher, ApplemusicSearchResult};
use lyrix::searchers::ISearchResult;
use lyrix::searchers::ISearcher;

fn apple_track(title: &str, artist: &str, album_artist: &str, duration_ms: Option<u32>) -> TrackMetadata {
    TrackMetadata {
        title: Some(title.to_string()),
        artist: Some(artist.to_string()),
        album: None, // Apple Music 专辑从 artist 字段 "—" 拆分取第二段
        album_artist: if album_artist.is_empty() { None } else { Some(album_artist.to_string()) },
        duration_ms,
        ..Default::default()
    }
}

fn apple_result(title: &str, artists: &[&str], album: &str, duration_ms: Option<u32>) -> ApplemusicSearchResult {
    ApplemusicSearchResult {
        id: String::new(),
        title: title.to_string(),
        artists: artists.iter().map(|s| s.to_string()).collect(),
        album: album.to_string(),
        duration_ms,
        match_score: 0,
        has_lyrics: true,
    }
}

// ── Artist 拆分: "Artists—Album" ────────────────────────────

#[test]
fn apple_score_artist_split_em_dash() {
    let s = ApplemusicSearcher::new("".into());
    // artist = "A—Album" → "—" 拆分 → ["a", "album"]
    // 取 index 0 = "a" → 按 "、" 再拆 → ["a"]
    let t = apple_track("Song", "A—AlbumName", "", None);
    let r = apple_result("Song", &["A"], "AlbumName", None);
    let (score, is_trial) = s.compare_track(&t as &dyn ITrackMetadata, &r as &dyn ISearchResult);
    // title exact(+4) + artist(+1) + album exact(from em dash)(+1) = 6
    assert_eq!(score, 6);
    assert!(!is_trial);
}

#[test]
fn apple_score_artist_split_jp_comma() {
    let s = ApplemusicSearcher::new("".into());
    // artist = "A、B—Album" → "—" 拆分 → ["a、b", "album"]
    // index 0 = "a、b" → "、" 拆分 → ["a", "b"]
    let t = apple_track("Song", "A、B—AlbumName", "", None);
    let r = apple_result("Song", &["A", "B"], "AlbumName", None);
    let (score, _) = s.compare_track(&t as &dyn ITrackMetadata, &r as &dyn ISearchResult);
    // title exact(+4) + artist A(+1) + artist B(+1) + album exact(+1) = 7
    assert_eq!(score, 7);
}

// ── Album: 从 "—" 第二段取，仅完全匹配 ──────────────────────

#[test]
fn apple_score_album_from_em_dash_exact() {
    let s = ApplemusicSearcher::new("".into());
    // artist = "Art—Alb", album_from_split = "Alb"
    let t = apple_track("Song", "Art—Alb", "", None);
    let r = apple_result("Song", &["Art"], "Alb", None);
    let (score, _) = s.compare_track(&t as &dyn ITrackMetadata, &r as &dyn ISearchResult);
    // title(+4) + artist(+1) + album exact(+1) = 6
    assert_eq!(score, 6);
}

#[test]
fn apple_score_album_contains_no_match() {
    // Apple Music 专辑匹配仅支持完全匹配，不支持 contains
    let s = ApplemusicSearcher::new("".into());
    let t = apple_track("Song", "Art—Album", "", None);
    // track album = "Album", result album = "Album Extended"
    let r = apple_result("Song", &["Art"], "Album Extended", None);
    let (score, _) = s.compare_track(&t as &dyn ITrackMetadata, &r as &dyn ISearchResult);
    // title(+4) + artist(+1) = 5, album NO match (contains doesn't apply)
    assert_eq!(score, 5);
}

#[test]
fn apple_score_album_empty_on_no_dash() {
    let s = ApplemusicSearcher::new("".into());
    // artist 不含 "—" → split 只有一个元素 → album = ""
    let t = apple_track("Song", "JustArtist", "", None);
    let r = apple_result("Song", &["JustArtist"], "AnyAlbum", None);
    let (score, _) = s.compare_track(&t as &dyn ITrackMetadata, &r as &dyn ISearchResult);
    // title(+4) + artist(+1) = 5, album NO match (track album is "")
    assert_eq!(score, 5);
}

// ── Trial 永远 false ─────────────────────────────────────────

#[test]
fn apple_score_always_no_trial() {
    let s = ApplemusicSearcher::new("".into());
    let t = apple_track("Song", "A—B", "", Some(200000));
    let r = apple_result("Song", &["A"], "B", Some(200000));
    let (_, is_trial) = s.compare_track(&t as &dyn ITrackMetadata, &r as &dyn ISearchResult);
    assert!(!is_trial);
}
