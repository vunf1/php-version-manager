/**
 * Error banner component
 */
export const ErrorBanner = ({ error, onDismiss }) => {
  if (!error) return null;

  return (
    <div className="error-banner">
      <span>{error}</span>
      <button onClick={onDismiss}>×</button>
    </div>
  );
};
