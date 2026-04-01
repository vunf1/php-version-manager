use crate::version::PhpVersion;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use regex::Regex;
use anyhow::Context;

/// Official Windows build manifest (JSON). Preferred over HTML scraping.
const WINDOWS_RELEASES_MANIFEST_URL: &str =
    "https://downloads.php.net/~windows/releases/releases.json";
/// Zip `path` values in the manifest are relative to this directory (php.net layout).
const WINDOWS_RELEASES_ZIP_BASE: &str = "https://windows.php.net/downloads/releases/";
/// Official PHP branch lifecycle (EOL, support phases) as JSON.
const PHP_RELEASE_STATES_URL: &str = "https://www.php.net/releases/states.php?json=1";
/// Latest patch per `major.minor` (matches php.net source releases), for parity when the Windows manifest lags.
const PHP_RELEASE_INDEX_URL: &str = "https://www.php.net/releases/index.php";

/// Branches usually absent from `releases.json` (archives-only Windows builds).
const LEGACY_ARCHIVE_VERSION_LINES: &[&str] = &[
    "5.6.40", "7.0.33", "7.1.33", "7.2.34", "7.3.33",
];

/// Last-resort list when all network sources fail (offline / outages).
const EMERGENCY_FALLBACK_VERSIONS: &[&str] = &["8.3.30", "8.2.30", "7.4.33", "5.6.40"];

#[derive(Debug, Clone, Deserialize)]
struct PhpBranchState {
    #[serde(default)]
    initial_release: Option<String>,
    #[serde(default)]
    security_support_end: Option<String>,
}

