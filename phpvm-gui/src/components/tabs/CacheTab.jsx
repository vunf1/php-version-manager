/**
 * Cache management tab component
 */
import { useState, useEffect } from "react";
import { phpvmApi } from "../../services/phpvmApi";
import { ConfirmModal } from "../modals/ConfirmModal";

export const CacheTab = ({ showSuccess, showError }) => {
  const [cachedFiles, setCachedFiles] = useState([]);
  const [loading, setLoading] = useState(true);
  const [removingHash, setRemovingHash] = useState(null);
  const [clearing, setClearing] = useState(false);
  const [showRemoveConfirm, setShowRemoveConfirm] = useState(false);
  const [pendingRemoveHash, setPendingRemoveHash] = useState(null);
  const [showClearAllConfirm, setShowClearAllConfirm] = useState(false);

  const loadCachedFiles = async () => {
    try {
      setLoading(true);
      const files = await phpvmApi.listCachedFiles();
      // Convert timestamp to readable date
      const filesWithDates = files.map(file => ({
        ...file,
        formattedDate: file.modified && file.modified !== "0" && file.modified !== "Unknown"
          ? new Date(parseInt(file.modified) * 1000).toLocaleString()
          : "Unknown"
      }));
      setCachedFiles(filesWithDates);
    } catch (err) {
      console.error("[CacheTab] Failed to load cached files:", err);
      if (showError) {
        showError(`Failed to load cached files: ${err}`);
      }
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadCachedFiles();
  }, []);

  const handleRemoveFileClick = (hash) => {
    setPendingRemoveHash(hash);
    setShowRemoveConfirm(true);
  };

  const handleRemoveFile = async () => {
    const hash = pendingRemoveHash;
    setShowRemoveConfirm(false);
    
    if (!hash) return;

    try {
      setRemovingHash(hash);
      await phpvmApi.removeCachedFile(hash);
      if (showSuccess) {
        showSuccess("Cached file removed successfully");
      }
      await loadCachedFiles();
    } catch (err) {
      console.error("[CacheTab] Failed to remove cached file:", err);
      if (showError) {
        showError(`Failed to remove cached file: ${err}`);
      }
    } finally {
      setRemovingHash(null);
      setPendingRemoveHash(null);
    }
  };

  const handleClearAllClick = () => {
    setShowClearAllConfirm(true);
  };

  const handleClearAll = async () => {
    setShowClearAllConfirm(false);

    try {
      setClearing(true);
      await phpvmApi.clearAllCache();
      if (showSuccess) {
        showSuccess("All cached files cleared successfully");
      }
      await loadCachedFiles();
    } catch (err) {
      console.error("[CacheTab] Failed to clear cache:", err);
      if (showError) {
        showError(`Failed to clear cache: ${err}`);
      }
    } finally {
      setClearing(false);
    }
  };

  const formatSize = (bytes) => {
    if (!bytes || bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    const sizeIndex = Math.min(i, sizes.length - 1);
    return `${(bytes / Math.pow(k, sizeIndex)).toFixed(2)} ${sizes[sizeIndex]}`;
  };

  if (loading) {
    return (
      <div className="tab-content">
        <h2>Cache Management</h2>
        <p>Loading cached files...</p>
      </div>
    );
  }

  return (
    <div className="tab-content">
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem' }}>
        <h2>Cache Management</h2>
        {cachedFiles.length > 0 && (
          <button
            className="btn btn-danger"
            onClick={handleClearAllClick}
            disabled={clearing}
            style={{ padding: '0.5rem 1rem', fontSize: '0.875rem' }}
          >
            {clearing ? 'Clearing...' : `Clear All (${cachedFiles.length})`}
          </button>
        )}
      </div>

      {cachedFiles.length === 0 ? (
        <div style={{ textAlign: 'center', padding: '2rem', color: '#757575' }}>
          <p>No cached files found.</p>
          <p style={{ fontSize: '0.875rem', marginTop: '0.5rem' }}>
            Cached files are automatically created when downloading PHP versions.
          </p>
        </div>
      ) : (
        <div className="version-list">
          {cachedFiles.map((file) => (
            <div key={file.hash} className="version-card">
              <div style={{ flex: 1 }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '0.5rem', flexWrap: 'wrap' }}>
                  <svg width="20" height="20" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                    <path d="M12 2L2 7L12 12L22 7L12 2Z" stroke="#4caf50" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
                    <path d="M2 17L12 22L22 17" stroke="#4caf50" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
                    <path d="M2 12L12 17L22 12" stroke="#4caf50" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
                  </svg>
                  {file.version ? (
                    <>
                      <strong style={{ fontSize: '0.9375rem' }}>PHP {file.version}</strong>
                      <span style={{ fontFamily: 'monospace', fontSize: '0.75rem', color: '#757575' }}>
                        ({file.hash.substring(0, 8)}...)
                      </span>
                    </>
                  ) : (
                    <strong style={{ fontFamily: 'monospace', fontSize: '0.875rem' }}>{file.hash}</strong>
                  )}
                </div>
                <div style={{ display: 'flex', gap: '1rem', fontSize: '0.875rem', color: '#757575' }}>
                  <span>Size: {formatSize(file.size)}</span>
                  <span>Modified: {file.formattedDate}</span>
                </div>
              </div>
              <button
                className="btn btn-danger"
                onClick={() => handleRemoveFileClick(file.hash)}
                disabled={removingHash === file.hash || clearing}
                style={{ padding: '0.5rem 1rem', fontSize: '0.875rem' }}
              >
                {removingHash === file.hash ? 'Removing...' : 'Remove'}
              </button>
            </div>
          ))}
        </div>
      )}

      {showRemoveConfirm && (
        <ConfirmModal
          title="Remove Cached File"
          message={pendingRemoveHash ? `Are you sure you want to remove this cached file?\n\nHash: ${pendingRemoveHash.substring(0, 16)}...` : "Remove cached file?"}
          confirmText="Remove"
          cancelText="Cancel"
          onConfirm={handleRemoveFile}
          onCancel={() => {
            setShowRemoveConfirm(false);
            setPendingRemoveHash(null);
          }}
          isProcessing={removingHash === pendingRemoveHash}
        />
      )}

      {showClearAllConfirm && (
        <ConfirmModal
          title="Clear All Cache"
          message={`Are you sure you want to clear ALL cached files?\n\nThis will remove ${cachedFiles.length} file(s).`}
          confirmText="Clear All"
          cancelText="Cancel"
          onConfirm={handleClearAll}
          onCancel={() => setShowClearAllConfirm(false)}
          isProcessing={clearing}
        />
      )}
    </div>
  );
};
