/**
 * Integration tests for PhpVersion
 * Tests the public API of the version module
 */
use phpvm_core::PhpVersion;

#[test]
fn test_version_parsing_public_api() {
    let v = PhpVersion::from_string("8.2.0").unwrap();
    assert_eq!(v.major, 8);
    assert_eq!(v.minor, 2);
    assert_eq!(v.patch, 0);
    assert_eq!(v.suffix, None);
}

#[test]
fn test_version_with_suffix_public_api() {
    let v = PhpVersion::from_string("8.2.0-rc1").unwrap();
    assert_eq!(v.suffix, Some("rc1".to_string()));
}

#[test]
fn test_version_to_string_public_api() {
    let v = PhpVersion::new(8, 2, 0);
    assert_eq!(v.to_string(), "8.2.0");
}

#[test]
fn test_version_directory_name_public_api() {
    let v = PhpVersion::new(8, 2, 0);
    assert_eq!(v.directory_name(), "php-8.2.0");
}

#[test]
fn test_version_comparison_public_api() {
    let v1 = PhpVersion::new(8, 2, 0);
    let v2 = PhpVersion::new(8, 3, 0);
    assert!(v1 < v2);
}

#[test]
fn test_version_parsing_invalid_public_api() {
    assert!(PhpVersion::from_string("8.2").is_err());
    assert!(PhpVersion::from_string("invalid").is_err());
}
