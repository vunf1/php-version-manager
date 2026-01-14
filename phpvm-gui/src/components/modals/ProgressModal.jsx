/**
 * Progress modal component for installation, switching, and deletion operations
 */
export const ProgressModal = ({ type, version, progress, title, downloadProgress }) => {
  const getIcon = () => {
    switch (type) {
      case 'install':
        // Show different icon for cached files
        if (downloadProgress?.isCached) {
          return (
            <div className="install-cached-icon">
              <svg width="48" height="48" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                <path d="M12 2L2 7L12 12L22 7L12 2Z" stroke="#4caf50" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
                <path d="M2 17L12 22L22 17" stroke="#4caf50" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
                <path d="M2 12L12 17L22 12" stroke="#4caf50" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
              </svg>
            </div>
          );
        }
        return (
          <div className="install-spinner"></div>
        );
      case 'switch':
        return (
          <div className="switch-icon">
            <svg width="48" height="48" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
              <path d="M7 16L3 12M3 12L7 8M3 12H21M17 8L21 12M21 12L17 16M21 12H3" stroke="#1976d2" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
            </svg>
          </div>
        );
      case 'delete':
        return (
          <div className="delete-icon">
            <svg width="48" height="48" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
              <path d="M3 6H5H21M8 6V4C8 3.46957 8.21071 2.96086 8.58579 2.58579C8.96086 2.21071 9.46957 2 10 2H14C14.5304 2 15.0391 2.21071 15.4142 2.58579C15.7893 2.96086 16 3.46957 16 4V6M19 6V20C19 20.5304 18.7893 21.0391 18.4142 21.4142C18.0391 21.7893 17.5304 22 17 22H7C6.46957 22 5.96086 21.7893 5.58579 21.4142C5.21071 21.0391 5 20.5304 5 20V6H19Z" stroke="#d32f2f" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
              <path d="M10 11V17M14 11V17" stroke="#d32f2f" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
            </svg>
          </div>
        );
      default:
        return null;
    }
  };

  const getProgressBarClass = () => {
    switch (type) {
      case 'switch':
        return 'switch-progress-fill';
      case 'delete':
        return 'delete-progress-fill';
      default:
        return '';
    }
  };

  // Extract base version for display (handle both -ts/-nts and .ts/.nts formats)
  const displayVersion = version ? (
    version.includes('-') 
      ? version.split('-')[0]  // "8.3.29-ts" -> "8.3.29"
      : version.endsWith('.ts') || version.endsWith('.nts')
        ? version.substring(0, version.lastIndexOf('.'))  // "8.3.29.ts" -> "8.3.29"
        : version
  ) : '';

  // Format file size
  const formatSize = (bytes) => {
    if (!bytes || bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    const sizeIndex = Math.min(i, sizes.length - 1);
    return `${(bytes / Math.pow(k, sizeIndex)).toFixed(2)} ${sizes[sizeIndex]}`;
  };

  // Format speed
  const formatSpeed = (mbps) => {
    if (mbps < 0.001) return '0 B/s';
    if (mbps < 1) return `${(mbps * 1024).toFixed(2)} KB/s`;
    return `${mbps.toFixed(2)} MB/s`;
  };

  // Show download progress only while actively downloading
  // Hide it when download is complete (progress text is "Installing..." or later)
  const isDownloading = progress === "Downloading PHP archive..." || progress === "Using cached PHP archive...";
  const showDownloadProgress = type === 'install' && 
    downloadProgress && 
    downloadProgress.total > 0 &&
    isDownloading;

  return (
    <div className="modal-overlay install-modal-overlay">
      <div className="modal-content install-modal-content" onClick={(e) => e.stopPropagation()}>
        <div className="install-modal-header">
          {getIcon()}
          <h2>{title || `${type === 'install' ? 'Installing' : type === 'switch' ? 'Switching to' : 'Removing'} PHP ${displayVersion}`}</h2>
        </div>
        <p className={`install-progress ${downloadProgress?.isCached ? 'install-progress-cached' : ''}`}>{progress}</p>
        {showDownloadProgress ? (
          <div className={`download-progress-info ${downloadProgress?.isCached ? 'download-progress-cached' : ''}`}>
            {downloadProgress?.isCached ? (
              <div className="cached-file-indicator">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                  <path d="M12 2L2 7L12 12L22 7L12 2Z" stroke="#4caf50" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
                  <path d="M2 17L12 22L22 17" stroke="#4caf50" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
                  <path d="M2 12L12 17L22 12" stroke="#4caf50" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
                </svg>
                <span>Using cached file</span>
              </div>
            ) : null}
            <div className="download-progress-stats">
              <span>{formatSize(downloadProgress.downloaded)} / {formatSize(downloadProgress.total)}</span>
              {!downloadProgress?.isCached && <span>{formatSpeed(downloadProgress.speed)}</span>}
              <span>{downloadProgress.percent}%</span>
            </div>
          </div>
        ) : null}
        <div className="install-progress-bar">
          <div 
            className={`install-progress-fill ${getProgressBarClass()}`}
            style={showDownloadProgress ? { 
              width: `${downloadProgress.percent}%`,
              animation: 'none' // Disable animation when showing real progress
            } : {}}
          ></div>
        </div>
        {type === 'switch' && (
          <p className="switch-hint">Note: You may need to restart your terminal for PATH changes to take effect.</p>
        )}
      </div>
    </div>
  );
};
