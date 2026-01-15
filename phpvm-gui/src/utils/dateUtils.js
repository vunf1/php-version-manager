/**
 * Utility functions for date formatting
 */

/**
 * Format a date string to a localized date string
 */
export const formatDate = (dateString) => {
  if (!dateString) return "N/A";
  try {
    const date = new Date(dateString);
    return date.toLocaleDateString();
  } catch {
    return dateString;
  }
};

/**
 * Get EOL status class based on the EOL date
 * Returns: 'eol-future' (green), 'eol-current-year' (yellow), or 'eol-past' (red)
 */
export const getEolStatusClass = (eolDateString) => {
  if (!eolDateString) return '';
  
  try {
    const eolDate = new Date(eolDateString);
    const today = new Date();
    const currentYear = today.getFullYear();
    const eolYear = eolDate.getFullYear();
    
    // Set time to start of day for accurate comparison
    const todayStart = new Date(currentYear, today.getMonth(), today.getDate());
    const eolStart = new Date(eolYear, eolDate.getMonth(), eolDate.getDate());
    
    // EOL date is in the past
    if (eolStart < todayStart) {
      return 'eol-past';
    }
    
    // EOL date is in the current year
    if (eolYear === currentYear) {
      return 'eol-current-year';
    }
    
    // EOL date is in the future
    return 'eol-future';
  } catch {
    return '';
  }
};
