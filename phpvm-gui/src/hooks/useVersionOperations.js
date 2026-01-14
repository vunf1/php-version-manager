/**
 * Custom hook for managing version operations (install, remove, switch)
 */
import { useState, useRef, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { phpvmApi } from "../services/phpvmApi";

export const useVersionOperations = ({ 
  installedVersions,
  loadData,
  refreshInstalledData,
  showNotification,
  showSuccess,
  showError,
  showWarning,
}) => {
  const [showThreadSafeModal, setShowThreadSafeModal] = useState(false);
  const [pendingInstallVersion, setPendingInstallVersion] = useState(null);
  const [selectedThreadSafe, setSelectedThreadSafe] = useState(true);
  const selectedThreadSafeRef = useRef(true);
  
  const [showVariantSelectModal, setShowVariantSelectModal] = useState(false);
  const [pendingSwitchVersion, setPendingSwitchVersion] = useState(null);
  const [availableVariants, setAvailableVariants] = useState([]);
  
  const [showDeleteConfirmModal, setShowDeleteConfirmModal] = useState(false);
  const [pendingDeleteVersion, setPendingDeleteVersion] = useState(null);
  const [pendingDeleteVariants, setPendingDeleteVariants] = useState([]);
  
  const [isInstalling, setIsInstalling] = useState(false);
  const [installingVersion, setInstallingVersion] = useState(null);
  const [installProgress, setInstallProgress] = useState("");
  
  const [isSwitching, setIsSwitching] = useState(false);
  const [switchingVersion, setSwitchingVersion] = useState(null);
  const [switchProgress, setSwitchProgress] = useState("");
  
  const [isDeleting, setIsDeleting] = useState(false);
  const [deletingVersion, setDeletingVersion] = useState(null);
  const [deleteProgress, setDeleteProgress] = useState("");
  
  // Download progress state
  const [downloadProgress, setDownloadProgress] = useState({
    downloaded: 0,
    total: 0,
    speed: 0,
    percent: 0,
    isCached: false, // Track if file is from cache
  });

  const handleInstallClick = (version) => {
    if (!version) return;
    
    const tsInstalled = installedVersions.some(v => v.startsWith(`${version}-ts`) || v === `${version}-ts`);
    const ntsInstalled = installedVersions.some(v => v.startsWith(`${version}-nts`) || v === `${version}-nts`);
    
    if (tsInstalled && !ntsInstalled) {
      // Only TS installed, install NTS
      handleInstall(version, false);
    } else if (!tsInstalled && ntsInstalled) {
      // Only NTS installed, install TS
      handleInstall(version, true);
    } else if (!tsInstalled && !ntsInstalled) {
      // Neither installed, show modal
      setPendingInstallVersion(version);
      setSelectedThreadSafe(true);
      selectedThreadSafeRef.current = true;
      setShowThreadSafeModal(true);
    } else {
      // Both installed
      if (showWarning) {
        showWarning(`PHP ${version} (both TS and NTS) is already installed.`);
      }
    }
  };

  // Listen for download progress events
  useEffect(() => {
    let unlistenFn = null;
    
    const setupListener = async () => {
      try {
        console.log("[useVersionOperations] Setting up download-progress listener...");
        unlistenFn = await listen("download-progress", (event) => {
          console.log("[useVersionOperations] ✅ RECEIVED download-progress event!");
          console.log("[useVersionOperations] Full event object:", JSON.stringify(event, null, 2));
          console.log("[useVersionOperations] Event payload:", event.payload);
          console.log("[useVersionOperations] Event payload type:", typeof event.payload);
          
          const payload = event.payload;
          
          // Try different ways to access the payload
          let downloaded, total, speed_mbps, percent;
          
          if (typeof payload === 'object' && payload !== null) {
            // Direct property access
            downloaded = payload.downloaded ?? 0;
            total = payload.total ?? 0;
            speed_mbps = payload.speed_mbps ?? 0;
            percent = payload.percent ?? 0;
            
            // If values are still 0, try string keys
            if (downloaded === 0 && total === 0) {
              downloaded = payload['downloaded'] ?? 0;
              total = payload['total'] ?? 0;
              speed_mbps = payload['speed_mbps'] ?? 0;
              percent = payload['percent'] ?? 0;
            }
          } else {
            downloaded = 0;
            total = 0;
            speed_mbps = 0;
            percent = 0;
          }
          
          console.log("[useVersionOperations] Parsed progress:", { downloaded, total, speed_mbps, percent });
          
          // Detect if file is cached: speed is 0, percent is 100, and total > 0
          const isCached = speed_mbps === 0 && percent === 100 && total > 0 && downloaded === total;
          
          // Update progress message immediately when we detect cached file or start downloading
          if (total > 0) {
            if (isCached) {
              setInstallProgress("Using cached PHP archive...");
            } else if (downloaded < total) {
              // Only update to "Downloading" if we're actually downloading (not complete)
              setInstallProgress("Downloading PHP archive...");
            }
          }
          
          // Update state regardless of values (to track progress even if it starts at 0)
          // The condition check was preventing updates when total was 0 initially
          setDownloadProgress({
            downloaded: Number(downloaded) || 0,
            total: Number(total) || 0,
            speed: Number(speed_mbps) || 0,
            percent: Number(percent) || 0,
            isCached: isCached,
          });
          console.log("[useVersionOperations] Updated downloadProgress state:", { downloaded, total, speed_mbps, percent, isCached });
        });
        console.log("[useVersionOperations] Download progress listener set up successfully");
      } catch (err) {
        console.error("[useVersionOperations] Failed to set up download progress listener:", err);
      }
    };
    
    setupListener();
    
    return () => {
      if (unlistenFn) {
        console.log("[useVersionOperations] Cleaning up download progress listener");
        unlistenFn();
      }
    };
  }, [isInstalling]);

  const handleInstall = async (version, threadSafe = null) => {
    if (!version) return;
    try {
      setIsInstalling(true);
      setInstallingVersion(version);
      setInstallProgress("Preparing installation...");
      // Initialize download progress (but don't reset if we already have data)
      setDownloadProgress(prev => prev.total === 0 ? { downloaded: 0, total: 0, speed: 0, percent: 0, isCached: false } : prev);
      
      const wasFirstInstall = installedVersions.length === 0;
      
      let threadSafeParam;
      if (threadSafe === true) {
        threadSafeParam = "ts";
      } else if (threadSafe === false) {
        threadSafeParam = "nts";
      } else {
        threadSafeParam = "ts";
      }
      
      // Set initial message - will be updated when we receive progress event
      setInstallProgress("Preparing download...");
      console.log("[useVersionOperations] Starting installation, waiting for download progress events...");
      // Reset download progress when starting a NEW download
      setDownloadProgress({ downloaded: 0, total: 0, speed: 0, percent: 0, isCached: false });
      
      const params = {
        version: version,
        thread_safe: threadSafeParam
      };
      
      await phpvmApi.installVersion(params);
      
      setInstallProgress("Installing...");
      // Don't reset downloadProgress here - keep the final download stats visible
      // Only reset if we're starting a new download
      // Quick refresh without showing loading state
      await refreshInstalledData();
      
      if (wasFirstInstall) {
        setInstallProgress("Activating version...");
        await phpvmApi.switchVersion(version);
        // Quick refresh without showing loading state
        await refreshInstalledData();
      }
      
      setInstallProgress("Installation complete!");
      if (showSuccess) {
        showSuccess(`PHP ${version} installed successfully`);
      }
      await new Promise(resolve => setTimeout(resolve, 500));
    } catch (err) {
      const errorMsg = err.toString();
      if (showError) {
        showError(`Failed to install PHP ${version}: ${errorMsg}`);
      }
      setInstallProgress("Installation failed");
      throw err;
    } finally {
      setIsInstalling(false);
      setInstallingVersion(null);
      setInstallProgress("");
    }
  };

  const handleConfirmInstall = async () => {
    try {
      const threadSafeValue = selectedThreadSafeRef.current;
      setShowThreadSafeModal(false);
      await handleInstall(pendingInstallVersion, threadSafeValue);
      setPendingInstallVersion(null);
    } catch (err) {
      // Error already handled in handleInstall
      setPendingInstallVersion(null);
    }
  };

  const handleCancelInstall = () => {
    setShowThreadSafeModal(false);
    setPendingInstallVersion(null);
  };

  const handleSwitchClick = (baseVersion, variants) => {
    const tsVariant = variants.find(v => v.endsWith('-ts') || v.endsWith('.ts'));
    const ntsVariant = variants.find(v => v.endsWith('-nts') || v.endsWith('.nts'));
    
    if (tsVariant && ntsVariant) {
      setPendingSwitchVersion(baseVersion);
      setAvailableVariants([tsVariant, ntsVariant]);
      setShowVariantSelectModal(true);
    } else {
      const variantToSwitch = tsVariant || ntsVariant || variants[0];
      handleSwitch(variantToSwitch);
    }
  };

  const handleSwitch = async (version) => {
    try {
      setIsSwitching(true);
      setSwitchingVersion(version);
      setSwitchProgress("Switching PHP version...");
      
      await phpvmApi.switchVersion(version);
      
      setSwitchProgress("Version switched successfully!");
      // Quick refresh without showing loading state
      await refreshInstalledData();
      if (showSuccess) {
        showSuccess(`Switched to PHP ${version}`);
      }
      // Show warning about restarting terminal (2x longer than success notification: 6000ms)
      if (showWarning) {
        showWarning('Please restart your terminal/console for PATH changes to take effect. The "php" command will not work in your current terminal until you restart it.', 6000);
      }
      await new Promise(resolve => setTimeout(resolve, 500));
    } catch (err) {
      const errorMsg = err.toString();
      if (showError) {
        showError(`Failed to switch version: ${errorMsg}`);
      }
      setSwitchProgress("Switch failed");
      throw err;
    } finally {
      setIsSwitching(false);
      setSwitchingVersion(null);
      setSwitchProgress("");
    }
  };

  const handleConfirmSwitch = async (variant) => {
    setShowVariantSelectModal(false);
    await handleSwitch(variant);
    setPendingSwitchVersion(null);
    setAvailableVariants([]);
  };

  const handleCancelSwitch = () => {
    setShowVariantSelectModal(false);
    setPendingSwitchVersion(null);
    setAvailableVariants([]);
  };

  const handleRemoveClick = (baseVersion, variants) => {
    setPendingDeleteVersion(baseVersion);
    setPendingDeleteVariants(variants);
    setShowDeleteConfirmModal(true);
  };

  const handleConfirmDelete = async (variantToDelete) => {
    setShowDeleteConfirmModal(false);
    if (variantToDelete) {
      await handleRemove(variantToDelete);
    }
    setPendingDeleteVersion(null);
    setPendingDeleteVariants([]);
  };

  const handleCancelDelete = () => {
    setShowDeleteConfirmModal(false);
    setPendingDeleteVersion(null);
    setPendingDeleteVariants([]);
  };

  const handleRemove = async (version) => {
    let baseVersion, variant;
    if (version.includes('-ts') || version.includes('-nts')) {
      const parts = version.split('-');
      baseVersion = parts[0];
      variant = parts[1] || '';
      variant = variant === 'ts' ? 'TS' : variant === 'nts' ? 'NTS' : '';
    } else if (version.endsWith('.ts') || version.endsWith('.nts')) {
      const lastDotIndex = version.lastIndexOf('.');
      const beforeLastDot = version.substring(0, lastDotIndex);
      const parts = beforeLastDot.split('.');
      baseVersion = parts.slice(0, 3).join('.');
      variant = version.endsWith('.ts') ? 'TS' : 'NTS';
    } else {
      baseVersion = version.split('.').slice(0, 3).join('.');
      variant = '';
    }
    const displayVersion = baseVersion;
    
    try {
      setIsDeleting(true);
      setDeletingVersion(version);
      setDeleteProgress("Preparing to remove...");
      await new Promise(resolve => setTimeout(resolve, 300));
      
      setDeleteProgress(`Removing PHP ${displayVersion}${variant ? ` (${variant})` : ''}...`);
      await phpvmApi.removeVersion(version);
      
      setDeleteProgress("Cleaning up files...");
      await new Promise(resolve => setTimeout(resolve, 200));
      
      setDeleteProgress("Removal complete!");
      // Quick refresh without showing loading state
      await refreshInstalledData();
      if (showSuccess) {
        showSuccess(`PHP ${displayVersion}${variant ? ` (${variant})` : ''} removed successfully`);
      }
      await new Promise(resolve => setTimeout(resolve, 500));
    } catch (err) {
      const errorMsg = err.toString();
      if (showError) {
        showError(`Failed to remove PHP ${displayVersion}: ${errorMsg}`);
      }
      setDeleteProgress("Removal failed");
      throw err;
    } finally {
      setIsDeleting(false);
      setDeletingVersion(null);
      setDeleteProgress("");
    }
  };

  return {
    // Install
    showThreadSafeModal,
    pendingInstallVersion,
    selectedThreadSafe,
    setSelectedThreadSafe,
    selectedThreadSafeRef,
    handleInstallClick,
    handleConfirmInstall,
    handleCancelInstall,
    isInstalling,
    installingVersion,
    installProgress,
    
    // Switch
    showVariantSelectModal,
    pendingSwitchVersion,
    availableVariants,
    handleSwitchClick,
    handleConfirmSwitch,
    handleCancelSwitch,
    isSwitching,
    switchingVersion,
    switchProgress,
    
    // Remove
    showDeleteConfirmModal,
    pendingDeleteVersion,
    pendingDeleteVariants,
    handleRemoveClick,
    handleConfirmDelete,
    handleCancelDelete,
    isDeleting,
    deletingVersion,
    deleteProgress,
    downloadProgress,
  };
};
