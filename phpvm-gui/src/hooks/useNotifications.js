/**
 * Custom hook for managing notifications
 */
import { useState, useCallback } from "react";

export const useNotifications = () => {
  const [notifications, setNotifications] = useState([]);

  const dismissNotification = useCallback((id) => {
    setNotifications((prev) => prev.filter((n) => n.id !== id));
  }, []);

  const showNotification = useCallback((message, type = "info", duration = null, actions = null) => {
    const id = Date.now() + Math.random();
    
    // Errors and warnings stay until manually dismissed
    // Success and info auto-dismiss after 3 seconds
    let finalDuration = duration;
    if (duration === null || duration === undefined) {
      if (type === "error" || type === "warning") {
        finalDuration = 0; // No auto-dismiss
      } else {
        finalDuration = 3000; // Auto-dismiss after 3 seconds
      }
    }
    
    // If notification has actions, don't auto-dismiss
    if (actions && actions.length > 0) {
      finalDuration = 0;
    }
    
    const notification = {
      id,
      message,
      type, // 'success', 'error', 'warning', 'info'
      duration: finalDuration,
      actions, // Array of { label, onClick } objects
    };

    setNotifications((prev) => [...prev, notification]);

    // Auto-dismiss after duration (only if duration > 0)
    if (finalDuration > 0) {
      setTimeout(() => {
        dismissNotification(id);
      }, finalDuration);
    }

    return id;
  }, [dismissNotification]);

  const showSuccess = useCallback((message, duration, actions) => {
    return showNotification(message, "success", duration, actions);
  }, [showNotification]);

  const showError = useCallback((message, duration, actions) => {
    // Errors always stay until manually dismissed unless explicitly overridden
    return showNotification(message, "error", duration === undefined ? 0 : duration, actions);
  }, [showNotification]);

  const showWarning = useCallback((message, duration, actions) => {
    // Warnings always stay until manually dismissed unless explicitly overridden
    return showNotification(message, "warning", duration === undefined ? 0 : duration, actions);
  }, [showNotification]);

  const showInfo = useCallback((message, duration, actions) => {
    return showNotification(message, "info", duration, actions);
  }, [showNotification]);

  return {
    notifications,
    showNotification,
    showSuccess,
    showError,
    showWarning,
    showInfo,
    dismissNotification,
  };
};
