/**
 * API service layer for Tauri invocations
 * Centralizes all backend communication
 */
import { invoke } from "@tauri-apps/api/core";

export const phpvmApi = {
  /**
   * Get list of installed PHP versions
   */
  listInstalled: async () => {
    console.log("[phpvmApi] Calling list_installed...");
    try {
      const result = await invoke("list_installed");
      console.log("[phpvmApi] list_installed result:", result);
      return result;
    } catch (err) {
      console.error("[phpvmApi] list_installed error:", err);
      throw err;
    }
  },

  /**
   * Get list of available PHP versions
   */
  listAvailable: async () => {
    console.log("[phpvmApi] Calling list_available...");
    try {
      const result = await invoke("list_available");
      console.log("[phpvmApi] list_available result:", result);
      return result;
    } catch (err) {
      console.error("[phpvmApi] list_available error:", err);
      throw err;
    }
  },

  /**
   * Get currently active PHP version
   */
  getActive: async () => {
    console.log("[phpvmApi] Calling get_active...");
    try {
      const result = await invoke("get_active");
      console.log("[phpvmApi] get_active result:", result);
      return result;
    } catch (err) {
      console.error("[phpvmApi] get_active error:", err);
      throw err;
    }
  },

  /**
   * Get installation path
   */
  getInstallPath: async () => {
    console.log("[phpvmApi] Calling get_install_path...");
    try {
      const result = await invoke("get_install_path");
      console.log("[phpvmApi] get_install_path result:", result);
      return result;
    } catch (err) {
      console.error("[phpvmApi] get_install_path error:", err);
      throw err;
    }
  },

  /**
   * Get log path
   */
  getLogPath: async () => {
    console.log("[phpvmApi] Calling get_log_path...");
    try {
      const result = await invoke("get_log_path");
      console.log("[phpvmApi] get_log_path result:", result);
      return result;
    } catch (err) {
      console.error("[phpvmApi] get_log_path error:", err);
      throw err;
    }
  },

  /**
   * Check PATH status
   */
  checkPathStatus: async () => {
    console.log("[phpvmApi] Calling check_path_status...");
    try {
      const result = await invoke("check_path_status");
      console.log("[phpvmApi] check_path_status result:", result);
      return result;
    } catch (err) {
      console.error("[phpvmApi] check_path_status error:", err);
      throw err;
    }
  },

  /**
   * Set PATH
   */
  setPath: async () => {
    return await invoke("set_path");
  },

  /**
   * Get version status (includes EOL date, install path, etc.)
   */
  getVersionStatus: async (version) => {
    return await invoke("get_version_status", { version });
  },

  /**
   * Install a PHP version
   */
  installVersion: async (params) => {
    console.log("[phpvmApi] Calling install_version with params:", params);
    try {
      const result = await invoke("install_version", { params });
      console.log("[phpvmApi] install_version result:", result);
      return result;
    } catch (err) {
      console.error("[phpvmApi] install_version error:", err);
      throw err;
    }
  },

  /**
   * Remove a PHP version
   */
  removeVersion: async (version) => {
    return await invoke("remove_version", { version });
  },

  /**
   * Switch to a PHP version
   */
  switchVersion: async (version) => {
    return await invoke("switch_version", { version });
  },

  /**
   * List all cached files
   */
  listCachedFiles: async () => {
    return await invoke("list_cached_files");
  },

  /**
   * Remove a specific cached file by hash
   */
  removeCachedFile: async (hash) => {
    return await invoke("remove_cached_file", { hash });
  },

  /**
   * Clear all cached files
   */
  clearAllCache: async () => {
    return await invoke("clear_all_cache");
  },

  /**
   * Get current application version
   */
  getAppVersion: async () => {
    return await invoke("get_app_version");
  },

  /**
   * Check for updates
   */
  checkForUpdates: async () => {
    return await invoke("check_for_updates");
  },

  /**
   * Download update
   */
  downloadUpdate: async (downloadUrl) => {
    return await invoke("download_update", { downloadUrl });
  },

  /**
   * Apply update (replace current executable)
   */
  applyUpdate: async (updateFilePath) => {
    return await invoke("apply_update", { updateFilePath });
  },
};
