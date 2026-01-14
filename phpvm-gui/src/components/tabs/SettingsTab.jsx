/**
 * Settings Tab Component
 */
import { useState } from "react";

export const SettingsTab = ({
  pathStatus,
  installPath,
  logPath,
  activeVersion,
  installedVersions,
  availableVersions,
  loading,
  onSetPath,
  onRefresh,
  showSuccess,
}) => {
  const [copiedPath, setCopiedPath] = useState(null);

  const handleCopyToClipboard = async (text, type) => {
    if (!text || text === "Not set") return;
    
    try {
      await navigator.clipboard.writeText(text);
      setCopiedPath(type);
      if (showSuccess) {
        showSuccess("Copied to clipboard!");
      }
      // Reset the copied indicator after 2 seconds
      setTimeout(() => setCopiedPath(null), 2000);
    } catch (err) {
      console.error("Failed to copy to clipboard:", err);
    }
  };

  const CopyIcon = () => (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" style={{ marginLeft: '0.5rem', opacity: 0.6 }}>
      <path d="M16 1H4C2.9 1 2 1.9 2 3V17H4V3H16V1ZM19 5H8C6.9 5 6 5.9 6 7V21C6 22.1 6.9 23 8 23H19C20.1 23 21 22.1 21 21V7C21 5.9 20.1 5 19 5ZM19 21H8V7H19V21Z" fill="currentColor"/>
    </svg>
  );

  const CheckIcon = () => (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" style={{ marginLeft: '0.5rem', color: '#4caf50' }}>
      <path d="M9 16.17L4.83 12L3.41 13.41L9 19L21 7L19.59 5.59L9 16.17Z" fill="currentColor"/>
    </svg>
  );

  return (
    <div className="tab-content">
      <h2>Settings</h2>
      <div className="settings-section">
        <h3>Environment</h3>
        <div className="setting-item">
          <label>PATH Environment Variable</label>
          <div className="setting-value">
            <div className="path-status-row">
              <span className={pathStatus.is_set ? "status-ok" : "status-error"}>
                {pathStatus.is_set ? "✓ Configured" : "✗ Not Configured"}
              </span>
              {!pathStatus.is_set && (
                <button
                  className="btn btn-primary"
                  onClick={onSetPath}
                  disabled={loading}
                >
                  Set PATH
                </button>
              )}
            </div>
          </div>
          {pathStatus.current_path && (
            <p className="setting-hint">
              Current Path:{" "}
              <span
                className="clickable-path"
                onClick={() => handleCopyToClipboard(pathStatus.current_path, "path")}
                title="Click to copy to clipboard"
              >
                {pathStatus.current_path}
                {copiedPath === "path" ? <CheckIcon /> : <CopyIcon />}
              </span>
              {pathStatus.is_set
                ? " (Added to PATH - restart your terminal/console for changes to take effect)"
                : " (Not in PATH - click 'Set PATH' to add)"}
            </p>
          )}
        </div>
      </div>
      <div className="settings-section">
        <h3>Installation</h3>
        <div className="setting-item">
          <label>Installation Directory</label>
          <div
            className={`setting-value clickable-path ${installPath ? "" : "disabled"}`}
            onClick={() => installPath && handleCopyToClipboard(installPath, "install")}
            title={installPath ? "Click to copy to clipboard" : ""}
            style={{ cursor: installPath ? "pointer" : "default" }}
          >
            {installPath || "Not set"}
            {installPath && (copiedPath === "install" ? <CheckIcon /> : <CopyIcon />)}
          </div>
          <p className="setting-hint">
            PHP versions will be installed in this directory.
          </p>
        </div>
      </div>
      <div className="settings-section">
        <h3>Logging</h3>
        <div className="setting-item">
          <label>Log File Location</label>
          <div
            className={`setting-value clickable-path ${logPath ? "" : "disabled"}`}
            onClick={() => logPath && handleCopyToClipboard(logPath, "log")}
            title={logPath ? "Click to copy to clipboard" : ""}
            style={{ cursor: logPath ? "pointer" : "default" }}
          >
            {logPath || "Not set"}
            {logPath && (copiedPath === "log" ? <CheckIcon /> : <CopyIcon />)}
          </div>
          <p className="setting-hint">
            Application logs are written to this file for debugging purposes.
          </p>
        </div>
      </div>
      <div className="settings-section">
        <h3>Information</h3>
        <div className="setting-item">
          <label>Active Version</label>
          <div className="setting-value">
            {activeVersion ? `PHP ${activeVersion}` : "None"}
          </div>
        </div>
        <div className="setting-item">
          <label>Installed Versions</label>
          <div className="setting-value">{installedVersions.length}</div>
        </div>
        <div className="setting-item">
          <label>Available Versions</label>
          <div className="setting-value">{availableVersions.length}</div>
        </div>
      </div>
      <div className="settings-section">
        <button className="btn btn-secondary" onClick={onRefresh} disabled={loading}>
          Refresh Data
        </button>
      </div>
    </div>
  );
};
