mod test_auth;
mod test_http;
mod test_json;
mod test_proxy;

use lyrix::error::provider::ProviderError;

#[test]
fn provider_error_from_http() {
    let e = lyrix::error::provider::http::HttpError::Timeout {
        url: "http://x.com".into(),
    };
    let pe: ProviderError = e.into();
    assert!(pe.to_string().contains("timeout"));
}

#[test]
fn provider_error_from_json() {
    let bad = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
    let e = lyrix::error::provider::json::JsonError {
        api: "ApiName".into(),
        source: bad,
    };
    let pe: ProviderError = e.into();
    assert!(pe.to_string().contains("ApiName"));
}

#[test]
fn provider_error_from_auth() {
    let e = lyrix::error::provider::auth::AuthError::MissingCredential {
        provider: "P".into(),
        field: "f".into(),
    };
    let pe: ProviderError = e.into();
    assert!(pe.to_string().contains("P"));
}

#[test]
fn provider_error_from_proxy() {
    let e = lyrix::error::provider::proxy::ProxyError::InvalidUrl {
        url: "u".into(),
        reason: "r".into(),
    };
    let pe: ProviderError = e.into();
    assert!(pe.to_string().contains("u"));
}

#[test]
fn provider_error_debug() {
    let e = ProviderError::Auth(lyrix::error::provider::auth::AuthError::CredentialExpired {
        provider: "X".into(),
        field: "y".into(),
    });
    let dbg = format!("{:?}", e);
    assert!(dbg.contains("Auth"));
}
