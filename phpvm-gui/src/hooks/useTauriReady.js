/**
 * Custom hook to wait for Tauri to be ready before making API calls
 */
import { useEffect, useState, useRef } from "react";

export const useTauriReady = (onReady) => {
  const [isReady, setIsReady] = useState(false);
  const [error, setError] = useState(null);
  const onReadyRef = useRef(onReady);
  const hasCalledOnReady = useRef(false);

  // Keep the ref updated with the latest callback
  useEffect(() => {
    onReadyRef.current = onReady;
  }, [onReady]);

  useEffect(() => {
    let attempts = 0;
    const maxAttempts = 100; // Try for up to 10 seconds
    let timeoutId = null;

    const checkTauri = () => {
      // Check multiple ways Tauri might expose itself
      const isTauri = typeof window !== 'undefined' && (
        window.__TAURI_INTERNALS__ ||
        window.__TAURI__ ||
        (window.invoke && typeof window.invoke === 'function')
      );

      if (isTauri) {
        setIsReady(true);
        // Only call onReady once
        if (onReadyRef.current && !hasCalledOnReady.current) {
          hasCalledOnReady.current = true;
          console.log("[useTauriReady] Tauri is ready, calling onReady");
          // Use setTimeout to ensure Tauri is fully initialized
          setTimeout(() => {
            try {
              onReadyRef.current();
            } catch (err) {
              console.error("[useTauriReady] Error calling onReady:", err);
            }
          }, 100);
        }
      } else if (attempts < maxAttempts) {
        attempts++;
        timeoutId = setTimeout(checkTauri, 100);
      } else {
        const err = "Tauri runtime is not available. Please ensure you're running the application through Tauri (not in a browser).";
        setError(err);
        setIsReady(false);
      }
    };

    // Start checking immediately
    checkTauri();
    
    // Cleanup timeout on unmount
    return () => {
      if (timeoutId) {
        clearTimeout(timeoutId);
      }
    };
  }, []); // Empty dependency array - only run once on mount

  return { isReady, error };
};
