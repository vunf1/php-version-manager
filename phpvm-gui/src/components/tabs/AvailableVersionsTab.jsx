/**
 * Available Versions Tab Component
 */
import { formatDate } from "../../utils/dateUtils";

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
  return (
    <div className="tab-content">
      <h2>Available PHP Versions (Top 10 Latest)</h2>
      <div className="install-controls">
        <input
          type="text"
          value={installVersion}
          onChange={(e) => setInstallVersion(e.target.value)}
          placeholder="Enter version (e.g., 8.2.0)"
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
      {!loading && availableVersions.length > 0 && (
        <div className="version-grid">
          {availableVersions.map((version) => {
            const status = versionStatuses[version] || {};
            const tsInstalled = installedVersions.some(v => v.startsWith(`${version}-ts`) || v === `${version}-ts`);
            const ntsInstalled = installedVersions.some(v => v.startsWith(`${version}-nts`) || v === `${version}-nts`);
            const bothInstalled = tsInstalled && ntsInstalled;
            const isInstalled = tsInstalled || ntsInstalled;

            return (
              <div key={version} className={`version-card ${isInstalled ? "installed" : ""}`}>
                <div className="version-header">
                  <div className="version-title">
                    <h3>PHP {version}</h3>
                    <div className="badges-row">
                      {status.online && (
                        <span className="badge online-badge">Online</span>
                      )}
                      {!status.online && (
                        <span className="badge offline-badge">Offline</span>
                      )}
                      {tsInstalled && (
                        <span className="badge ts-badge">TS Installed</span>
                      )}
                      {ntsInstalled && (
                        <span className="badge nts-badge">NTS Installed</span>
                      )}
                    </div>
                  </div>
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
                      <span className="detail-value">{formatDate(status.eol_date)}</span>
                    </div>
                  )}
                </div>
                <div className="version-actions">
                  {bothInstalled ? (
                    <span className="text-muted">Both variants installed</span>
                  ) : (
                    <button
                      className={`btn btn-primary ${isInstalled ? (tsInstalled ? 'install-nts-btn' : 'install-ts-btn') : ''}`}
                      onClick={() => onInstallClick(version)}
                      disabled={isInstalling || !status.online || showThreadSafeModal}
                      title={!status.online ? "Version not available online" : isInstalled ? (tsInstalled ? "Install NTS variant" : "Install TS variant") : ""}
                    >
                      {bothInstalled ? "Both installed" : isInstalled ? `Install ${tsInstalled ? "NTS" : "TS"}` : "Install"}
                    </button>
                  )}
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
};
