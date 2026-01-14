/**
 * Main Application Component
 * Refactored into smaller, organized components
 */
import { useState } from "react";
import { useTauriReady } from "./hooks/useTauriReady";
import { usePhpVersions } from "./hooks/usePhpVersions";
import { useVersionOperations } from "./hooks/useVersionOperations";
import { useNotifications } from "./hooks/useNotifications";
import { Header } from "./components/Header";
import { Tabs } from "./components/Tabs";
import { NotificationContainer } from "./components/NotificationContainer";
import { InstalledVersionsTab } from "./components/tabs/InstalledVersionsTab";
import { AvailableVersionsTab } from "./components/tabs/AvailableVersionsTab";
import { CacheTab } from "./components/tabs/CacheTab";
import { SettingsTab } from "./components/tabs/SettingsTab";
import { ThreadSafeModal } from "./components/modals/ThreadSafeModal";
import { VariantSelectModal } from "./components/modals/VariantSelectModal";
import { DeleteConfirmModal } from "./components/modals/DeleteConfirmModal";
import { ProgressModal } from "./components/modals/ProgressModal";
import { phpvmApi } from "./services/phpvmApi";
import "./styles/index.css";

function App() {
  const [activeTab, setActiveTab] = useState("installed");
  const [installVersion, setInstallVersion] = useState("");

  // Notification system
  const {
    notifications,
    showSuccess,
    showError,
    showWarning,
    showInfo,
    dismissNotification,
  } = useNotifications();

  // Custom hooks for data management
  const {
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
  } = usePhpVersions();

  // Version operations hook
  const {
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
    showVariantSelectModal,
    pendingSwitchVersion,
    availableVariants,
    handleSwitchClick,
    handleConfirmSwitch,
    handleCancelSwitch,
    isSwitching,
    switchingVersion,
    switchProgress,
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
  } = useVersionOperations({
    installedVersions,
    loadData,
    refreshInstalledData,
    setError,
    showNotification: showInfo,
    showSuccess,
    showError,
    showWarning,
  });

  // Wait for Tauri to be ready
  useTauriReady(loadData);

  // Handle PATH setting
  const handleSetPath = async () => {
    try {
      setError(null);
      await phpvmApi.setPath();
      await loadData();
      showSuccess("PATH updated successfully");
    } catch (err) {
      const errorMsg = err.toString();
      setError(errorMsg);
      showError(errorMsg);
    }
  };

  // Check if variants are installed for thread safe modal
  const tsInstalled = pendingInstallVersion && installedVersions.some(
    v => v.startsWith(`${pendingInstallVersion}-ts`) || v === `${pendingInstallVersion}-ts`
  );
  const ntsInstalled = pendingInstallVersion && installedVersions.some(
    v => v.startsWith(`${pendingInstallVersion}-nts`) || v === `${pendingInstallVersion}-nts`
  );

  // Determine base version for variant select modal
  const variantBaseVersion = availableVariants.length > 0
    ? (availableVariants[0].includes('-') 
        ? availableVariants[0].split('-')[0]
        : availableVariants[0].substring(0, availableVariants[0].lastIndexOf('.')))
    : pendingSwitchVersion;

  return (
    <div className="app">
      <Header currentDir={pathStatus.current_path} showSuccess={showSuccess} />

      <NotificationContainer
        notifications={notifications}
        onDismiss={dismissNotification}
      />

      <Tabs
        activeTab={activeTab}
        onTabChange={setActiveTab}
        installedCount={installedVersions.length}
        availableCount={availableVersions.length}
      />

      <main className="content">
        {activeTab === "installed" && (
          <InstalledVersionsTab
            installedVersions={installedVersions}
            versionStatuses={versionStatuses}
            activeVersion={activeVersion}
            loading={loading}
            loadingStatuses={loadingStatuses}
            onSwitch={handleSwitchClick}
            onRemove={handleRemoveClick}
            disabled={isInstalling || isSwitching || isDeleting}
          />
        )}

        {activeTab === "available" && (
          <AvailableVersionsTab
            availableVersions={availableVersions}
            installedVersions={installedVersions}
            versionStatuses={versionStatuses}
            loading={loading}
            loadingStatuses={loadingStatuses}
            installVersion={installVersion}
            setInstallVersion={setInstallVersion}
            onInstallClick={handleInstallClick}
            isInstalling={isInstalling}
            showThreadSafeModal={showThreadSafeModal}
          />
        )}

        {activeTab === "cache" && (
          <CacheTab
            showSuccess={showSuccess}
            showError={showError}
          />
        )}

        {activeTab === "settings" && (
          <SettingsTab
            pathStatus={pathStatus}
            installPath={installPath}
            logPath={logPath}
            activeVersion={activeVersion}
            installedVersions={installedVersions}
            availableVersions={availableVersions}
            loading={loading}
            onSetPath={handleSetPath}
            onRefresh={async () => {
              await loadData();
              showInfo("Data refreshed");
            }}
            showSuccess={showSuccess}
          />
        )}
      </main>

      {/* Modals */}
      {showThreadSafeModal && (
        <ThreadSafeModal
          version={pendingInstallVersion}
          selectedThreadSafe={selectedThreadSafe}
          onThreadSafeChange={(value) => {
            setSelectedThreadSafe(value);
            selectedThreadSafeRef.current = value;
          }}
          tsInstalled={tsInstalled}
          ntsInstalled={ntsInstalled}
          onConfirm={handleConfirmInstall}
          onCancel={handleCancelInstall}
          isInstalling={isInstalling}
        />
      )}

      {showVariantSelectModal && (
        <VariantSelectModal
          variants={availableVariants}
          activeVersion={activeVersion}
          baseVersion={variantBaseVersion}
          onSelect={handleConfirmSwitch}
          onCancel={handleCancelSwitch}
          isSwitching={isSwitching}
        />
      )}

      {showDeleteConfirmModal && (
        <DeleteConfirmModal
          baseVersion={pendingDeleteVersion}
          variants={pendingDeleteVariants}
          activeVersion={activeVersion}
          onConfirm={handleConfirmDelete}
          onCancel={handleCancelDelete}
          isDeleting={isDeleting}
        />
      )}

      {isInstalling && (
        <ProgressModal
          type="install"
          version={installingVersion}
          progress={installProgress}
          downloadProgress={downloadProgress}
        />
      )}

      {isSwitching && (
        <ProgressModal
          type="switch"
          version={switchingVersion}
          progress={switchProgress}
        />
      )}

      {isDeleting && (
        <ProgressModal
          type="delete"
          version={deletingVersion}
          progress={deleteProgress}
        />
      )}
    </div>
  );
}

export default App;
