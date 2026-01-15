use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PhpVersion {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
    pub suffix: Option<String>,
}

impl PhpVersion {
    pub fn new(major: u8, minor: u8, patch: u8) -> Self {
        PhpVersion {
            major,
            minor,
            patch,
            suffix: None,
        }
    }

    pub fn from_string(s: &str) -> anyhow::Result<Self> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() < 3 {
            anyhow::bail!("Invalid version format: {}", s);
        }

        let major = parts[0].parse()?;
        let minor = parts[1].parse()?;
        let patch_parts: Vec<&str> = parts[2].split('-').collect();
        let patch = patch_parts[0].parse()?;
        let suffix = if patch_parts.len() > 1 {
            Some(patch_parts[1..].join("-"))
        } else {
            None
        };

        Ok(PhpVersion {
            major,
            minor,
            patch,
            suffix,
        })
    }

    pub fn to_string(&self) -> String {
        if let Some(ref suffix) = self.suffix {
            format!("{}.{}.{}-{}", self.major, self.minor, self.patch, suffix)
        } else {
            format!("{}.{}.{}", self.major, self.minor, self.patch)
        }
    }

    pub fn directory_name(&self) -> String {
        format!("php-{}", self.to_string())
    }
}

impl Default for PhpVersion {
    fn default() -> Self {
        PhpVersion {
            major: 0,
            minor: 0,
            patch: 0,
            suffix: None,
        }
    }
}

impl fmt::Display for PhpVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        let v = PhpVersion::from_string("8.2.0").unwrap();
        assert_eq!(v.major, 8);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 0);
        assert_eq!(v.suffix, None);
    }

    #[test]
    fn test_version_with_suffix() {
        let v = PhpVersion::from_string("8.2.0-rc1").unwrap();
        assert_eq!(v.major, 8);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 0);
        assert_eq!(v.suffix, Some("rc1".to_string()));
    }

    #[test]
    fn test_version_with_complex_suffix() {
        let v = PhpVersion::from_string("8.2.0-rc1-alpha").unwrap();
        assert_eq!(v.suffix, Some("rc1-alpha".to_string()));
    }

    #[test]
    fn test_version_new() {
        let v = PhpVersion::new(8, 3, 1);
        assert_eq!(v.major, 8);
        assert_eq!(v.minor, 3);
        assert_eq!(v.patch, 1);
        assert_eq!(v.suffix, None);
    }

    #[test]
    fn test_version_to_string() {
        let v = PhpVersion::new(8, 2, 0);
        assert_eq!(v.to_string(), "8.2.0");
    }

    #[test]
    fn test_version_to_string_with_suffix() {
        let mut v = PhpVersion::new(8, 2, 0);
        v.suffix = Some("rc1".to_string());
        assert_eq!(v.to_string(), "8.2.0-rc1");
    }

    #[test]
    fn test_version_directory_name() {
        let v = PhpVersion::new(8, 2, 0);
        assert_eq!(v.directory_name(), "php-8.2.0");
    }

    #[test]
    fn test_version_directory_name_with_suffix() {
        let mut v = PhpVersion::new(8, 2, 0);
        v.suffix = Some("rc1".to_string());
        assert_eq!(v.directory_name(), "php-8.2.0-rc1");
    }

    #[test]
    fn test_version_display() {
        let v = PhpVersion::new(8, 2, 0);
        assert_eq!(format!("{}", v), "8.2.0");
    }

    #[test]
    fn test_version_default() {
        let v = PhpVersion::default();
        assert_eq!(v.major, 0);
        assert_eq!(v.minor, 0);
        assert_eq!(v.patch, 0);
        assert_eq!(v.suffix, None);
    }

    #[test]
    fn test_version_parsing_invalid_format() {
        assert!(PhpVersion::from_string("8.2").is_err());
        assert!(PhpVersion::from_string("8").is_err());
        assert!(PhpVersion::from_string("invalid").is_err());
    }

    #[test]
    fn test_version_parsing_invalid_numbers() {
        assert!(PhpVersion::from_string("a.b.c").is_err());
        assert!(PhpVersion::from_string("8.b.0").is_err());
    }

    #[test]
    fn test_version_comparison() {
        let v1 = PhpVersion::new(8, 2, 0);
        let v2 = PhpVersion::new(8, 3, 0);
        let v3 = PhpVersion::new(8, 2, 1);
        
        assert!(v1 < v2);
        assert!(v1 < v3);
        assert!(v2 > v1);
        assert!(v3 > v1);
    }

    #[test]
    fn test_version_equality() {
        let v1 = PhpVersion::new(8, 2, 0);
        let v2 = PhpVersion::new(8, 2, 0);
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_version_parsing_various_versions() {
        let test_cases = vec![
            ("7.4.33", (7, 4, 33, None)),
            ("8.0.30", (8, 0, 30, None)),
            ("8.1.27", (8, 1, 27, None)),
            ("8.2.15", (8, 2, 15, None)),
            ("8.3.2", (8, 3, 2, None)),
            ("8.4.0", (8, 4, 0, None)),
        ];

        for (input, (major, minor, patch, suffix)) in test_cases {
            let v = PhpVersion::from_string(input).unwrap();
            assert_eq!(v.major, major);
            assert_eq!(v.minor, minor);
            assert_eq!(v.patch, patch);
            assert_eq!(v.suffix, suffix);
        }
    }
}
