//! 测试 ISearcher::make_search_string
//! 使用 NeteaseSearcher 验证搜索候选字符串生成

use lyrix::models::{ITrackMetadata, TrackMetadata};
use lyrix::searchers::netease::NeteaseSearcher;
use lyrix::searchers::ISearcher;

fn make_search(track: &TrackMetadata) -> Vec<String> {
    NeteaseSearcher::new().make_search_string(track as &dyn ITrackMetadata)
}

#[test]
fn make_search_string_full() {
    let t = TrackMetadata {
        title: Some("Hello".into()),
        artist: Some("World".into()),
        album: Some("Album".into()),
        ..Default::default()
    };
    let candidates = make_search(&t);
    assert!(!candidates.is_empty());
    // 第一个候选: "Hello World" (title + artist)
    assert_eq!(candidates[0], "Hello World");
    // 应该生成多个候选
    assert!(candidates.len() <= 8);
}

#[test]
fn make_search_string_no_album() {
    let t = TrackMetadata {
        title: Some("Hello".into()),
        artist: Some("World".into()),
        album: None,
        ..Default::default()
    };
    let candidates = make_search(&t);
    assert!(!candidates.is_empty());
    // 不应包含专辑相关的候选
    for c in &candidates {
        assert!(!c.contains("Album"));
    }
}

#[test]
fn make_search_string_no_artist() {
    let t = TrackMetadata {
        title: Some("Hello".into()),
        artist: None,
        album: Some("Album".into()),
        ..Default::default()
    };
    let candidates = make_search(&t);
    // title-only 候选
    assert!(candidates.contains(&"Hello".to_string()));
}

#[test]
fn make_search_string_no_dedup_adjacent() {
    // 确保相邻相同候选被去重
    let t = TrackMetadata {
        title: Some("Song".into()),
        artist: Some("Artist".into()),
        album: Some("Album".into()),
        ..Default::default()
    };
    let candidates = make_search(&t);
    // 验证没有相邻重复
    for i in 1..candidates.len() {
        assert_ne!(candidates[i], candidates[i - 1], "adjacent duplicate found");
    }
}

#[test]
fn make_search_string_clean_title_interaction() {
    // title 会被 clean_title + remove_feat 处理
    let t = TrackMetadata {
        title: Some("Song (feat. X)".into()),
        artist: Some("Artist".into()),
        album: Some("Album".into()),
        ..Default::default()
    };
    let candidates = make_search(&t);
    // ct 应该是 "Song" (remove_feat → clean_title)
    assert!(candidates.contains(&"Song Artist".to_string()));
}

#[test]
fn make_search_string_empty_all() {
    let t = TrackMetadata {
        title: None,
        artist: None,
        album: None,
        ..Default::default()
    };
    let candidates = make_search(&t);
    // 全是空时应该为空
    assert!(candidates.is_empty());
}
