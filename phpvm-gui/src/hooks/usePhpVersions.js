/**
 * Custom hook for managing PHP version data and operations
 */
import { useState, useEffect, useRef, useCallback } from "react";
import { phpvmApi } from "../services/phpvmApi";

export const usePhpVersions = () => {
  const [installedVersions, setInstalledVersions] = useState([]);
  const [availableVersions, setAvailableVersions] = useState([]);
  const [activeVersion, setActiveVersion] = useState(null);
  const [installPath, setInstallPath] = useState("");
  const [logPath, setLogPath] = useState("");
  const [pathStatus, setPathStatus] = useState({ is_set: false, current_path: "" });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [versionStatuses, setVersionStatuses] = useState({});
  const [loadingStatuses, setLoadingStatuses] = useState(false);
  const isLoadingRef = useRef(false);

  // Lightweight refresh that only updates installed/active versions (no loading state)
  const refreshInstalledData = useCallback(async () => {
    try {
      // Only fetch what changed after switch/remove operations
      const installed = await phpvmApi.listInstalled().catch(err => {
        console.error("[refreshInstalledData] Error in listInstalled:", err);
        return null; // Return null to indicate error, don't update state
      });
      
      const active = await phpvmApi.getActive().catch(err => {
        console.error("[refreshInstalledData] Error in getActive:", err);
        return null;
      });
      
      // Only update if we got valid data
      if (installed !== null) {
        setInstalledVersions(installed || []);
      }
      if (active !== null) {
        setActiveVersion(active || null);
      }
    } catch (err) {
      console.error("[refreshInstalledData] Error refreshing data:", err);
      // Don't set error state for background refreshes
    }
  }, []);

  const loadData = useCallback(async () => {
    // Prevent concurrent calls
    if (isLoadingRef.current) {
      console.log("[loadData] Already loading, skipping duplicate call");
      return;
    }
    
    let loadingSet = false;
    try {
      isLoadingRef.current = true;
      setLoading(true);
      loadingSet = true;
      setError(null);

      // Verify Tauri is available before making calls
      if (!window.__TAURI_INTERNALS__ && !window.__TAURI__) {
        throw new Error("Tauri runtime is not initialized. Please restart the application.");
      }

      console.log("[loadData] Starting to load data...");
      
      // Load data sequentially to avoid potential deadlocks with manager lock
      // Fast operations first, then slower ones
      const path = await phpvmApi.getInstallPath().catch(err => {
        console.error("[loadData] Error in getInstallPath:", err);
        return "";
      });
      
      const logPath = await phpvmApi.getLogPath().catch(err => {
        console.error("[loadData] Error in getLogPath:", err);
        return "";
      });
      
      const pathStatus = await phpvmApi.checkPathStatus().catch(err => {
        console.error("[loadData] Error in checkPathStatus:", err);
        return { is_set: false, current_path: "" };
      });
      
      // Load manager-dependent calls sequentially to avoid lock contention
      // These all need to lock the same Mutex, so sequential is safer
      const installed = await phpvmApi.listInstalled().catch(err => {
        console.error("[loadData] Error in listInstalled:", err);
        return [];
      });
      
      const active = await phpvmApi.getActive().catch(err => {
        console.error("[loadData] Error in getActive:", err);
        return null;
      });
      
      // list_available can be slow (network request), so do it last
      const available = await phpvmApi.listAvailable().catch(err => {
        console.error("[loadData] Error in listAvailable:", err);
        return [];
      });

      console.log("[loadData] Data loaded successfully:", { installed, available, active });
      
      setInstalledVersions(installed || []);
      setAvailableVersions(available || []);
      setActiveVersion(active || null);
      setInstallPath(path || "");
      setLogPath(logPath || "");
      setPathStatus(pathStatus || { is_set: false, current_path: "" });
    } catch (err) {
      console.error("[loadData] Error loading data:", err);
      setError(err?.toString() || "Failed to load data");
      // Set default values on error
      setInstalledVersions([]);
      setAvailableVersions([]);
      setActiveVersion(null);
    } finally {
      isLoadingRef.current = false;
      if (loadingSet) {
        console.log("[loadData] Setting loading to false");
        setLoading(false);
      }
    }
  }, []); // Empty deps - loadData doesn't depend on any state

  const loadVersionStatuses = async (availableVersions, installedVersions, activeVersion) => {
    if (availableVersions.length === 0 && installedVersions.length === 0) return;

    setLoadingStatuses(true);
    try {
      // Load statuses for both available versions and installed variants
      const allVersions = new Set(availableVersions);

      // Add base versions from installed variants (handle both -ts/-nts and .ts/.nts formats)
      installedVersions.forEach(v => {
        let baseVersion;
        if (v.includes('-ts') || v.includes('-nts')) {
          baseVersion = v.split('-')[0]; // "8.3.29-ts" -> "8.3.29"
        } else if (v.endsWith('.ts') || v.endsWith('.nts')) {
          const lastDotIndex = v.lastIndexOf('.');
          const beforeLastDot = v.substring(0, lastDotIndex);
          const parts = beforeLastDot.split('.');
          baseVersion = parts.slice(0, 3).join('.');
        } else {
          baseVersion = v.split('.').slice(0, 3).join('.');
        }
        allVersions.add(baseVersion);
      });

      // Load all statuses in parallel
      const statusPromises = Array.from(allVersions).map(async (version) => {
        try {
          const status = await phpvmApi.getVersionStatus(version);
          return { version, status };
        } catch (err) {
          return {
            version,
            status: {
              version,
              installed: installedVersions?.some(v => v.startsWith(version)) || false,
              active: activeVersion === version || installedVersions?.some(v => v === activeVersion && v.startsWith(version)) || false,
              online: true,
              install_path: null,
              release_date: null,
              eol_date: null,
            },
          };
        }
      });

      // Also load statuses for installed variants directly
      const variantPromises = installedVersions.map(async (variant) => {
        try {
          let baseVersion;
          if (variant.includes('-ts') || variant.includes('-nts')) {
            baseVersion = variant.split('-')[0];
          } else if (variant.endsWith('.ts') || variant.endsWith('.nts')) {
            const lastDotIndex = variant.lastIndexOf('.');
            const beforeLastDot = variant.substring(0, lastDotIndex);
            const parts = beforeLastDot.split('.');
            baseVersion = parts.slice(0, 3).join('.');
          } else {
            baseVersion = variant.split('.').slice(0, 3).join('.');
          }
          const status = await phpvmApi.getVersionStatus(baseVersion);
          return { version: variant, status };
        } catch (err) {
          return null;
        }
      });

      const results = await Promise.all([...statusPromises, ...variantPromises]);
      const statuses = {};
      results.forEach((result) => {
        if (result) {
          statuses[result.version] = result.status;
        }
      });
      setVersionStatuses(statuses);
    } catch (err) {
      console.error("Error loading version statuses:", err);
    } finally {
      setLoadingStatuses(false);
    }
  };

  // Track previous values to avoid unnecessary re-renders
  const prevVersionsRef = useRef({ 
    available: JSON.stringify([]), 
    installed: JSON.stringify([]), 
    active: null 
  });
  
  // Load version statuses when versions change
  useEffect(() => {
    // Only reload if versions actually changed (not just array reference)
    const availableStr = JSON.stringify(availableVersions);
    const installedStr = JSON.stringify(installedVersions);
    const availableChanged = prevVersionsRef.current.available !== availableStr;
    const installedChanged = prevVersionsRef.current.installed !== installedStr;
    const activeChanged = prevVersionsRef.current.active !== activeVersion;
    
    if (!loading && (availableChanged || installedChanged || activeChanged) && 
        (availableVersions.length > 0 || installedVersions.length > 0)) {
      prevVersionsRef.current = {
        available: availableStr,
        installed: installedStr,
        active: activeVersion,
      };
      loadVersionStatuses(availableVersions, installedVersions, activeVersion);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [availableVersions, installedVersions, activeVersion, loading]);

  return {
    installedVersions,
    availableVersions,
    activeVersion,
    installPath,
    logPath,
    pathStatus,
    loading,
    error,
    versionStatuses,
    loadingStatuses,
    loadData,
    refreshInstalledData,
    setError,
  };
};
