use lyrix::error::fetcher::proxy::ProxyError;

#[test]
fn invalid_url_display() {
    let e = ProxyError::InvalidUrl {
        url: "http://bad proxy".into(),
        reason: "contains space".into(),
    };
    let msg = e.to_string();
    assert!(msg.contains("invalid proxy URL"));
    assert!(msg.contains("http://bad proxy"));
    assert!(msg.contains("contains space"));
}

#[test]
fn proxy_error_debug() {
    let e = ProxyError::InvalidUrl {
        url: "x".into(),
        reason: "y".into(),
    };
    let dbg = format!("{:?}", e);
    assert!(dbg.contains("InvalidUrl"));
}
