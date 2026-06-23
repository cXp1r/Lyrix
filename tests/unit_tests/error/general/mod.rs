use lyrix::error::general::GeneralError;

#[test]
fn unsupported_player_display() {
    let e = GeneralError::UnsupportedPlayer {
        name: "MyPlayer".into(),
    };
    assert!(e.to_string().contains("MyPlayer"));
    assert!(e.to_string().contains("unsupported"));
}

#[test]
fn missing_field_display() {
    let e = GeneralError::MissingField {
        field: "important_field".into(),
    };
    assert!(e.to_string().contains("important_field"));
    assert!(e.to_string().contains("missing"));
}

#[test]
fn io_error_from() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let e: GeneralError = io_err.into();
    assert!(e.to_string().contains("IO error"));
}

#[test]
fn internal_display() {
    let e = GeneralError::Internal {
        detail: "something went very wrong".into(),
    };
    assert!(e.to_string().contains("internal error"));
    assert!(e.to_string().contains("something went very wrong"));
}

#[test]
fn general_error_debug() {
    let e = GeneralError::MissingField { field: "x".into() };
    let debug = format!("{:?}", e);
    assert!(debug.contains("MissingField"));
}
