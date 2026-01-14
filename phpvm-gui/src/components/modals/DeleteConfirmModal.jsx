/**
 * Modal for confirming version deletion
 */
export const DeleteConfirmModal = ({
  baseVersion,
  variants,
  activeVersion,
  onConfirm,
  onCancel,
  isDeleting,
}) => {
  return (
    <div className="modal-overlay" onClick={onCancel}>
      <div className="modal-content" onClick={(e) => e.stopPropagation()}>
        <h2>Remove PHP Version</h2>
        <p>Select which variant(s) to remove for PHP {baseVersion || 'this version'}:</p>
        <div className="thread-safe-options">
          {variants && variants.length > 0 ? (
            variants.map((variant) => {
              const isTS = variant.endsWith('-ts') || variant.endsWith('.ts');
              const variantIsActive = activeVersion === variant || (activeVersion === baseVersion && variants.length === 1);
              return (
                <label
                  key={variant}
                  className="thread-safe-option"
                  onClick={() => onConfirm(variant)}
                  style={{ cursor: 'pointer' }}
                >
                  <div>
                    <strong>PHP {baseVersion} ({isTS ? 'TS' : 'NTS'})</strong>
                    {variantIsActive && <span className="option-badge active-badge">Currently Active (will deactivate first)</span>}
                    <span className="option-description">
                      {isTS ? 'Thread Safe variant' : 'Non-Thread Safe variant'}
                    </span>
                  </div>
                </label>
              );
            })
          ) : (
            <p style={{ padding: '1rem', color: '#757575' }}>No variants available to remove.</p>
          )}
        </div>
        <div className="modal-actions">
          <button
            className="btn btn-secondary"
            onClick={onCancel}
            disabled={isDeleting}
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  );
};
