/**
 * Installed Versions Tab Component
 */
import { VersionCard } from "../VersionCard";
import { groupVersionsByBase, isTS, isNTS } from "../../utils/versionUtils";

export const InstalledVersionsTab = ({
  installedVersions,
  versionStatuses,
  activeVersion,
  loading,
  loadingStatuses,
  onSwitch,
  onRemove,
  disabled,
}) => {
  if (loading) {
    return <div className="loading">Loading installed versions...</div>;
  }

  if (installedVersions.length === 0) {
    return (
      <div className="empty-state">
        <p>No PHP versions installed.</p>
        <p className="hint">Switch to the "Available" tab to install versions.</p>
      </div>
    );
  }

  const groupedVersions = groupVersionsByBase(installedVersions);

  return (
    <div className="tab-content">
      <h2>Installed PHP Versions</h2>
      <div className="version-grid">
        {Array.from(groupedVersions.entries()).map(([baseVersion, variants]) => {
          const tsVariant = variants.find(v => isTS(v));
          const ntsVariant = variants.find(v => isNTS(v));
          const primaryVariant = tsVariant || ntsVariant || variants[0];
          const status = versionStatuses[baseVersion] || versionStatuses[primaryVariant] || {};
          const isActive = activeVersion === baseVersion || variants.some(v => activeVersion === v);

          return (
            <VersionCard
              key={baseVersion}
              baseVersion={baseVersion}
              variants={variants}
              status={status}
              isActive={isActive}
              activeVersion={activeVersion}
              onSwitch={onSwitch}
              onRemove={onRemove}
              disabled={disabled}
            />
          );
        })}
      </div>
    </div>
  );
};
