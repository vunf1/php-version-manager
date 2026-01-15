/**
 * Utility functions for parsing and handling PHP version strings
 */

/**
 * Parse a version string to extract base version and variant
 * Handles both formats: "8.3.29-ts" (hyphen) or "8.3.29.ts" (dot)
 */
export const parseVersion = (versionStr) => {
  if (versionStr.includes('-ts') || versionStr.includes('-nts')) {
    // Format: "8.3.29-ts" or "8.3.29-nts"
    const parts = versionStr.split('-');
    const baseVersion = parts[0]; // "8.3.29"
    const variant = parts[1] || ''; // "ts" or "nts"
    return { baseVersion, variant, fullVersion: versionStr };
  } else if (versionStr.endsWith('.ts') || versionStr.endsWith('.nts')) {
    // Format: "8.3.29.ts" or "8.3.29.nts"
    const lastDotIndex = versionStr.lastIndexOf('.');
    const beforeLastDot = versionStr.substring(0, lastDotIndex);
    const variantPart = versionStr.substring(lastDotIndex + 1);
    // Extract base version (first 3 parts: major.minor.patch)
    const parts = beforeLastDot.split('.');
    const baseVersion = parts.slice(0, 3).join('.');
    const variant = variantPart;
    return { baseVersion, variant, fullVersion: versionStr };
  } else {
    // No variant suffix, just base version
    const parts = versionStr.split('.');
    const baseVersion = parts.slice(0, 3).join('.');
    return { baseVersion, variant: '', fullVersion: versionStr };
  }
};

/**
 * Check if a version string is TS (Thread Safe)
 */
export const isTS = (v) => v.endsWith('-ts') || v.endsWith('.ts');

/**
 * Check if a version string is NTS (Non-Thread Safe)
 */
export const isNTS = (v) => v.endsWith('-nts') || v.endsWith('.nts');

/**
 * Group versions by base version
 * Returns a Map where keys are base versions and values are arrays of variant strings
 */
export const groupVersionsByBase = (versions) => {
  const grouped = new Map();
  versions.forEach(v => {
    const { baseVersion } = parseVersion(v);
    if (!grouped.has(baseVersion)) {
      grouped.set(baseVersion, []);
    }
    grouped.get(baseVersion).push(v);
  });
  return grouped;
};

/**
 * Extract base version from a version string (for display)
 */
export const getBaseVersion = (version) => {
  return parseVersion(version).baseVersion;
};

/**
 * Get variant label (TS/NTS) from a version string
 */
export const getVariantLabel = (version) => {
  if (isTS(version)) return 'TS';
  if (isNTS(version)) return 'NTS';
  return '';
};

/**
 * Compare two version strings for sorting
 * Returns: negative if v1 < v2, positive if v1 > v2, 0 if equal
 * Sorts in descending order (newest first)
 */
export const compareVersions = (v1, v2) => {
  const parseVersionParts = (versionStr) => {
    const { baseVersion } = parseVersion(versionStr);
    const parts = baseVersion.split('.').map(Number);
    // Ensure we have at least major.minor.patch
    while (parts.length < 3) {
      parts.push(0);
    }
    return parts;
  };

  const parts1 = parseVersionParts(v1);
  const parts2 = parseVersionParts(v2);

  // Compare major, minor, patch
  for (let i = 0; i < 3; i++) {
    if (parts1[i] !== parts2[i]) {
      // Return negative for descending order (newer first)
      return parts2[i] - parts1[i];
    }
  }

  return 0;
};
