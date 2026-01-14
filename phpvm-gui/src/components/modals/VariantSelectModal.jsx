/**
 * Modal for selecting which variant to switch to
 */
export const VariantSelectModal = ({
  variants,
  activeVersion,
  baseVersion,
  onSelect,
  onCancel,
  isSwitching,
}) => {
  return (
    <div className="modal-overlay" onClick={onCancel}>
      <div className="modal-content" onClick={(e) => e.stopPropagation()}>
        <h2>Select Variant to Switch</h2>
        <p>Both variants are installed. Choose which one to activate:</p>
        <div className="thread-safe-options">
          {variants.map((variant) => {
            const isTS = variant.endsWith('-ts') || variant.endsWith('.ts');
            const variantBaseVersion = variant.includes('-') 
              ? variant.split('-')[0]
              : variant.substring(0, variant.lastIndexOf('.'));
            const isCurrentlyActive = activeVersion === variant || (activeVersion === baseVersion && variants.length === 1);
            return (
              <label
                key={variant}
                className={`thread-safe-option ${isCurrentlyActive ? 'active-variant' : ''}`}
                onClick={() => onSelect(variant)}
                style={{ cursor: 'pointer' }}
              >
                <div>
                  <strong>PHP {baseVersion} ({isTS ? 'TS' : 'NTS'})</strong>
                  {isCurrentlyActive && <span className="option-badge active-badge">Currently Active</span>}
                  <span className="option-description">
                    {isTS ? 'Thread Safe - Recommended for Apache and IIS with thread-safe modules' : 'Non-Thread Safe - Recommended for Nginx and FastCGI'}
                  </span>
                </div>
              </label>
            );
          })}
        </div>
        <div className="modal-actions">
          <button
            className="btn btn-secondary"
            onClick={onCancel}
            disabled={isSwitching}
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  );
};
