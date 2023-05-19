use simple_process_tracker_rs::*;

#[test]
fn test_string_to_duration() {
    let res = string_to_duration("01:00:00");

    assert!(res.is_ok());

    let res = res.unwrap();

    assert_eq!(res, 3600);
}

#[test]
fn test_string_to_duration_invalid() {
    let res = string_to_duration("abc");

    assert!(res.is_err());

    let res = string_to_duration("0");

    assert!(res.is_err());
}

#[test]
fn test_duration_to_string() {
    assert_eq!(duration_to_string(3600), "01:00:00");
    assert_eq!(duration_to_string(123456), "34:17:36");
    assert_eq!(duration_to_string(0), "00:00:00");
}
