use lyrix::error::parser::totp_gen::TotpGenError;

#[test]
fn invalid_index_display() {
    let e = TotpGenError::InvalidIndex { index: 5 };
    let msg = e.to_string();
    assert!(msg.contains("5"));
    assert!(msg.contains("invalid TOTP index"));
}

#[test]
fn clock_error_display() {
    let e = TotpGenError::ClockError;
    assert!(e.to_string().contains("system clock error"));
}

#[test]
fn totp_gen_error_debug() {
    let e = TotpGenError::InvalidIndex { index: 99 };
    let dbg = format!("{:?}", e);
    assert!(dbg.contains("InvalidIndex"));
}
