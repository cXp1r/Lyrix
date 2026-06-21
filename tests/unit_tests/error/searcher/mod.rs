use lyrix::error::searcher::SearcherError;

#[test]
fn no_results_display() {
    let e = SearcherError::NoResults {
        label: "网易云".into(),
        query: "test song".into(),
    };
    let msg = e.to_string();
    assert!(msg.contains("网易云"));
    assert!(msg.contains("test song"));
    assert!(msg.contains("no search results"));
}

#[test]
fn low_score_display() {
    let e = SearcherError::LowScore {
        label: "QQ音乐".into(),
        score: 3,
        threshold: 5,
        query: "song".into(),
    };
    let msg = e.to_string();
    assert!(msg.contains("3"));
    assert!(msg.contains("5"));
    assert!(msg.contains("score too low"));
}

#[test]
fn no_match_display() {
    let e = SearcherError::NoMatch {
        label: "酷狗".into(),
        title: "Unknown Song".into(),
    };
    let msg = e.to_string();
    assert!(msg.contains("酷狗"));
    assert!(msg.contains("Unknown Song"));
    assert!(msg.contains("no matching track"));
}

#[test]
fn missing_field_display() {
    let e = SearcherError::MissingField {
        label: "Spotify".into(),
        field: "id".into(),
    };
    let msg = e.to_string();
    assert!(msg.contains("Spotify"));
    assert!(msg.contains("id"));
    assert!(msg.contains("missing required field"));
}

#[test]
fn searcher_error_debug() {
    let e = SearcherError::NoResults {
        label: "X".into(),
        query: "q".into(),
    };
    let dbg = format!("{:?}", e);
    assert!(dbg.contains("NoResults"));
}
