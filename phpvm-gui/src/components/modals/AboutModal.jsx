/**
 * About Modal Component
 * Displays information about the application and developer
 */
import { useState, useEffect } from "react";
import { phpvmApi } from "../../services/phpvmApi";

export const AboutModal = ({ onClose }) => {
  const [appVersion, setAppVersion] = useState("Loading...");

  useEffect(() => {
    const loadVersion = async () => {
      try {
        const version = await phpvmApi.getAppVersion();
        setAppVersion(version);
      } catch (err) {
        console.error("Failed to load app version:", err);
        setAppVersion("Unknown");
      }
    };
    loadVersion();
  }, []);

  const handleLinkClick = async (e, url) => {
    e.preventDefault();
    e.stopPropagation();
    
    try {
      // Use Tauri command to open URL in default browser
      await phpvmApi.openUrl(url);
    } catch (err) {
      console.error('Failed to open URL:', err);
      // Fallback to window.open if Tauri command fails
      try {
        window.open(url, '_blank', 'noopener,noreferrer');
      } catch (fallbackErr) {
        console.error('Fallback also failed:', fallbackErr);
      }
    }
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content about-modal-content" onClick={(e) => e.stopPropagation()}>
        <div className="modal-body about-modal-body">
          <div className="about-app-info">
            <div className="about-app-name">
              <h3>PHP Version Manager</h3>
              <span className="about-version">v{appVersion}</span>
            </div>
            <p className="about-description">
              A professional tool for managing multiple PHP versions on Windows.
            </p>
          </div>

          <div className="about-source-badge">
            <button
              type="button"
              className="about-source-btn"
              title="View source code on GitHub"
              onClick={(e) => handleLinkClick(e, "https://github.com/vunf1/php-version-manager")}
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                <path d="M12 2C6.477 2 2 6.477 2 12C2 16.42 4.865 20.335 8.839 21.49C9.339 21.58 9.521 21.272 9.521 21.01C9.521 20.78 9.512 20.14 9.507 19.478C6.726 20.041 6.14 18.093 6.14 18.093C5.685 16.828 5.029 16.562 5.029 16.562C4.121 15.995 5.097 15.999 5.097 15.999C6.101 16.065 6.629 17.033 6.629 17.033C7.521 18.576 8.97 18.118 9.539 17.88C9.631 17.221 9.889 16.78 10.175 16.54C7.954 16.297 5.619 15.378 5.619 11.691C5.619 10.577 6.01 9.68 6.649 9.006C6.546 8.73 6.203 7.628 6.747 6.252C6.747 6.252 7.586 5.977 9.496 7.437C10.295 7.213 11.15 7.101 12 7.097C12.85 7.101 13.705 7.213 14.504 7.437C16.414 5.977 17.253 6.252 17.253 6.252C17.797 7.628 17.454 8.73 17.351 9.006C17.99 9.68 18.381 10.577 18.381 11.691C18.381 15.388 16.04 16.297 13.813 16.536C14.172 16.853 14.493 17.481 14.493 18.456C14.493 19.875 14.481 21.041 14.481 21.01C14.481 21.275 14.66 21.585 15.168 21.489C19.138 20.333 22 16.418 22 12C22 6.477 17.523 2 12 2Z" fill="currentColor"/>
              </svg>
              <span>View Source Code</span>
            </button>
          </div>

          <div className="about-divider"></div>

          <div className="about-developer">
            <div className="about-developer-name">
              <strong>JMSIT</strong>
              <span className="about-developer-role">Full-Stack Engineer & IT Solutions Specialist</span>
            </div>
            <div className="about-links">
              <button
                type="button"
                className="about-link-btn"
                title="Visit developer website"
                onClick={(e) => handleLinkClick(e, "https://jmsit.cloud/")}
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                  <path d="M12 2C6.48 2 2 6.48 2 12C2 17.52 6.48 22 12 22C17.52 22 22 17.52 22 12C22 6.48 17.52 2 12 2ZM13 17H11V15H13V17ZM13 13H11V7H13V13Z" fill="currentColor"/>
                </svg>
                <span>jmsit.cloud</span>
              </button>
              <button
                type="button"
                className="about-link-btn"
                title="View on GitHub"
                onClick={(e) => handleLinkClick(e, "https://github.com/vunf1/")}
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                  <path d="M12 2C6.477 2 2 6.477 2 12C2 16.42 4.865 20.335 8.839 21.49C9.339 21.58 9.521 21.272 9.521 21.01C9.521 20.78 9.512 20.14 9.507 19.478C6.726 20.041 6.14 18.093 6.14 18.093C5.685 16.828 5.029 16.562 5.029 16.562C4.121 15.995 5.097 15.999 5.097 15.999C6.101 16.065 6.629 17.033 6.629 17.033C7.521 18.576 8.97 18.118 9.539 17.88C9.631 17.221 9.889 16.78 10.175 16.54C7.954 16.297 5.619 15.378 5.619 11.691C5.619 10.577 6.01 9.68 6.649 9.006C6.546 8.73 6.203 7.628 6.747 6.252C6.747 6.252 7.586 5.977 9.496 7.437C10.295 7.213 11.15 7.101 12 7.097C12.85 7.101 13.705 7.213 14.504 7.437C16.414 5.977 17.253 6.252 17.253 6.252C17.797 7.628 17.454 8.73 17.351 9.006C17.99 9.68 18.381 10.577 18.381 11.691C18.381 15.388 16.04 16.297 13.813 16.536C14.172 16.853 14.493 17.481 14.493 18.456C14.493 19.875 14.481 21.041 14.481 21.01C14.481 21.275 14.66 21.585 15.168 21.489C19.138 20.333 22 16.418 22 12C22 6.477 17.523 2 12 2Z" fill="currentColor"/>
                </svg>
                <span>GitHub</span>
              </button>
            </div>
          </div>

          <div className="about-footer">
            <p className="about-copyright">
              © {new Date().getFullYear()} JMSIT. Licensed under MIT.
            </p>
            <p className="about-tagline">
              Crafting tools that heal and building systems that stand.
            </p>
          </div>
        </div>

        <div className="modal-footer about-modal-footer">
          <button
            className="btn btn-secondary about-close-btn"
            onClick={onClose}
          >
            Close
          </button>
        </div>
      </div>
    </div>
  );
};
