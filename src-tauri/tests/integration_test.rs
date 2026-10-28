use app_lib::commands::system::system_ping;

#[test]
fn test_system_ping_returns_pong() {
    let res = system_ping();
    assert_eq!(res.message, "pong");
    assert!(res.timestamp > 0);
    assert!(!res.arch.is_empty());
}
