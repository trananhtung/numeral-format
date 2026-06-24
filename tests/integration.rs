//! Integration tests exercising the public API of `numeral-format`.

use numeral_format::format;

#[test]
fn common_display_formats() {
    assert_eq!(format(1234.5678, "0,0.00"), "1,234.57");
    assert_eq!(format(1234.5678, "0,0"), "1,235");
    assert_eq!(format(0.0, "0,0.000"), "0.000");
    assert_eq!(format(-1234.5, "0,0.0"), "-1,234.5");
}

#[test]
fn abbreviations_and_units() {
    assert_eq!(format(1_500_000.0, "0.0a"), "1.5m");
    assert_eq!(format(2_500_000_000.0, "0.0a"), "2.5b");
    assert_eq!(format(1500.0, "0.0b"), "1.5KB");
    assert_eq!(format(1536.0, "0.0ib"), "1.5KiB");
}

#[test]
fn currency_percentage_ordinal() {
    assert_eq!(format(1234.56, "$0,0.00"), "$1,234.56");
    assert_eq!(format(-50.0, "($0,0.00)"), "($50.00)");
    assert_eq!(format(0.25, "0%"), "25%");
    assert_eq!(format(101.0, "0o"), "101st");
    assert_eq!(format(112.0, "0o"), "112th");
}

#[test]
fn signs_parens_and_time() {
    assert_eq!(format(42.0, "+0,0"), "+42");
    assert_eq!(format(-42.0, "+0,0"), "-42");
    assert_eq!(format(-1000.0, "(0,0)"), "(1,000)");
    assert_eq!(format(3661.0, "00:00:00"), "1:01:01");
}

#[test]
fn empty_format_defaults_and_nan() {
    assert_eq!(format(1234.0, ""), "1,234");
    assert_eq!(format(f64::NAN, "0,0.00"), "0.00");
}
