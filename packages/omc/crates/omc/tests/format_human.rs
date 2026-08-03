use omc::cli::ui::format_human;

#[test]
fn test_format_human_zero() {
    assert_eq!(format_human(0), "0");
}

#[test]
fn test_format_human_small_numbers() {
    assert_eq!(format_human(1), "1");
    assert_eq!(format_human(999), "999");
}

#[test]
fn test_format_human_thousands() {
    assert_eq!(format_human(1000), "1.0k");
    assert_eq!(format_human(1234), "1.2k");
    assert_eq!(format_human(1500), "1.5k");
    assert_eq!(format_human(12345), "12.3k");
    assert_eq!(format_human(999999), "999.9k");
}

#[test]
fn test_format_human_millions() {
    assert_eq!(format_human(1_000_000), "1.0m");
    assert_eq!(format_human(1_500_000), "1.5m");
    assert_eq!(format_human(2_345_678), "2.3m");
    assert_eq!(format_human(10_000_000), "10.0m");
}

#[test]
fn test_format_human_negative() {
    assert_eq!(format_human(-1234), "1.2k");
    assert_eq!(format_human(-1_500_000), "1.5m");
}
