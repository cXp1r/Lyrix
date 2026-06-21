use lyrix::error::provider::auth::AuthError;

#[test]
fn missing_credential_display() {
    let e = AuthError::MissingCredential {
        provider: "Spotify".into(),
        field: "spotify_cookie".into(),
    };
    let msg = e.to_string();
    assert!(msg.contains("Spotify"));
    assert!(msg.contains("spotify_cookie"));
    assert!(msg.contains("missing"));
}

#[test]
fn credential_expired_display() {
    let e = AuthError::CredentialExpired {
        provider: "AppleMusic".into(),
        field: "token".into(),
    };
    let msg = e.to_string();
    assert!(msg.contains("AppleMusic"));
    assert!(msg.contains("token"));
    assert!(msg.contains("expired"));
}

#[test]
fn auth_error_debug() {
    let e = AuthError::MissingCredential {
        provider: "P".into(),
        field: "f".into(),
    };
    let dbg = format!("{:?}", e);
    assert!(dbg.contains("MissingCredential"));
}
