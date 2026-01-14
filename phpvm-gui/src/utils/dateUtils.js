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
