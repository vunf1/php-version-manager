/**
 * Integration tests for Provider
 * Tests the public API of the provider module
 */
use phpvm_core::provider::Provider;

#[test]
fn test_provider_new_public_api() {
    let provider = Provider::new().unwrap();
    // Provider should be created successfully
    assert!(true); // Just verify it doesn't panic
}

#[test]
fn test_generate_download_url_public_api() {
    // Test newer versions (>= 7.4)
    let url = Provider::generate_download_url("8.2.0", 8, 2);
    assert!(url.contains("php-8.2.0-Win32-vs16-x64.zip"));
    
    let url = Provider::generate_download_url("8.4.0", 8, 4);
    assert!(url.contains("php-8.4.0-Win32-vs17-x64.zip"));
    
    let url = Provider::generate_download_url("7.4.33", 7, 4);
    assert!(url.contains("php-7.4.33-Win32-vc15-x64.zip"));
    
    // Test older versions (< 7.4) - use archives directory
    let url = Provider::generate_download_url("7.3.33", 7, 3);
    assert!(url.contains("php-7.3.33-Win32-VC15-x64.zip"));
    assert!(url.contains("archives"));
    
    let url = Provider::generate_download_url("5.6.40", 5, 6);
    assert!(url.contains("php-5.6.40-Win32-VC11-x64.zip"));
    assert!(url.contains("archives"));
}

#[test]
fn test_get_eol_date_public_api() {
    // Test known EOL dates
    assert_eq!(Provider::get_eol_date(8, 5), Some("2029-12-31".to_string()));
    assert_eq!(Provider::get_eol_date(8, 4), Some("2028-12-31".to_string()));
    assert_eq!(Provider::get_eol_date(8, 3), Some("2027-12-31".to_string()));
    assert_eq!(Provider::get_eol_date(8, 2), Some("2026-12-31".to_string()));
    assert_eq!(Provider::get_eol_date(8, 1), Some("2025-12-31".to_string()));
    assert_eq!(Provider::get_eol_date(8, 0), Some("2023-11-26".to_string()));
    assert_eq!(Provider::get_eol_date(7, 4), Some("2022-11-28".to_string()));
    assert_eq!(Provider::get_eol_date(7, 3), Some("2021-12-06".to_string()));
    assert_eq!(Provider::get_eol_date(7, 2), Some("2020-11-30".to_string()));
    assert_eq!(Provider::get_eol_date(7, 1), Some("2019-12-01".to_string()));
    assert_eq!(Provider::get_eol_date(7, 0), Some("2019-01-10".to_string()));
    assert_eq!(Provider::get_eol_date(5, 6), Some("2018-12-31".to_string()));
    
    // Test unknown versions
    assert_eq!(Provider::get_eol_date(9, 0), None);
    assert_eq!(Provider::get_eol_date(6, 0), None);
}
