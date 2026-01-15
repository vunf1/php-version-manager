/**
 * Installed Versions Tab Component
 */
import { useMemo } from "react";
import { VersionCard } from "../VersionCard";
import { groupVersionsByBase, isTS, isNTS, compareVersions } from "../../utils/versionUtils";

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
  // Sort versions: active first, then by version (newest first)
  const sortedVersions = useMemo(() => {
    const groupedVersions = groupVersionsByBase(installedVersions);
    const entries = Array.from(groupedVersions.entries());
    
    // Separate active and non-active versions
    const activeEntries = [];
    const nonActiveEntries = [];
    
    entries.forEach(([baseVersion, variants]) => {
      const isActive = activeVersion === baseVersion || variants.some(v => activeVersion === v);
      if (isActive) {
        activeEntries.push([baseVersion, variants]);
      } else {
        nonActiveEntries.push([baseVersion, variants]);
      }
    });
    
    // Sort non-active versions by version number (newest first)
    nonActiveEntries.sort(([v1], [v2]) => compareVersions(v1, v2));
    
    // Combine: active first, then sorted non-active
    return [...activeEntries, ...nonActiveEntries];
  }, [installedVersions, activeVersion]);

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

  return (
    <div className="tab-content">
      <div className="version-grid">
        {sortedVersions.map(([baseVersion, variants]) => {
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