/// `index.php?json=1&version=X.Y` — latest full version string for that branch.
#[derive(Debug, Clone, Deserialize)]
struct PhpReleaseIndexJson {
    version: String,
    #[serde(default)]
    date: Option<String>,
}

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
                .user_agent(concat!("phpvm/", env!("CARGO_PKG_VERSION")))
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

    /// Base URL (with trailing slash) for official Windows PHP zip builds.
    /// Archives vs releases matches [`is_archived_version`] (same rule as the installer).
    pub fn official_windows_zip_base_url(major: u8, minor: u8) -> &'static str {
        if Self::is_archived_version(major, minor) {
            "https://windows.php.net/downloads/releases/archives/"
        } else {
            "https://windows.php.net/downloads/releases/"
        }
    }

    /// Official `windows.php.net` zip URL for thread-safe (TS) or NTS builds.
    pub fn build_official_windows_php_zip_url(version: &PhpVersion, thread_safe: bool) -> String {
        let semver = format!("{}.{}.{}", version.major, version.minor, version.patch);
        let base_url = Self::official_windows_zip_base_url(version.major, version.minor);
        let vs_version = Self::get_vs_version(version.major, version.minor);
        if thread_safe {
            format!(
                "{}php-{}-Win32-{}-x64.zip",
                base_url, semver, vs_version
            )
        } else {
            format!(
                "{}php-{}-nts-Win32-{}-x64.zip",
                base_url, semver, vs_version
            )
        }
    }

    /// Generate download URL for a PHP version (thread-safe / TS zip).
    /// Older versions (&lt; 7.4) use archives; VS/VC token from [`get_vs_version`].
    pub fn generate_download_url(version_str: &str, major: u8, minor: u8) -> String {
        let patch_part = version_str.split('.').nth(2).unwrap_or("0");
        let patch_num = patch_part
            .split('-')
            .next()
            .unwrap_or(patch_part)
            .parse::<u8>()
            .unwrap_or(0);
        let v = PhpVersion::new(major, minor, patch_num);
        Self::build_official_windows_php_zip_url(&v, true)
    }

    /// EOL dates for major.minor when [`PHP_RELEASE_STATES_URL`] is unavailable or omits a branch.
    /// Prefer `states.php?json=1` (`security_support_end`) at runtime.
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

    fn trim_iso_datetime_to_date(s: &str) -> String {
        s.split('T').next().unwrap_or(s).to_string()
    }

    fn sort_versions_newest_first(versions: &mut [VersionInfo]) {
        versions.sort_by(|a, b| {
            let va = PhpVersion::from_string(&a.version).unwrap_or_default();
            let vb = PhpVersion::from_string(&b.version).unwrap_or_default();
            vb.cmp(&va)
        });
    }

    /// Merge EOL / initial release from php.net states, then static [`get_eol_date`] fallback.
    fn enrich_versions_with_release_states(
        versions: &mut [VersionInfo],
        states: Option<&HashMap<(u8, u8), PhpBranchState>>,
    ) {
        for v in versions.iter_mut() {
            let Ok(pv) = PhpVersion::from_string(&v.version) else {
                continue;
            };
            if let Some(map) = states {
                if let Some(st) = map.get(&(pv.major, pv.minor)) {
                    if let Some(ref e) = st.security_support_end {
                        v.eol_date = Some(Self::trim_iso_datetime_to_date(e));
                    }
                    // Branch first release only; do not replace patch dates from versionlog/HTML.
                    if v.release_date.is_none() {
                        if let Some(ref e) = st.initial_release {
                            v.release_date = Some(Self::trim_iso_datetime_to_date(e));
                        }
                    }
                }
            }
            if v.eol_date.is_none() {
                v.eol_date = Self::get_eol_date(pv.major, pv.minor);
            }
        }
    }

    fn merge_legacy_archive_versions(versions: &mut Vec<VersionInfo>) {
        let have: HashSet<(u8, u8)> = versions
            .iter()
            .filter_map(|vi| {
                PhpVersion::from_string(&vi.version).ok().map(|p| (p.major, p.minor))
            })
            .collect();

        for &ver in LEGACY_ARCHIVE_VERSION_LINES {
            let Ok(pv) = PhpVersion::from_string(ver) else {
                continue;
            };
            if have.contains(&(pv.major, pv.minor)) {
                continue;
            }
            versions.push(VersionInfo {
                version: ver.to_string(),
                release_date: None,
                eol_date: None,
                download_url: Some(Self::generate_download_url(ver, pv.major, pv.minor)),
                checksum: None,
            });
        }
    }

    fn flatten_php_release_states(
        nested: HashMap<String, HashMap<String, PhpBranchState>>,
    ) -> HashMap<(u8, u8), PhpBranchState> {
        let mut flat = HashMap::new();
        for inner in nested.values() {
            for (xy, st) in inner {
                let mut parts = xy.split('.');
                let (Some(maj_s), Some(min_s)) = (parts.next(), parts.next()) else {
                    continue;
                };
                let Ok(major) = maj_s.parse::<u8>() else {
                    continue;
                };
                let Ok(minor) = min_s.parse::<u8>() else {
                    continue;
                };
                flat.insert((major, minor), st.clone());
            }
        }
        flat
    }

    async fn fetch_php_release_states_map(
        &self,
    ) -> anyhow::Result<HashMap<(u8, u8), PhpBranchState>> {
        let response = self
            .client
            .get(PHP_RELEASE_STATES_URL)
            .send()
            .await
            .with_context(|| format!("Failed to fetch {}", PHP_RELEASE_STATES_URL))?;

        let nested: HashMap<String, HashMap<String, PhpBranchState>> =
            response.json().await.with_context(|| "Invalid states JSON")?;

        Ok(Self::flatten_php_release_states(nested))
    }

    /// Parse [`WINDOWS_RELEASES_MANIFEST_URL`] document: one TS x64 zip URL per branch.
    fn version_infos_from_releases_json_root(root: &serde_json::Value) -> anyhow::Result<Vec<VersionInfo>> {
        let Some(obj) = root.as_object() else {
            anyhow::bail!("releases.json root must be an object");
        };

        let mut out = Vec::new();

        for (branch_key, branch_val) in obj {
            let Some(bobj) = branch_val.as_object() else {
                continue;
            };
            let Some(version_str) = bobj.get("version").and_then(|v| v.as_str()) else {
                tracing::debug!("Branch {} has no version field, skipping", branch_key);
                continue;
            };

            let mut zip_url: Option<String> = None;
            let mut sha256: Option<String> = None;

            for (k, val) in bobj {
                if k.starts_with("ts-") && k.ends_with("-x64") {
                    let Some(z) = val.get("zip") else {
                        continue;
                    };
                    let Some(path) = z.get("path").and_then(|p| p.as_str()) else {
                        continue;
                    };
                    zip_url = Some(format!("{}{}", WINDOWS_RELEASES_ZIP_BASE, path));
                    sha256 = z
                        .get("sha256")
                        .and_then(|s| s.as_str())
                        .map(|s| s.to_lowercase());
                    break;
                }
            }

            let Some(url) = zip_url else {
                tracing::warn!(
                    "No ts-*-x64 zip entry in manifest for branch {}",
                    branch_key
                );
                continue;
            };

            out.push(VersionInfo {
                version: version_str.to_string(),
                release_date: None,
                eol_date: None,
                download_url: Some(url),
                checksum: sha256,
            });
        }

        Self::sort_versions_newest_first(&mut out);
        tracing::info!(
            "Parsed {} branch(es) from Windows releases.json",
            out.len()
        );
        Ok(out)
    }

    /// Latest Windows x64 **TS** zip per branch from the official manifest.
    async fn fetch_versions_from_windows_releases_json(&self) -> anyhow::Result<Vec<VersionInfo>> {
        let response = self
            .client
            .get(WINDOWS_RELEASES_MANIFEST_URL)
            .send()
            .await
            .with_context(|| format!("Failed to fetch {}", WINDOWS_RELEASES_MANIFEST_URL))?;

        let root: serde_json::Value = response
            .json()
            .await
            .with_context(|| "Invalid releases manifest JSON")?;

        Self::version_infos_from_releases_json_root(&root)
    }

    fn emergency_fallback_version_infos() -> Vec<VersionInfo> {
        EMERGENCY_FALLBACK_VERSIONS
            .iter()
            .filter_map(|ver| {
                let pv = PhpVersion::from_string(ver).ok()?;
                Some(VersionInfo {
                    version: (*ver).to_string(),
                    release_date: None,
                    eol_date: Self::get_eol_date(pv.major, pv.minor),
                    download_url: Some(Self::generate_download_url(ver, pv.major, pv.minor)),
                    checksum: None,
                })
            })
            .collect()
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
            
            if major > 0 && patch > 0 {
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

    /// `index.php` dates look like `03 Nov 2022` or `15 Jan 2026` (day, mon, year).
    fn parse_php_net_index_date(date_str: &str) -> Option<String> {
        let month_map: HashMap<&str, &str> = [
            ("jan", "01"),
            ("feb", "02"),
            ("mar", "03"),
            ("apr", "04"),
            ("may", "05"),
            ("jun", "06"),
            ("jul", "07"),
            ("aug", "08"),
            ("sep", "09"),
            ("oct", "10"),
            ("nov", "11"),
            ("dec", "12"),
        ]
        .iter()
        .cloned()
        .collect();

        let parts: Vec<&str> = date_str.split_whitespace().collect();
        if parts.len() != 3 {
            return None;
        }
        let day: u8 = parts[0].trim_end_matches(',').parse().ok()?;
        if !(1..=31).contains(&day) {
            return None;
        }
        let mon = parts[1].to_lowercase();
        let month_num = month_map.get(mon.as_str()).copied()?;
        let year: u16 = parts[2].parse().ok()?;
        if !(1995..=2100).contains(&year) {
            return None;
        }
        Some(format!("{}-{}-{:02}", year, month_num, day))
    }

    async fn fetch_release_index_for_branch(
        &self,
        major: u8,
        minor: u8,
    ) -> Option<PhpReleaseIndexJson> {
        let url = format!(
            "{}?json=1&version={}.{}",
            PHP_RELEASE_INDEX_URL, major, minor
        );
        let response = match self.client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                tracing::debug!("release index request failed for {}: {}", url, e);
                return None;
            }
        };
        if !response.status().is_success() {
            tracing::debug!(
                "release index HTTP {} for {}",
                response.status(),
                url
            );
            return None;
        }
        match response.json::<PhpReleaseIndexJson>().await {
            Ok(j) if !j.version.is_empty() => Some(j),
            Ok(_) => None,
            Err(e) => {
                tracing::debug!("release index JSON decode failed for {}: {}", url, e);
                None
            }
        }
    }

    /// If php.net reports a newer patch than the current row, bump semver, TS zip URL, and clear manifest checksum.
    async fn cross_check_versions_with_release_index(&self, versions: &mut [VersionInfo]) {
        for vi in versions.iter_mut() {
            let Ok(cur) = PhpVersion::from_string(&vi.version) else {
                continue;
            };
            let Some(index) = self
                .fetch_release_index_for_branch(cur.major, cur.minor)
                .await
            else {
                continue;
            };
            let Ok(api_pv) = PhpVersion::from_string(&index.version) else {
                continue;
            };
            if api_pv.major != cur.major || api_pv.minor != cur.minor {
                tracing::debug!(
                    "release index branch mismatch: row {}.{} vs API {}",
                    cur.major,
                    cur.minor,
                    index.version
                );
                continue;
            }
            if api_pv <= cur {
                continue;
            }
            tracing::info!(
                "Release index: branch {}.{} {} -> {} (php.net latest)",
                cur.major,
                cur.minor,
                vi.version,
                index.version
            );
            vi.version = index.version.clone();
            vi.checksum = None;
            vi.download_url = Some(Self::generate_download_url(
                &index.version,
                api_pv.major,
                api_pv.minor,
            ));
            if vi.release_date.is_none() {
                vi.release_date = index
                    .date
                    .as_deref()
                    .and_then(Self::parse_php_net_index_date)
                    .or_else(|| {
                        index
                            .date
                            .as_deref()
                            .and_then(Self::parse_date_to_iso)
                    });
            }
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
        let states = self.fetch_php_release_states_map().await.ok();
        if states.is_none() {
            tracing::warn!(
                "Could not fetch {}; EOL dates will use static fallback where needed",
                PHP_RELEASE_STATES_URL
            );
        }

        match self.fetch_versions_from_windows_releases_json().await {
            Ok(mut versions) if !versions.is_empty() => {
                Self::enrich_versions_with_release_states(&mut versions, states.as_ref());
                Self::merge_legacy_archive_versions(&mut versions);
                self.cross_check_versions_with_release_index(&mut versions)
                    .await;
                Self::enrich_versions_with_release_states(&mut versions, states.as_ref());
                Self::sort_versions_newest_first(&mut versions);
                tracing::info!(
                    "Using {} version(s) from Windows releases.json (after legacy merge)",
                    versions.len()
                );
                return Ok(versions);
            }
            Ok(_) => {
                tracing::warn!("Empty Windows releases.json, trying versionlog.com");
            }
            Err(e) => {
                tracing::warn!(
                    "Windows releases.json failed: {}, trying versionlog.com",
                    e
                );
            }
        }

        match self.fetch_versions_from_versionlog().await {
            Ok(mut versions) if !versions.is_empty() => {
                Self::enrich_versions_with_release_states(&mut versions, states.as_ref());
                Self::merge_legacy_archive_versions(&mut versions);
                self.cross_check_versions_with_release_index(&mut versions)
                    .await;
                Self::enrich_versions_with_release_states(&mut versions, states.as_ref());
                Self::sort_versions_newest_first(&mut versions);
                tracing::info!(
                    "Successfully fetched {} versions from versionlog.com (with enrich/legacy)",
                    versions.len()
                );
                return Ok(versions);
            }
            Ok(_) => {
                tracing::warn!("Fetched empty version list from versionlog.com, trying PHP.net HTML");
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to fetch versions from versionlog.com: {}, trying PHP.net HTML",
                    e
                );
            }
        }

        match self.fetch_versions_from_php_net().await {
            Ok(mut versions) if !versions.is_empty() => {
                Self::enrich_versions_with_release_states(&mut versions, states.as_ref());
                Self::merge_legacy_archive_versions(&mut versions);
                self.cross_check_versions_with_release_index(&mut versions)
                    .await;
                Self::enrich_versions_with_release_states(&mut versions, states.as_ref());
                Self::sort_versions_newest_first(&mut versions);
                tracing::info!(
                    "Successfully fetched {} versions from PHP.net HTML (with enrich/legacy)",
                    versions.len()
                );
                return Ok(versions);
            }
            Ok(_) => {
                tracing::warn!("Fetched empty version list from PHP.net HTML");
            }
            Err(e) => {
                tracing::warn!("Failed to fetch versions from PHP.net HTML: {}", e);
            }
        }

        tracing::info!("Using minimal emergency fallback version list (offline / total failure)");
        let mut versions = Self::emergency_fallback_version_infos();
        self.cross_check_versions_with_release_index(&mut versions).await;
        Self::enrich_versions_with_release_states(&mut versions, states.as_ref());
        Self::sort_versions_newest_first(&mut versions);
        Ok(versions)
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
        
        // Test PHP 8.3.29 specifically - should use vs16 and releases directory
        let url = Provider::generate_download_url("8.3.29", 8, 3);
        assert_eq!(url, "https://windows.php.net/downloads/releases/php-8.3.29-Win32-vs16-x64.zip");
        
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

    #[test]
    fn test_version_infos_from_releases_json_root_ts_x64_zip_url() {
        let root = serde_json::json!({
            "7.4": {
                "version": "7.4.33",
                "ts-vc15-x64": {
                    "zip": {
                        "path": "php-7.4.33-Win32-vc15-x64.zip",
                        "sha256": "AAbbCC"
                    }
                }
            }
        });
        let list = Provider::version_infos_from_releases_json_root(&root).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].version, "7.4.33");
        assert_eq!(
            list[0].download_url.as_deref(),
            Some("https://windows.php.net/downloads/releases/php-7.4.33-Win32-vc15-x64.zip")
        );
        assert_eq!(list[0].checksum.as_deref(), Some("aabbcc"));
    }

    #[test]
    fn test_parse_php_net_index_date_day_mon_year() {
        assert_eq!(
            Provider::parse_php_net_index_date("03 Nov 2022"),
            Some("2022-11-03".to_string())
        );
        assert_eq!(
            Provider::parse_php_net_index_date("10 Jan 2019"),
            Some("2019-01-10".to_string())
        );
        assert_eq!(
            Provider::parse_php_net_index_date("15 Jan 2026"),
            Some("2026-01-15".to_string())
        );
    }

    #[test]
    fn test_flatten_php_release_states_major_minor_keys() {
        let nested: HashMap<String, HashMap<String, PhpBranchState>> =
            serde_json::from_str(r#"{"8":{"8.3":{"initial_release":"2023-11-23T00:00:00+00:00","security_support_end":"2027-12-31T00:00:00+00:00"}}}"#).unwrap();
        let flat = Provider::flatten_php_release_states(nested);
        let st = flat.get(&(8, 3)).expect("8.3 branch");
        assert_eq!(
            st.security_support_end.as_deref(),
            Some("2027-12-31T00:00:00+00:00")
        );
    }
}
