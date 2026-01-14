/**
 * Version card component for displaying installed versions
 */
import { parseVersion, isTS, isNTS } from "../utils/versionUtils";
import { formatDate } from "../utils/dateUtils";

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

  return (
    <div className={`version-card ${isActive ? "active" : ""}`}>
      <div className="version-header">
        <div className="version-title">
          <h3>PHP {baseVersion}</h3>
          <div className="badges-row">
            {isActive && (
              <span className="badge active-badge">
                <span className="active-indicator"></span>
                Active
              </span>
            )}
            {tsVariant && (
              <span className="badge ts-badge">TS</span>
            )}
            {ntsVariant && (
              <span className="badge nts-badge">NTS</span>
            )}
          </div>
        </div>
      </div>
      <div className="version-details">
        {status?.eol_date && (
          <div className="detail-item">
            <span className="detail-label">Security Support EOL:</span>
            <span className="detail-value">{formatDate(status.eol_date)}</span>
          </div>
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
  );
};
