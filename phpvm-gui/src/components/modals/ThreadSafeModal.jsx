/**
 * Modal for selecting thread safety option during installation
 */
export const ThreadSafeModal = ({
  version,
  selectedThreadSafe,
  onThreadSafeChange,
  tsInstalled,
  ntsInstalled,
  onConfirm,
  onCancel,
  isInstalling,
}) => {
  return (
    <div className="modal-overlay" onClick={onCancel}>
      <div className="modal-content" onClick={(e) => e.stopPropagation()}>
        <h2>Select Thread Safety</h2>
        <p>Choose the thread safety option for PHP {version}:</p>
        <div className="thread-safe-options">
          <label className={`thread-safe-option ${tsInstalled ? 'disabled' : ''}`}>
            <input
              type="radio"
              name="threadSafe"
              checked={selectedThreadSafe === true}
              onChange={() => onThreadSafeChange(true)}
              disabled={tsInstalled}
            />
            <div>
              <strong>Thread Safe (TS)</strong>
              {tsInstalled && <span className="option-badge installed-badge">Already Installed</span>}
              <span className="option-description">Recommended for Apache and IIS with thread-safe modules</span>
            </div>
          </label>
          <label className={`thread-safe-option ${ntsInstalled ? 'disabled' : ''}`}>
            <input
              type="radio"
              name="threadSafe"
              checked={selectedThreadSafe === false}
              onChange={() => onThreadSafeChange(false)}
              disabled={ntsInstalled}
            />
            <div>
              <strong>Non-Thread Safe (NTS)</strong>
              {ntsInstalled && <span className="option-badge installed-badge">Already Installed</span>}
              <span className="option-description">Recommended for Nginx and FastCGI</span>
            </div>
          </label>
        </div>
        <div className="modal-actions">
          <button
            className="btn btn-secondary"
            onClick={onCancel}
            disabled={isInstalling}
          >
            Cancel
          </button>
          <button
            className="btn btn-primary"
            onClick={onConfirm}
            disabled={isInstalling || (tsInstalled && selectedThreadSafe) || (ntsInstalled && !selectedThreadSafe)}
          >
            Install
          </button>
        </div>
      </div>
    </div>
  );
};
