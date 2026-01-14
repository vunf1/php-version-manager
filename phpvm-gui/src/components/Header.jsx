/**
 * Application header component
 */
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useState } from "react";

export const Header = ({ currentDir, showSuccess }) => {
  const [copied, setCopied] = useState(false);

  const handleMinimize = async () => {
    try {
      const appWindow = getCurrentWindow();
      await appWindow.minimize();
    } catch (err) {
      console.error("Failed to minimize window:", err);
    }
  };

  const handleMaximize = async () => {
    try {
      const appWindow = getCurrentWindow();
      const isMaximized = await appWindow.isMaximized();
      if (isMaximized) {
        await appWindow.unmaximize();
      } else {
        await appWindow.maximize();
      }
    } catch (err) {
      console.error("Failed to maximize/unmaximize window:", err);
    }
  };

  const handleClose = async () => {
    try {
      const appWindow = getCurrentWindow();
      await appWindow.close();
    } catch (err) {
      console.error("Failed to close window:", err);
    }
  };

  // Format the IDE path - ensure proper path separator for Windows
  const idePath = currentDir 
    ? `${currentDir.replace(/\//g, '\\')}\\php.exe`
    : "";

  const handleCopyPath = async (e) => {
    e.stopPropagation(); // Prevent window dragging when clicking
    if (!idePath) return;
    
    try {
      await navigator.clipboard.writeText(idePath);
      setCopied(true);
      if (showSuccess) {
        showSuccess("Copied to clipboard!");
      }
      // Reset the copied indicator after 2 seconds
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error("Failed to copy to clipboard:", err);
    }
  };

  const CopyIcon = () => (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" style={{ marginLeft: '0.5rem', opacity: 0.6 }}>
      <path d="M16 1H4C2.9 1 2 1.9 2 3V17H4V3H16V1ZM19 5H8C6.9 5 6 5.9 6 7V21C6 22.1 6.9 23 8 23H19C20.1 23 21 22.1 21 21V7C21 5.9 20.1 5 19 5ZM19 21H8V7H19V21Z" fill="currentColor"/>
    </svg>
  );

  const CheckIcon = () => (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" style={{ marginLeft: '0.5rem', color: '#4caf50' }}>
      <path d="M9 16.17L4.83 12L3.41 13.41L9 19L21 7L19.59 5.59L9 16.17Z" fill="currentColor"/>
    </svg>
  );

  return (
    <header className="app-header" data-tauri-drag-region>
      <h1>PHP Version Manager</h1>
      <div className="header-info">
        {idePath && (
          <div className="header-path-item">
            <div 
              className="setting-value clickable-path" 
              onClick={handleCopyPath} 
              title="Click to copy to clipboard"
              style={{ cursor: 'pointer' }}
            >
              {idePath}
              {copied ? <CheckIcon /> : <CopyIcon />}
            </div>
          </div>
        )}
        <div className="window-controls">
          <button
            className="window-control-btn minimize-btn"
            onClick={handleMinimize}
            title="Minimize"
            aria-label="Minimize"
          >
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none" xmlns="http://www.w3.org/2000/svg">
              <path d="M0 6H12" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round"/>
            </svg>
          </button>
          <button
            className="window-control-btn maximize-btn"
            onClick={handleMaximize}
            title="Maximize"
            aria-label="Maximize"
          >
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none" xmlns="http://www.w3.org/2000/svg">
              <path d="M1 1H11V11H1V1Z" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"/>
            </svg>
          </button>
          <button
            className="window-control-btn close-btn"
            onClick={handleClose}
            title="Close"
            aria-label="Close"
          >
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none" xmlns="http://www.w3.org/2000/svg">
              <path d="M1 1L11 11M11 1L1 11" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round"/>
            </svg>
          </button>
        </div>
      </div>
    </header>
  );
};
