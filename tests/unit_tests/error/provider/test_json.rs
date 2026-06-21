use lyrix::error::provider::json::JsonError;

#[test]
fn json_error_display() {
    let result = serde_json::from_str::<serde_json::Value>("{bad json}");
    let e = JsonError {
        api: "TestApi".into(),
        source: result.unwrap_err(),
    };
    let msg = e.to_string();
    assert!(msg.contains("TestApi"));
    assert!(msg.contains("parse failed"));
}

#[test]
fn json_error_source() {
    let result = serde_json::from_str::<serde_json::Value>("invalid");
    let e = JsonError {
        api: "X".into(),
        source: result.unwrap_err(),
    };
    assert!(std::error::Error::source(&e).is_some());
}

#[test]
fn json_error_debug() {
    let result = serde_json::from_str::<serde_json::Value>("{");
    let e = JsonError {
        api: "DebugApi".into(),
        source: result.unwrap_err(),
    };
    let dbg = format!("{:?}", e);
    assert!(dbg.contains("DebugApi"));
}
