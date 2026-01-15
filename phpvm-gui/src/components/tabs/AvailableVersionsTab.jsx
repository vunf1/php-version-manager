/**
 * Available Versions Tab Component
 */
import { useMemo, useState, useEffect } from "react";
import { formatDate, getEolStatusClass } from "../../utils/dateUtils";
import { phpvmApi } from "../../services/phpvmApi";

export const AvailableVersionsTab = ({
  availableVersions,
  installedVersions,
  versionStatuses,
  loading,
  loadingStatuses,
  installVersion,
  setInstallVersion,
  onInstallClick,
  isInstalling,
  showThreadSafeModal,
}) => {
  const [fetchedVersions, setFetchedVersions] = useState([]);
  const [fetchingVersion, setFetchingVersion] = useState(false);

  // Check if input looks like a version number (e.g., 8.2.0, 8.2, 6, etc.)
  const isValidVersionFormat = (text) => {
    if (!text || text.trim() === '') return false;
    // Match patterns like: 8.2.0, 8.2, 7.4.33, 6, 6., 6.0, etc.
    return /^\d+(\.\d+)?(\.\d+)?(-.*)?$/.test(text.trim());
  };

  // Generate potential version numbers from a partial input (e.g., "6" -> ["6.0.0", "6.1.0", ...])
  const generateVersionCandidates = (partial) => {
    const candidates = [];
    const num = parseInt(partial);
    
    if (isNaN(num)) return candidates;
    
    // If it's just a number, try common minor versions
    if (partial === num.toString()) {
      // Try versions like 6.0.0, 6.1.0, 6.2.0, etc. up to 6.9.0
      for (let minor = 0; minor <= 9; minor++) {
        candidates.push(`${num}.${minor}.0`);
      }
    } else if (partial.includes('.')) {
      const parts = partial.split('.');
      const major = parseInt(parts[0]);
      const minor = parts[1] ? parseInt(parts[1]) : null;
      
      if (!isNaN(major)) {
        if (minor !== null && !isNaN(minor)) {
          // Has major.minor, try patch versions
          for (let patch = 0; patch <= 50; patch++) {
            candidates.push(`${major}.${minor}.${patch}`);
          }
        } else {
          // Just major, try minor versions
          for (let m = 0; m <= 9; m++) {
            candidates.push(`${major}.${m}.0`);
          }
        }
      }
    }
    
    return candidates.slice(0, 20); // Limit to 20 candidates
  };

  // Combine available and fetched versions
  const allVersions = useMemo(() => {
    const combined = [...availableVersions];
    fetchedVersions.forEach(v => {
      if (!combined.includes(v)) {
        combined.push(v);
      }
    });
    return combined;
  }, [availableVersions, fetchedVersions]);

  // Filter available versions based on input text
  const filteredVersions = useMemo(() => {
    if (!installVersion || installVersion.trim() === '') {
      return availableVersions;
    }
    
    const searchText = installVersion.trim().toLowerCase();
    return allVersions.filter(version => 
      version.toLowerCase().includes(searchText)
    );
  }, [allVersions, installVersion, availableVersions]);

  // Clear fetched versions when input is cleared
  useEffect(() => {
    if (!installVersion || installVersion.trim() === '') {
      setFetchedVersions([]);
    }
  }, [installVersion]);

  // Fetch version info when user types a version that's not in the list
  useEffect(() => {
    const searchText = installVersion?.trim() || '';
    
    if (!searchText || !isValidVersionFormat(searchText)) {
      return;
    }

    // Normalize version (remove variant suffix if present, e.g., "8.2.0-ts" -> "8.2.0")
    const baseVersion = searchText.split('-')[0].split('.ts')[0].split('.nts')[0];
    const normalizedSearch = baseVersion.toLowerCase();
    
    // Check if any version in the current filtered list already matches
    // If we have matches, we might still want to fetch more if it's a partial match
    const hasExactMatch = allVersions.some(v => v.toLowerCase() === normalizedSearch);
    
    // If it's a partial match (like "6"), try to fetch more versions
    const isPartialMatch = !baseVersion.includes('.') || baseVersion.split('.').length < 3;
    
    if (hasExactMatch && !isPartialMatch) {
      return; // Exact match found and it's a complete version, no need to fetch
    }

    // Generate version candidates to try fetching
    const candidates = generateVersionCandidates(baseVersion);
    
    // Debounce the fetch
    const timeoutId = setTimeout(async () => {
      setFetchingVersion(true);
      const foundVersions = [];
      
      // Try fetching each candidate version
      for (const candidate of candidates) {
        try {
          const status = await phpvmApi.getVersionStatus(candidate);
          if (status && status.online) {
            const version = status.version;
            // Only add if it matches the search and isn't already in the list
            if (version.toLowerCase().includes(normalizedSearch) &&
                !availableVersions.includes(version) && 
                !fetchedVersions.includes(version)) {
              foundVersions.push(version);
            }
          }
        } catch (err) {
          // Version doesn't exist, continue
        }
      }
      
      if (foundVersions.length > 0) {
        setFetchedVersions(prev => {
          const newVersions = foundVersions.filter(v => !prev.includes(v));
          return [...prev, ...newVersions];
        });
      }
      
      setFetchingVersion(false);
    }, 500); // 500ms debounce

    return () => clearTimeout(timeoutId);
  }, [installVersion, availableVersions, fetchedVersions, allVersions]);

  return (
    <div className="tab-content">
      <div className="install-controls">
        <input
          type="text"
          value={installVersion}
          onChange={(e) => setInstallVersion(e.target.value)}
          placeholder="Search or enter version (e.g., 8.2.0)"
          className="version-input"
          disabled={loading}
        />
        <button
          className="btn btn-primary"
          onClick={() => onInstallClick(installVersion)}
          disabled={isInstalling || !installVersion || showThreadSafeModal}
        >
          Install
        </button>
      </div>
      {loading && (
        <div className="loading">Loading available versions...</div>
      )}
      {!loading && availableVersions.length === 0 && (
        <div className="empty-state">
          <p>No versions available.</p>
        </div>
      )}
      {!loading && availableVersions.length > 0 && filteredVersions.length === 0 && (
        <div className="empty-state">
          {fetchingVersion ? (
            <p>Checking if version "{installVersion}" is available...</p>
          ) : (
            <p>No versions match "{installVersion}".</p>
          )}
        </div>
      )}
      {!loading && filteredVersions.length > 0 && (
        <div className="version-grid">
          {filteredVersions.map((version) => {
            const status = versionStatuses[version] || {};
            const tsInstalled = installedVersions.some(v => v.startsWith(`${version}-ts`) || v === `${version}-ts`);
            const ntsInstalled = installedVersions.some(v => v.startsWith(`${version}-nts`) || v === `${version}-nts`);
            const bothInstalled = tsInstalled && ntsInstalled;
            const isInstalled = tsInstalled || ntsInstalled;

            return (
              <div key={version} className={`version-card ${isInstalled ? "installed" : ""}`}>
                <div className="version-main-content">
                  <div className="version-header">
                    <div className="version-title">
                      <div className="version-title-left">
                        <h3>PHP {version}</h3>
                        {status.online && (
                          <span className="badge online-badge">Online</span>
                        )}
                        {!status.online && (
                          <span className="badge offline-badge">Offline</span>
                        )}
                      </div>
                      <div className="version-actions">
                        {bothInstalled ? (
                          <span className="text-muted">Both installed</span>
                        ) : (
                          <button
                            className={`btn btn-primary ${isInstalled ? (tsInstalled ? 'install-nts-btn' : 'install-ts-btn') : ''}`}
                            onClick={() => onInstallClick(version)}
                            disabled={isInstalling || !status.online || showThreadSafeModal}
                            title={!status.online ? "Version not available online" : isInstalled ? (tsInstalled ? "Install NTS variant" : "Install TS variant") : "Install version"}
                          >
                            {isInstalled ? `Install ${tsInstalled ? "NTS" : "TS"}` : "Install"}
                          </button>
                        )}
                      </div>
                    </div>
                    {(tsInstalled || ntsInstalled) && (
                      <div className="variants-section">
                        <div className="variants-label">Installed:</div>
                        <div className="variants-list">
                          {tsInstalled && (
                            <div className="variant-item">
                              <span className="variant-badge ts-badge">TS</span>
                            </div>
                          )}
                          {ntsInstalled && (
                            <div className="variant-item">
                              <span className="variant-badge nts-badge">NTS</span>
                            </div>
                          )}
                        </div>
                      </div>
                    )}
                  </div>
                  <div className="version-details">
                    {status.release_date && (
                      <div className="detail-item">
                        <span className="detail-label">Release Date:</span>
                        <span className="detail-value">{formatDate(status.release_date)}</span>
                      </div>
                    )}
                    {status.eol_date && (
                      <div className="detail-item">
                        <span className="detail-label">Security Support EOL:</span>
                        <span className={`detail-value ${getEolStatusClass(status.eol_date)}`}>{formatDate(status.eol_date)}</span>
                      </div>
                    )}
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
};
