use crate::version::PhpVersion;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use regex::Regex;
use anyhow::Context;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub version: String,
    pub release_date: Option<String>,
    pub eol_date: Option<String>,
    pub download_url: Option<String>,
    pub checksum: Option<String>,
}

pub struct Provider {
    client: reqwest::Client,
}

impl Provider {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Provider {
            client: reqwest::Client::builder()
                .user_agent("phpvm/0.1.0")
                .timeout(std::time::Duration::from_secs(30))
                .build()?,
        })
    }

    /// Determine VS/VC version based on PHP version
    /// PHP 8.4+ uses vs17 (Visual Studio 2017/2019)
    /// PHP 8.0-8.3 uses vs16 (Visual Studio 2016)
    /// PHP 7.4 uses vc15 (Visual C++ 2017) or vs16
    /// PHP 7.2-7.3 uses VC15 (Visual C++ 2017)
    /// PHP 7.0-7.1 uses VC14 (Visual C++ 2015)
    /// PHP 5.6 uses VC11 (Visual C++ 2012)
    pub(crate) fn get_vs_version(major: u8, minor: u8) -> &'static str {
        if major > 8 || (major == 8 && minor >= 4) {
            "vs17"
        } else if major == 8 {
            "vs16"
        } else if major == 7 {
            if minor >= 4 {
                "vc15" // PHP 7.4 uses vc15 (can also be vs16 for some builds)
            } else if minor >= 2 {
                "VC15" // PHP 7.2-7.3 use VC15 (capital VC for archives)
            } else {
                "VC14" // PHP 7.0-7.1 use VC14 (capital VC for archives)
            }
        } else {
            "VC11" // PHP 5.6 and earlier use VC11 (capital VC for archives)
        }
    }

    /// Check if version should use archives URL (versions below 7.4)
    pub(crate) fn is_archived_version(major: u8, minor: u8) -> bool {
        major < 7 || (major == 7 && minor < 4)
    }

    /// Generate download URL for a PHP version
    /// Older versions (< 7.4) are in archives directory and use VC format (capital)
    /// Newer versions (>= 7.4) are in releases directory and use vs format (lowercase)
    pub fn generate_download_url(version_str: &str, major: u8, minor: u8) -> String {
        let vs_version = Self::get_vs_version(major, minor);
        
        if Self::is_archived_version(major, minor) {
            // Older versions: use archives directory and VC format (capital)
            format!(
                "https://windows.php.net/downloads/releases/archives/php-{}-Win32-{}-x64.zip",
                version_str, vs_version
            )
        } else {
            // Newer versions: use releases directory and vs format (lowercase)
            format!(
                "https://windows.php.net/downloads/releases/php-{}-Win32-{}-x64.zip",
                version_str, vs_version
            )
        }
    }

    // EOL dates for major.minor versions (security support end dates)
    // 
    // IMPORTANT: Keep this synchronized with the hardcoded fallback list below!
    // Source: https://versionlog.com/php/
    // Last updated: Dec 18, 2025
    // 
    // When updating, ensure EOL dates match the fallback list in fetch_available_versions()
    pub fn get_eol_date(major: u8, minor: u8) -> Option<String> {
        match (major, minor) {
            (8, 5) => Some("2029-12-31".to_string()), // PHP 8.5 EOL: Dec 31, 2029
            (8, 4) => Some("2028-12-31".to_string()), // PHP 8.4 EOL: Dec 31, 2028
            (8, 3) => Some("2027-12-31".to_string()), // PHP 8.3 EOL: Dec 31, 2027
            (8, 2) => Some("2026-12-31".to_string()), // PHP 8.2 EOL: Dec 31, 2026
            (8, 1) => Some("2025-12-31".to_string()), // PHP 8.1 EOL: Dec 31, 2025 (ended)
            (8, 0) => Some("2023-11-26".to_string()), // PHP 8.0 EOL: Nov 26, 2023 (ended)
            (7, 4) => Some("2022-11-28".to_string()), // PHP 7.4 EOL: Nov 28, 2022 (ended)
            (7, 3) => Some("2021-12-06".to_string()), // PHP 7.3 EOL: Dec 6, 2021 (ended)
            (7, 2) => Some("2020-11-30".to_string()), // PHP 7.2 EOL: Nov 30, 2020 (ended)
            (7, 1) => Some("2019-12-01".to_string()), // PHP 7.1 EOL: Dec 1, 2019 (ended)
            (7, 0) => Some("2019-01-10".to_string()), // PHP 7.0 EOL: Jan 10, 2019 (ended)
            (5, 6) => Some("2018-12-31".to_string()), // PHP 5.6 EOL: Dec 31, 2018 (ended)
            _ => None,
        }
    }

    /// Fetch version information from versionlog.com/php/
    /// Parses the HTML table to extract latest patch versions, release dates, and EOL dates
    async fn fetch_versions_from_versionlog(&self) -> anyhow::Result<Vec<VersionInfo>> {
        let url = "https://versionlog.com/php/";
        tracing::info!("Fetching PHP versions from: {}", url);
        
        let response = self
            .client
            .get(url)
            .send()
            .await
            .with_context(|| format!("Failed to fetch versionlog.com page from {}", url))?;
        
        let html: String = response.text().await.with_context(|| "Failed to read response body")?;
        
        // Strategy: Find all version numbers in format X.Y.Z, group by major.minor, take latest patch
        let version_regex = Regex::new(r#"(\d+)\.(\d+)\.(\d+)"#).unwrap();
        
        // Map to store latest patch version for each major.minor
        let mut latest_patches: HashMap<(u8, u8), (u8, String)> = HashMap::new();
        
        // Find all version numbers in the HTML
        for cap in version_regex.captures_iter(&html) {
            let major: u8 = cap[1].parse().unwrap_or(0);
            let minor: u8 = cap[2].parse().unwrap_or(0);
            let patch: u8 = cap[3].parse().unwrap_or(0);
            
            if major > 0 && minor >= 0 && patch > 0 {
                let version_str = format!("{}.{}.{}", major, minor, patch);
                let key = (major, minor);
                
                // Keep only the latest patch version for each major.minor
                match latest_patches.get(&key) {
                    Some((existing_patch, _)) if patch > *existing_patch => {
                        latest_patches.insert(key, (patch, version_str));
                    }
                    None => {
                        latest_patches.insert(key, (patch, version_str));
                    }
                    _ => {}
                }
            }
        }
        
        // Now try to extract EOL dates from table rows
        // Look for table rows that contain version links and EOL dates
        let mut versions_map: HashMap<String, VersionInfo> = HashMap::new();
        
        for ((major, minor), (_, version_str)) in latest_patches.iter() {
            // Try to find the table row for this major.minor version
            // Pattern: row containing link to /php/X.Y/ followed by EOL date
            let row_pattern = format!(r#"(?s)<tr[^>]*>.*?<a[^>]*href="/php/{}\.{}/"[^>]*>.*?</tr>"#, major, minor);
            let row_regex = match Regex::new(&row_pattern) {
                Ok(regex) => regex,
                Err(_) => continue, // Skip if regex fails
            };
            
            let mut eol_date: Option<String> = None;
            let mut release_date: Option<String> = None;
            
            if let Some(row_cap) = row_regex.captures(&html) {
                let row_content = row_cap.get(0).map(|m| m.as_str()).unwrap_or("");
                
                // Try to extract EOL date - look for "End of security fixes" followed by date
                // Dates can be in format "December 31, 2029" or "Dec 31, 2029"
                let eol_patterns = vec![
                    r#"End of security fixes[^<]*([A-Za-z]+\s+\d{1,2},\s+\d{4})"#,
                    r#"End of security fixes[^<]*(\d{4}-\d{2}-\d{2})"#,
                ];
                
                for pattern in &eol_patterns {
                    if let Ok(eol_regex) = Regex::new(*pattern) {
                        if let Some(eol_cap) = eol_regex.captures(row_content) {
                            if let Some(date_match) = eol_cap.get(1) {
                                let date_str = date_match.as_str().trim();
                                if !date_str.is_empty() {
                                    eol_date = Self::parse_date_to_iso(date_str).or_else(|| {
                                        // Try parsing as ISO date directly
                                        if date_str.len() == 10 && date_str.matches('-').count() == 2 {
                                            Some(date_str.to_string())
                                        } else {
                                            None
                                        }
                                    });
                                    if eol_date.is_some() {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                
                // Try to extract release date from "Latest patch release"
                let release_patterns = vec![
                    r#"Latest patch release[^<]*([A-Za-z]+\s+\d{1,2},\s+\d{4})"#,
                    r#"Latest patch release[^<]*(\d{4}-\d{2}-\d{2})"#,
                ];
                
                for pattern in &release_patterns {
                    if let Ok(release_regex) = Regex::new(*pattern) {
                        if let Some(release_cap) = release_regex.captures(row_content) {
                            if let Some(date_match) = release_cap.get(1) {
                                let date_str = date_match.as_str().trim();
                                if !date_str.is_empty() {
                                    release_date = Self::parse_date_to_iso(date_str).or_else(|| {
                                        if date_str.len() == 10 && date_str.matches('-').count() == 2 {
                                            Some(date_str.to_string())
                                        } else {
                                            None
                                        }
                                    });
                                    if release_date.is_some() {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            // Use EOL from get_eol_date if we couldn't parse it from HTML
            if eol_date.is_none() {
                eol_date = Self::get_eol_date(*major, *minor);
            }
            
            // Generate download URL for this version
            // This ensures versions from versionlog.com are marked as "online" and can be installed
            let download_url = Some(Self::generate_download_url(&version_str, *major, *minor));
            
            versions_map.insert(version_str.clone(), VersionInfo {
                version: version_str.clone(),
                release_date,
                eol_date,
                download_url,
                checksum: None,
            });
        }
        
        let mut versions: Vec<VersionInfo> = versions_map.into_values().collect();
        
        // Sort by version (newest first)
        versions.sort_by(|a, b| {
            let va = PhpVersion::from_string(&a.version).unwrap_or_default();
            let vb = PhpVersion::from_string(&b.version).unwrap_or_default();
            vb.cmp(&va)
        });
        
        tracing::info!("Found {} PHP versions from versionlog.com", versions.len());
        Ok(versions)
    }
    
    /// Parse date string like "December 31, 2029" to ISO format "2029-12-31"
    fn parse_date_to_iso(date_str: &str) -> Option<String> {
        // Simple date parsing for common formats
        // Format: "December 31, 2029" or "Dec 31, 2029"
        let month_map: HashMap<&str, &str> = [
            ("january", "01"), ("february", "02"), ("march", "03"),
            ("april", "04"), ("may", "05"), ("june", "06"),
            ("july", "07"), ("august", "08"), ("september", "09"),
            ("october", "10"), ("november", "11"), ("december", "12"),
            ("jan", "01"), ("feb", "02"), ("mar", "03"),
            ("apr", "04"), ("may", "05"), ("jun", "06"),
            ("jul", "07"), ("aug", "08"), ("sep", "09"),
            ("oct", "10"), ("nov", "11"), ("dec", "12"),
        ].iter().cloned().collect();
        
        let date_lower = date_str.to_lowercase().trim().to_string();
        let parts: Vec<&str> = date_lower.split_whitespace().collect();
        
        if parts.len() >= 3 {
            let month_name = parts[0];
            let day = parts[1].trim_end_matches(',');
            let year = parts[2];
            
            if let Some(month_num) = month_map.get(month_name) {
                if let (Ok(day_num), Ok(year_num)) = (day.parse::<u8>(), year.parse::<u16>()) {
                    if day_num >= 1 && day_num <= 31 && year_num >= 2000 && year_num <= 2100 {
                        return Some(format!("{}-{}-{:02}", year_num, month_num, day_num));
                    }
                }
            }
        }
        
        None
    }

    async fn fetch_versions_from_php_net(&self) -> anyhow::Result<Vec<VersionInfo>> {
        // Fetch the Windows PHP downloads page
        let url = "https://windows.php.net/downloads/releases/";
        tracing::info!("Fetching PHP versions from: {}", url);
        
        let response = self
            .client
            .get(url)
            .send()
            .await
            .with_context(|| format!("Failed to fetch PHP releases page from {}", url))?;
        
        let html: String = response.text().await.with_context(|| "Failed to read response body")?;
        
        // Regex to match PHP version files: 
        // - php-8.4.0-Win32-vs17-x64.zip (PHP 8.4+)
        // - php-8.3.0-Win32-vs16-x64.zip (PHP 8.0-8.3)
        // - php-7.4.33-Win32-vc15-x64.zip (PHP 7.x)
        // Pattern: php-{major}.{minor}.{patch}-Win32-{vs_version|vc_version}-x64
        // Extract both version and VS/VC version from filename
        let version_regex = Regex::new(r"php-(\d+)\.(\d+)\.(\d+)(?:-RC\d+)?-Win32-(vs\d+|vc\d+)-x64(?:-nts)?\.zip").unwrap();
        
        // Collect all unique versions (using HashMap to avoid duplicates)
        let mut versions_set: HashMap<String, VersionInfo> = HashMap::new();
        
        for cap in version_regex.captures_iter(&html) {
            let major: u8 = cap[1].parse().unwrap_or(0);
            let minor: u8 = cap[2].parse().unwrap_or(0);
            let patch: u8 = cap[3].parse().unwrap_or(0);
            let vs_version = cap.get(4).map(|m| m.as_str()).unwrap_or("vc15");
            
            // Skip invalid versions
            if major == 0 || minor == 0 || patch == 0 {
                continue;
            }
            
            // Only process thread-safe builds (non-nts), skip nts files
            // The regex will match both, but we only want the TS version for the main download URL
            let version_str = format!("{}.{}.{}", major, minor, patch);
            
            // Only add if we haven't seen this exact version before
            // Prefer vs17 over vs16 if both exist (newer versions)
            if !versions_set.contains_key(&version_str) {
                let eol_date = Self::get_eol_date(major, minor);
                versions_set.insert(version_str.clone(), VersionInfo {
                    version: version_str.clone(),
                    release_date: None, // We don't have release dates from the page
                    eol_date,
                    download_url: Some(format!("{}php-{}-Win32-{}-x64.zip", url, version_str, vs_version)),
                    checksum: None,
                });
            } else {
                // If version already exists, update if we found a newer VS version (vs17 > vs16)
                if vs_version == "vs17" {
                    if let Some(existing) = versions_set.get_mut(&version_str) {
                        if existing.download_url.as_ref().map(|u| u.contains("vs16")).unwrap_or(false) {
                            existing.download_url = Some(format!("{}php-{}-Win32-{}-x64.zip", url, version_str, vs_version));
                        }
                    }
                }
            }
        }
        
        let mut versions: Vec<VersionInfo> = versions_set.into_values().collect();
        
        // Sort by version (newest first)
        versions.sort_by(|a, b| {
            let va = PhpVersion::from_string(&a.version).unwrap_or_default();
            let vb = PhpVersion::from_string(&b.version).unwrap_or_default();
            vb.cmp(&va)
        });
        
        tracing::info!("Found {} PHP versions from PHP.net", versions.len());
        Ok(versions)
    }

    pub async fn fetch_available_versions(&self) -> anyhow::Result<Vec<VersionInfo>> {
        // Try to fetch dynamically from versionlog.com first (most reliable for EOL dates and latest patches)
        match self.fetch_versions_from_versionlog().await {
            Ok(versions) if !versions.is_empty() => {
                tracing::info!("Successfully fetched {} versions from versionlog.com", versions.len());
                return Ok(versions);
            }
            Ok(_) => {
                tracing::warn!("Fetched empty version list from versionlog.com, trying PHP.net");
            }
            Err(e) => {
                tracing::warn!("Failed to fetch versions from versionlog.com: {}, trying PHP.net", e);
            }
        }
        
        // Fallback to PHP.net if versionlog.com fails
        match self.fetch_versions_from_php_net().await {
            Ok(versions) if !versions.is_empty() => {
                tracing::info!("Successfully fetched {} versions from PHP.net", versions.len());
                return Ok(versions);
            }
            Ok(_) => {
                tracing::warn!("Fetched empty version list from PHP.net, using hardcoded fallback");
            }
            Err(e) => {
                tracing::warn!("Failed to fetch versions from PHP.net: {}, using hardcoded fallback", e);
            }
        }
        
        // Fallback to hardcoded list if fetching fails
        // 
        // IMPORTANT: This list must be kept up-to-date with the latest patch releases!
        // Source: https://versionlog.com/php/
        // Last updated: Dec 18, 2025
        // 
        // Update this list whenever new patch versions are released:
        // 1. Check versionlog.com/php/ for latest patch releases
        // 2. Update the version number, release date, and EOL date for each entry
        // 3. Also update get_eol_date() function to match EOL dates
        // 4. Update this "Last updated" date
        // 
        // Includes latest patch version for each major.minor branch from 5.6.40 onwards
        tracing::info!("Using fallback version list");
        let versions: Vec<(&str, Option<&str>, Option<&str>)> = vec![
            ("8.5.1", Some("2025-11-20"), Some("2029-12-31")), // PHP 8.5 - Latest patch (Nov 20, 2025), EOL: Dec 31, 2029
            ("8.4.16", Some("2025-12-18"), Some("2028-12-31")), // PHP 8.4 - Latest patch (Dec 18, 2025), EOL: Dec 31, 2028
            ("8.3.29", Some("2025-12-18"), Some("2027-12-31")), // PHP 8.3 - Latest patch (Dec 18, 2025), EOL: Dec 31, 2027
            ("8.2.30", Some("2025-12-18"), Some("2026-12-31")), // PHP 8.2 - Latest patch (Dec 18, 2025), EOL: Dec 31, 2026
            ("8.1.34", Some("2025-12-18"), Some("2025-12-31")), // PHP 8.1 - Latest patch (Dec 18, 2025), EOL: Dec 31, 2025 (ended)
            ("8.0.30", Some("2023-08-03"), Some("2023-11-26")), // PHP 8.0 - Latest patch (Aug 3, 2023), EOL: Nov 26, 2023 (ended)
            ("7.4.33", Some("2022-11-03"), Some("2022-11-28")), // PHP 7.4 - Latest patch (Nov 3, 2022), EOL: Nov 28, 2022 (ended)
            ("7.3.33", Some("2021-11-18"), Some("2021-12-06")), // PHP 7.3 - Latest patch (Nov 18, 2021), EOL: Dec 6, 2021 (ended)
            ("7.2.34", Some("2020-10-01"), Some("2020-11-30")), // PHP 7.2 - Latest patch (Oct 1, 2020), EOL: Nov 30, 2020 (ended)
            ("7.1.33", Some("2019-10-24"), Some("2019-12-01")), // PHP 7.1 - Latest patch (Oct 24, 2019), EOL: Dec 1, 2019 (ended)
            ("7.0.33", Some("2019-01-10"), Some("2019-01-10")), // PHP 7.0 - Latest patch (Jan 10, 2019), EOL: Jan 10, 2019 (ended)
            ("5.6.40", Some("2019-01-10"), Some("2018-12-31")), // PHP 5.6 - Latest patch (Jan 10, 2019), EOL: Dec 31, 2018 (ended)
        ];

        Ok(versions
            .into_iter()
            .map(|(v, release, eol)| {
                // Parse version to determine VS version for download URL
                let parts: Vec<&str> = v.split('.').collect();
                let major: u8 = parts.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
                let minor: u8 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                
                // Generate download URL so versions are marked as "online"
                let download_url = if major > 0 && minor >= 0 {
                    Some(Self::generate_download_url(v, major, minor))
                } else {
                    None
                };
                
                VersionInfo {
                    version: v.to_string(),
                    release_date: release.map(|s| s.to_string()),
                    eol_date: eol.map(|s| s.to_string()),
                    download_url,
                    checksum: None,
                }
            })
            .collect())
    }

    pub async fn get_top_versions(&self, limit: usize) -> anyhow::Result<Vec<VersionInfo>> {
        let mut versions = self.fetch_available_versions().await?;
        
        // Sort by version (newest first)
        versions.sort_by(|a, b| {
            let va = PhpVersion::from_string(&a.version).unwrap_or_default();
            let vb = PhpVersion::from_string(&b.version).unwrap_or_default();
            vb.cmp(&va)
        });
        
        versions.truncate(limit);
        Ok(versions)
    }

    pub fn detect_thread_safe_from_url(url: &str) -> Option<bool> {
        // Windows PHP downloads: TS = Thread Safe, NTS = Non-Thread Safe
        // URLs typically contain: php-{version}-Win32-vs16-x64.zip (TS)
        // or: php-{version}-Win32-vs16-x64-nts.zip (NTS)
        if url.contains("-nts") || url.contains("NTS") {
            Some(false)
        } else if url.contains("-ts") || url.contains("TS") {
            Some(true)
        } else {
            // Default: most Windows builds are TS
            Some(true)
        }
    }

    pub fn detect_thread_safe_from_filename(filename: &str) -> Option<bool> {
        // Check filename for TS/NTS indicators
        let lower = filename.to_lowercase();
        if lower.contains("-nts") || lower.contains("_nts") {
            Some(false)
        } else if lower.contains("-ts") || lower.contains("_ts") {
            Some(true)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_vs_version() {
        // Test PHP 8.4+ (vs17)
        assert_eq!(Provider::get_vs_version(8, 4), "vs17");
        assert_eq!(Provider::get_vs_version(8, 5), "vs17");
        assert_eq!(Provider::get_vs_version(9, 0), "vs17");
        
        // Test PHP 8.0-8.3 (vs16)
        assert_eq!(Provider::get_vs_version(8, 0), "vs16");
        assert_eq!(Provider::get_vs_version(8, 1), "vs16");
        assert_eq!(Provider::get_vs_version(8, 2), "vs16");
        assert_eq!(Provider::get_vs_version(8, 3), "vs16");
        
        // Test PHP 7.4 (vc15)
        assert_eq!(Provider::get_vs_version(7, 4), "vc15");
        
        // Test PHP 7.2-7.3 (VC15)
        assert_eq!(Provider::get_vs_version(7, 2), "VC15");
        assert_eq!(Provider::get_vs_version(7, 3), "VC15");
        
        // Test PHP 7.0-7.1 (VC14)
        assert_eq!(Provider::get_vs_version(7, 0), "VC14");
        assert_eq!(Provider::get_vs_version(7, 1), "VC14");
        
        // Test PHP 5.6 and earlier (VC11)
        assert_eq!(Provider::get_vs_version(5, 6), "VC11");
        assert_eq!(Provider::get_vs_version(5, 5), "VC11");
    }

    #[test]
    fn test_is_archived_version() {
        // Versions < 7.4 should be archived
        assert!(Provider::is_archived_version(5, 6));
        assert!(Provider::is_archived_version(7, 0));
        assert!(Provider::is_archived_version(7, 1));
        assert!(Provider::is_archived_version(7, 2));
        assert!(Provider::is_archived_version(7, 3));
        
        // Versions >= 7.4 should not be archived
        assert!(!Provider::is_archived_version(7, 4));
        assert!(!Provider::is_archived_version(8, 0));
        assert!(!Provider::is_archived_version(8, 1));
        assert!(!Provider::is_archived_version(8, 2));
        assert!(!Provider::is_archived_version(8, 3));
        assert!(!Provider::is_archived_version(8, 4));
    }

    #[test]
    fn test_generate_download_url() {
        // Test newer versions (>= 7.4) - use releases directory
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
        
        let url = Provider::generate_download_url("7.0.33", 7, 0);
        assert!(url.contains("php-7.0.33-Win32-VC14-x64.zip"));
        assert!(url.contains("archives"));
        
        let url = Provider::generate_download_url("5.6.40", 5, 6);
        assert!(url.contains("php-5.6.40-Win32-VC11-x64.zip"));
        assert!(url.contains("archives"));
    }

    #[test]
    fn test_get_eol_date() {
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

    #[test]
    fn test_provider_new() {
        let _provider = Provider::new().unwrap();
        // Provider should be created successfully
        assert!(true); // Just verify it doesn't panic
    }
}
