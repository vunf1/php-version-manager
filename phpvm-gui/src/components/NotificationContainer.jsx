/**
 * Notification container component
 * Displays all active notifications
 */
import { Notification } from "./Notification";

export const NotificationContainer = ({ notifications, onDismiss }) => {
  return (
    <div className="notification-container">
      {notifications.map((notification) => (
        <Notification
          key={notification.id}
          id={notification.id}
          message={notification.message}
          type={notification.type}
          actions={notification.actions}
          onDismiss={onDismiss}
        />
      ))}
    </div>
  );
};
