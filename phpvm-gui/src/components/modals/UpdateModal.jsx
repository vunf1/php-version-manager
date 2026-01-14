/**
 * Update Modal Component
 * Handles downloading and applying application updates
 */
import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { phpvmApi } from "../../services/phpvmApi";

export const UpdateModal = ({ updateInfo, onClose, onUpdateComplete, showError, showSuccess }) => {
  const [downloading, setDownloading] = useState(false);
  const [downloaded, setDownloaded] = useState(false);
  const [applying, setApplying] = useState(false);
  const [progress, setProgress] = useState({ downloaded: 0, total: 0, speed_mbps: 0, percent: 0 });
  const [updateFilePath, setUpdateFilePath] = useState(null);

  // Listen for download progress events
  useEffect(() => {
    let unlisten;
    
    const setupProgressListener = async () => {
      try {
        unlisten = await listen("update-download-progress", (event) => {
          const payload = event.payload;
          if (payload) {
            setProgress({
              downloaded: payload.downloaded || 0,
              total: payload.total || 0,
              speed_mbps: payload.speed_mbps || 0,
              percent: payload.percent || 0,
            });
          }
        });
      } catch (err) {
        console.error("Failed to setup progress listener:", err);
      }
    };
    
    setupProgressListener();
    
    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, []);

  const handleDownload = async () => {
    if (!updateInfo.download_url) {
      showError("Download URL not available");
      return;
    }

    setDownloading(true);
    setProgress({ downloaded: 0, total: 0, speed_mbps: 0, percent: 0 });

    try {
      const filePath = await phpvmApi.downloadUpdate(updateInfo.download_url);
      setUpdateFilePath(filePath);
      setDownloading(false);
      setDownloaded(true);
      showSuccess("Update downloaded successfully!");
    } catch (err) {
      const errorMsg = err.toString();
      showError(`Failed to download update: ${errorMsg}`);
      setDownloading(false);
    }
  };

  const handleApply = async () => {
    if (!updateFilePath) {
      showError("Update file not found");
      return;
    }

    setApplying(true);

    try {
      console.log("[UpdateModal] Applying update from:", updateFilePath);
      await phpvmApi.applyUpdate(updateFilePath);
      console.log("[UpdateModal] Update applied successfully, closing app...");
      showSuccess("Update will be applied when you restart the application. The application will now close.");
      
      // Close the app after a short delay to allow user to see the message
      setTimeout(async () => {
        console.log("[UpdateModal] Closing application window...");
        if (onUpdateComplete) {
          onUpdateComplete();
        }
        // Close the window (Tauri API)
        try {
          const appWindow = getCurrentWindow();
          await appWindow.close();
          console.log("[UpdateModal] Window closed successfully");
        } catch (err) {
          console.error("[UpdateModal] Failed to close window:", err);
          // Fallback: try to exit the process
          if (window.__TAURI__) {
            try {
              await window.__TAURI__.process.exit();
            } catch (e) {
              console.error("[UpdateModal] Failed to exit process:", e);
            }
          }
        }
      }, 2000);
    } catch (err) {
      const errorMsg = err.toString();
      console.error("[UpdateModal] Failed to apply update:", err);
      showError(`Failed to apply update: ${errorMsg}`);
      setApplying(false);
    }
  };

  const formatBytes = (bytes) => {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + " " + sizes[i];
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2>Update Available</h2>
          <button className="modal-close" onClick={onClose}>
            ×
          </button>
        </div>
        
        <div className="modal-body">
          <div style={{ marginBottom: "1rem" }}>
            <p>
              A new version of PHP Version Manager is available!
            </p>
            <div style={{ marginTop: "1rem" }}>
              <strong>Current Version:</strong> {updateInfo.current_version}
            </div>
            <div style={{ marginTop: "0.5rem" }}>
              <strong>Latest Version:</strong> {updateInfo.latest_version}
            </div>
            {updateInfo.release_url && (
              <div style={{ marginTop: "0.5rem" }}>
                <a
                  href={updateInfo.release_url}
                  target="_blank"
                  rel="noopener noreferrer"
                  style={{ color: "#2196f3" }}
                >
                  View Release Notes
                </a>
              </div>
            )}
          </div>

          {downloading && (
            <div style={{ marginTop: "1rem" }}>
              <div style={{ display: "flex", justifyContent: "space-between", marginBottom: "0.5rem" }}>
                <span>Downloading update...</span>
                <span>{progress.percent}%</span>
              </div>
              <div className="progress-bar">
                <div
                  className="progress-bar-fill"
                  style={{ width: `${progress.percent}%` }}
                />
              </div>
              <div style={{ marginTop: "0.5rem", fontSize: "0.9rem", color: "#666" }}>
                {formatBytes(progress.downloaded)} / {formatBytes(progress.total)} 
                {progress.speed_mbps > 0 && (
                  <span> • {progress.speed_mbps.toFixed(2)} MB/s</span>
                )}
              </div>
            </div>
          )}

          {downloaded && !applying && (
            <div style={{ marginTop: "1rem", padding: "1rem", backgroundColor: "#e8f5e9", borderRadius: "4px" }}>
              <p style={{ margin: 0, color: "#2e7d32" }}>
                ✓ Update downloaded successfully. Click "Apply Update" to install it.
              </p>
              <p style={{ marginTop: "0.5rem", fontSize: "0.9rem", color: "#666" }}>
                Note: The application will close and restart automatically after applying the update.
              </p>
            </div>
          )}

          {applying && (
            <div style={{ marginTop: "1rem", padding: "1rem", backgroundColor: "#fff3e0", borderRadius: "4px" }}>
              <p style={{ margin: 0, color: "#e65100" }}>
                Applying update... The application will close shortly.
              </p>
            </div>
          )}
        </div>

        <div className="modal-footer">
          {!downloaded && !downloading && (
            <button
              className="btn btn-primary"
              onClick={handleDownload}
              disabled={!updateInfo.download_url}
            >
              Download Update
            </button>
          )}
          {downloaded && !applying && (
            <button
              className="btn btn-primary"
              onClick={handleApply}
            >
              Apply Update
            </button>
          )}
          {!applying && (
            <button
              className="btn btn-secondary"
              onClick={onClose}
              disabled={downloading}
            >
              {downloaded ? "Cancel" : "Later"}
            </button>
          )}
        </div>
      </div>
    </div>
  );
};
