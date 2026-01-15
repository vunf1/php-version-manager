/**
 * Version card component for displaying installed versions
 */
import { parseVersion, isTS, isNTS } from "../utils/versionUtils";
import { formatDate, getEolStatusClass } from "../utils/dateUtils";

export const VersionCard = ({
  baseVersion,
  variants,
  status,
  isActive,
  activeVersion,
  onSwitch,
  onRemove,
  disabled,
}) => {
  const tsVariant = variants.find(v => isTS(v));
  const ntsVariant = variants.find(v => isNTS(v));
  
  // Determine which variant is active
  // activeVersion is stored with variant suffix (e.g., "8.5.1-ts" or "8.5.1-nts")
  let activeVariant = null;
  if (isActive && activeVersion) {
    // Check if activeVersion matches a specific variant
    if (isTS(activeVersion) && tsVariant) {
      activeVariant = 'TS';
    } else if (isNTS(activeVersion) && ntsVariant) {
      activeVariant = 'NTS';
    } else if (activeVersion === baseVersion) {
      // If activeVersion is base version (fallback case), check which variant exists
      // If only one variant exists, that's the active one
      if (tsVariant && !ntsVariant) {
        activeVariant = 'TS';
      } else if (ntsVariant && !tsVariant) {
        activeVariant = 'NTS';
      } else if (tsVariant && ntsVariant) {
        // Both exist - prefer TS as default
        activeVariant = 'TS';
      }
    }
  }

  return (
    <div className={`version-card ${isActive ? "active" : ""}`}>
      <div className="version-main-content">
        <div className="version-header">
          <div className="version-title">
            <div className="version-title-left">
              <h3 className={isActive && activeVariant ? `version-${activeVariant.toLowerCase()}` : ""}>PHP {baseVersion}</h3>
              {isActive && activeVariant && (
                <span className="badge active-badge">
                  <span className="active-indicator"></span>
                  Active {activeVariant}
                </span>
              )}
              {isActive && !activeVariant && (
                <span className="badge active-badge">
                  <span className="active-indicator"></span>
                  Active
                </span>
              )}
            </div>
            <div className="version-actions">
              <button
                className="btn btn-primary"
                onClick={() => onSwitch(baseVersion, variants)}
                disabled={disabled}
                title={tsVariant && ntsVariant ? "Switch between TS and NTS variants" : "Switch to this version"}
              >
                Switch
              </button>
              <button
                className="btn btn-danger"
                onClick={(e) => {
                  e.preventDefault();
                  e.stopPropagation();
                  onRemove(baseVersion, variants);
                }}
                disabled={disabled}
                title="Remove version(s)"
              >
                Remove
              </button>
            </div>
          </div>
          <div className="variants-section">
            <div className="variants-label">Available:</div>
            <div className="variants-list">
              {tsVariant && (
                <div className={`variant-item ${isActive && activeVariant === 'TS' ? 'active' : ''}`}>
                  <span className="variant-badge ts-badge">TS</span>
                </div>
              )}
              {ntsVariant && (
                <div className={`variant-item ${isActive && activeVariant === 'NTS' ? 'active' : ''}`}>
                  <span className="variant-badge nts-badge">NTS</span>
                </div>
              )}
            </div>
          </div>
        </div>
        <div className="version-details">
          {status?.eol_date && (
            <div className="detail-item">
              <span className="detail-label">Security Support EOL:</span>
              <span className={`detail-value ${getEolStatusClass(status.eol_date)}`}>{formatDate(status.eol_date)}</span>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
