import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [
    react({
      // Enable Fast Refresh for instant React component updates
      fastRefresh: true,
    }),
  ],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  clearScreen: false,
  // tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    // Enable HMR (Hot Module Replacement) for instant updates
    hmr: {
      protocol: 'ws',
      host: 'localhost',
      port: 1420,
      clientPort: 1420,
    },
    watch: {
      // tell vite to ignore watching `src-tauri` (Tauri CLI handles Rust watch)
      ignored: ["**/src-tauri/**", "**/target/**", "**/node_modules/**"],
      // Use polling only if native file watching fails
      usePolling: false,
      // Optimize for many files
      interval: 100,
      binaryInterval: 1000,
    },
  },

  // Build optimizations
  build: {
    // Use multi-threaded minification (faster builds)
    minify: 'esbuild', // Faster than terser
    // Parallel chunk loading
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ['react', 'react-dom'],
        },
      },
    },
    // Increase chunk size warnings threshold
    chunkSizeWarningLimit: 1000,
  },

  // Optimize dependencies for faster HMR
  optimizeDeps: {
    include: ['react', 'react-dom'],
    // Exclude src-tauri from optimization
    exclude: ['@tauri-apps/api'],
    // Force pre-bundling for faster cold starts
    force: false,
    // Disable esbuild optimization in dev for faster startup
    esbuildOptions: {
      target: 'esnext',
    },
  },
  
  // Development optimizations
  logLevel: 'info',
  
  // Faster file system access
  resolve: {
    // Prefer ES modules for faster resolution
    mainFields: ['module', 'jsnext', 'jsnext:main', 'browser', 'main'],
  },
}));
