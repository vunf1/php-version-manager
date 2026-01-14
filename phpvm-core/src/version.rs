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
    }

    #[test]
    fn test_version_with_suffix() {
        let v = PhpVersion::from_string("8.2.0-rc1").unwrap();
        assert_eq!(v.suffix, Some("rc1".to_string()));
    }
}
