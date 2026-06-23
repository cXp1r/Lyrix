use lyrix::error::fetcher::http::HttpError;

#[test]
fn bad_request_display() {
    let e = HttpError::BadRequest {
        url: "https://api.test/v1".into(),
    };
    assert!(e.to_string().contains("bad request"));
    assert!(e.to_string().contains("https://api.test/v1"));
}

#[test]
fn unauthorized_display() {
    let e = HttpError::Unauthorized {
        url: "https://api.test".into(),
    };
    assert!(e.to_string().contains("unauthorized"));
    assert!(e.to_string().contains("401"));
}

#[test]
fn forbidden_display() {
    let e = HttpError::Forbidden {
        url: "https://api.test".into(),
    };
    assert!(e.to_string().contains("forbidden"));
    assert!(e.to_string().contains("403"));
}

#[test]
fn not_found_display() {
    let e = HttpError::NotFound {
        url: "https://api.test/404".into(),
    };
    assert!(e.to_string().contains("not found"));
    assert!(e.to_string().contains("404"));
}

#[test]
fn too_many_requests_display() {
    let e = HttpError::TooManyRequests {
        url: "https://api.test".into(),
    };
    assert!(e.to_string().contains("too many requests"));
    assert!(e.to_string().contains("429"));
}

#[test]
fn server_error_display() {
    let e = HttpError::ServerError {
        url: "https://api.test".into(),
    };
    assert!(e.to_string().contains("server error"));
    assert!(e.to_string().contains("500"));
}

#[test]
fn bad_gateway_display() {
    let e = HttpError::BadGateway {
        url: "https://api.test".into(),
    };
    assert!(e.to_string().contains("bad gateway"));
    assert!(e.to_string().contains("502"));
}

#[test]
fn service_unavailable_display() {
    let e = HttpError::ServiceUnavailable {
        url: "https://api.test".into(),
    };
    assert!(e.to_string().contains("service unavailable"));
    assert!(e.to_string().contains("503"));
}

#[test]
fn redirect_display() {
    let e = HttpError::Redirect {
        status: 301,
        url: "https://old.test".into(),
    };
    assert!(e.to_string().contains("redirect"));
    assert!(e.to_string().contains("301"));
}

#[test]
fn other_status_display() {
    let e = HttpError::OtherStatus {
        status: 418,
        url: "https://teapot.test".into(),
    };
    assert!(e.to_string().contains("418"));
}

#[test]
fn connection_failed_display() {
    let e = HttpError::ConnectionFailed {
        detail: "DNS error".into(),
        url: "https://bad.test".into(),
    };
    assert!(e.to_string().contains("connection failed"));
    assert!(e.to_string().contains("DNS error"));
}

#[test]
fn timeout_display() {
    let e = HttpError::Timeout {
        url: "https://slow.test".into(),
    };
    assert!(e.to_string().contains("request timeout"));
}

#[test]
fn tls_error_display() {
    let e = HttpError::TlsError {
        detail: "certificate expired".into(),
        url: "https://expired.test".into(),
    };
    assert!(e.to_string().contains("TLS error"));
    assert!(e.to_string().contains("certificate expired"));
}

#[test]
fn http_error_debug() {
    let e = HttpError::NotFound {
        url: "https://x.com".into(),
    };
    let dbg = format!("{:?}", e);
    assert!(dbg.contains("NotFound"));
}
