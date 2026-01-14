/**
 * Tab navigation component
 */
export const Tabs = ({ activeTab, onTabChange, installedCount, availableCount }) => {
  return (
    <nav className="tabs">
      <button
        className={activeTab === "installed" ? "active" : ""}
        onClick={() => onTabChange("installed")}
      >
        Installed ({installedCount})
      </button>
      <button
        className={activeTab === "available" ? "active" : ""}
        onClick={() => onTabChange("available")}
      >
        Available ({availableCount})
      </button>
      <button
        className={activeTab === "cache" ? "active" : ""}
        onClick={() => onTabChange("cache")}
      >
        Cache
      </button>
      <button
        className={activeTab === "settings" ? "active" : ""}
        onClick={() => onTabChange("settings")}
      >
        Settings
      </button>
    </nav>
  );
};
