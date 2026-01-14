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

    // EOL dates for major.minor versions (security support end dates)
    fn get_eol_date(major: u8, minor: u8) -> Option<String> {
        match (major, minor) {
            (8, 5) => Some("2028-12-31".to_string()), // PHP 8.5 EOL: Dec 31, 2028
            (8, 4) => Some("2028-12-31".to_string()), // PHP 8.4 EOL: Dec 31, 2028
            (8, 3) => Some("2026-11-26".to_string()), // PHP 8.3 EOL: Nov 26, 2026
            (8, 2) => Some("2025-12-08".to_string()), // PHP 8.2 EOL: Dec 8, 2025
            (8, 1) => Some("2024-11-25".to_string()), // PHP 8.1 EOL: Nov 25, 2024
            (8, 0) => Some("2023-11-26".to_string()), // PHP 8.0 EOL: Nov 26, 2023
            (7, 4) => Some("2022-11-28".to_string()), // PHP 7.4 EOL: Nov 28, 2022
            _ => None,
        }
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
        // Try to fetch dynamically from PHP.net
        match self.fetch_versions_from_php_net().await {
            Ok(versions) if !versions.is_empty() => {
                tracing::info!("Successfully fetched {} versions from PHP.net", versions.len());
                return Ok(versions);
            }
            Ok(_) => {
                tracing::warn!("Fetched empty version list from PHP.net, using fallback");
            }
            Err(e) => {
                tracing::warn!("Failed to fetch versions from PHP.net: {}, using fallback", e);
            }
        }
        
        // Fallback to hardcoded list if fetching fails
        tracing::info!("Using fallback version list");
        let versions: Vec<(&str, Option<&str>, Option<&str>)> = vec![
            ("8.4.0", Some("2024-11-21"), Some("2028-12-31")), // PHP 8.4 EOL: Dec 31, 2028
            ("8.3.0", Some("2023-11-23"), Some("2026-11-26")), // PHP 8.3 EOL: Nov 26, 2026
            ("8.2.14", Some("2024-01-18"), Some("2025-12-08")), // PHP 8.2 EOL: Dec 8, 2025
            ("8.2.13", Some("2023-12-21"), Some("2025-12-08")),
            ("8.2.12", Some("2023-11-16"), Some("2025-12-08")),
            ("8.1.27", Some("2023-11-16"), Some("2024-11-25")), // PHP 8.1 EOL: Nov 25, 2024
            ("8.1.26", Some("2023-10-19"), Some("2024-11-25")),
            ("8.1.25", Some("2023-09-14"), Some("2024-11-25")),
            ("8.0.30", Some("2023-03-16"), Some("2023-11-26")), // PHP 8.0 EOL: Nov 26, 2023 (ended)
            ("8.0.29", Some("2023-02-16"), Some("2023-11-26")),
            ("7.4.33", Some("2022-11-03"), Some("2022-11-28")), // PHP 7.4 EOL: Nov 28, 2022 (ended)
        ];

        Ok(versions
            .into_iter()
            .map(|(v, release, eol)| VersionInfo {
                version: v.to_string(),
                release_date: release.map(|s| s.to_string()),
                eol_date: eol.map(|s| s.to_string()),
                download_url: None,
                checksum: None,
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

