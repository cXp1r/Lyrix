#[path = "general/mod.rs"]
mod test_general;

#[path = "parser/mod.rs"]
mod test_parser;

#[path = "fetcher/mod.rs"]
mod fetcher;

#[path = "searcher/mod.rs"]
mod test_searcher;

use lyrix::error::FetcherError;
use lyrix::error::GeneralError;
use lyrix::error::LyrixError;
use lyrix::error::ParserError;
use lyrix::error::SearcherError;

#[test]
fn lyrix_error_from_parser() {
    let e = lyrix::error::parser::lyrics_parse::LyricsParseError::EmptyContent;
    let lyrix_err: LyrixError = e.into();
    let msg = lyrix_err.to_string();
    assert!(msg.contains("empty lyrics content"));
}

#[test]
fn lyrix_error_from_http() {
    let e = lyrix::error::fetcher::http::HttpError::NotFound {
        url: "http://example.com".into(),
    };
    let lyrix_err: LyrixError = e.into();
    let msg = lyrix_err.to_string();
    assert!(msg.contains("not found"));
    assert!(msg.contains("http://example.com"));
}

#[test]
fn lyrix_error_from_json() {
    let bad_json = serde_json::from_str::<serde_json::Value>("not json");
    let e = lyrix::error::fetcher::json::JsonError {
        api: "TestApi".into(),
        source: bad_json.unwrap_err(),
    };
    let lyrix_err: LyrixError = e.into();
    let msg = lyrix_err.to_string();
    assert!(msg.contains("TestApi"));
}

#[test]
fn lyrix_error_from_auth() {
    let e = lyrix::error::fetcher::auth::AuthError::MissingCredential {
        provider: "Spotify".into(),
        field: "cookie".into(),
    };
    let lyrix_err: LyrixError = e.into();
    let msg = lyrix_err.to_string();
    assert!(msg.contains("Spotify"));
    assert!(msg.contains("cookie"));
}

#[test]
fn lyrix_error_from_proxy() {
    let e = lyrix::error::fetcher::proxy::ProxyError::InvalidUrl {
        url: "bad://url".into(),
        reason: "invalid scheme".into(),
    };
    let lyrix_err: LyrixError = e.into();
    let msg = lyrix_err.to_string();
    assert!(msg.contains("bad://url"));
}

#[test]
fn lyrix_error_from_general() {
    let e = GeneralError::UnsupportedPlayer {
        name: "TestPlayer".into(),
    };
    let lyrix_err: LyrixError = e.into();
    let msg = lyrix_err.to_string();
    assert!(msg.contains("TestPlayer"));
}

#[test]
fn lyrix_error_from_searcher() {
    let e = lyrix::error::SearcherError::NoResults {
        label: "Test".into(),
        query: "q".into(),
    };
    let lyrix_err: LyrixError = e.into();
    let msg = lyrix_err.to_string();
    assert!(msg.contains("Test"));
    assert!(msg.contains("q"));
}

#[test]
fn lyrix_error_debug() {
    let e = GeneralError::MissingField {
        field: "test_field".into(),
    };
    let lyrix_err = LyrixError::General(e);
    assert!(format!("{:?}", lyrix_err).contains("MissingField"));
}

#[test]
fn parser_error_display() {
    let e = ParserError::LyricsParse(
        lyrix::error::parser::lyrics_parse::LyricsParseError::UnknownSyncType,
    );
    assert!(e.to_string().contains("unknown lyrics sync type"));
}

#[test]
fn provider_error_display() {
    let e = FetcherError::Auth(lyrix::error::fetcher::auth::AuthError::CredentialExpired {
        provider: "X".into(),
        field: "token".into(),
    });
    assert!(e.to_string().contains("X"));
    assert!(e.to_string().contains("expired"));
}

#[test]
fn searcher_error_display() {
    let e = SearcherError::LowScore {
        label: "Test".into(),
        score: 3,
        threshold: 5,
        query: "song".into(),
    };
    let msg = e.to_string();
    assert!(msg.contains("3"));
    assert!(msg.contains("5"));
}
