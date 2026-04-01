/**
 * Integration tests for Provider
 * Tests the public API of the provider module
 */
use phpvm_core::PhpVersion;
use phpvm_core::provider::Provider;

#[test]
fn test_provider_new_public_api() {
    let _provider = Provider::new().unwrap();
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
fn test_build_official_windows_php_zip_url_ts_and_nts() {
    let v84 = PhpVersion::new(8, 4, 0);
    let ts = Provider::build_official_windows_php_zip_url(&v84, true);
    assert_eq!(
        ts,
        "https://windows.php.net/downloads/releases/php-8.4.0-Win32-vs17-x64.zip"
    );
    let nts = Provider::build_official_windows_php_zip_url(&v84, false);
    assert_eq!(
        nts,
        "https://windows.php.net/downloads/releases/php-8.4.0-nts-Win32-vs17-x64.zip"
    );
    assert!(nts.contains("-nts-Win32-"));

    let v73 = PhpVersion::new(7, 3, 33);
    let ts73 = Provider::build_official_windows_php_zip_url(&v73, true);
    assert!(ts73.contains("/archives/"));
    assert!(ts73.contains("php-7.3.33-Win32-VC15-x64.zip"));
    let nts73 = Provider::build_official_windows_php_zip_url(&v73, false);
    assert!(nts73.contains("php-7.3.33-nts-Win32-VC15-x64.zip"));

    let v74 = PhpVersion::new(7, 4, 33);
    let nts74 = Provider::build_official_windows_php_zip_url(&v74, false);
    assert!(nts74.contains("/releases/"));
    assert!(nts74.contains("php-7.4.33-nts-Win32-vc15-x64.zip"));
}

#[test]
fn test_official_windows_zip_base_url_matches_install_rules() {
    assert_eq!(
        Provider::official_windows_zip_base_url(8, 4),
        "https://windows.php.net/downloads/releases/"
    );
    assert_eq!(
        Provider::official_windows_zip_base_url(7, 3),
        "https://windows.php.net/downloads/releases/archives/"
    );
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
