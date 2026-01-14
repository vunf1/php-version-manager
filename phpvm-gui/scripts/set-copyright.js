// Node.js script to set copyright metadata in Windows executable using rcedit
import { execSync } from 'child_process';
import path from 'path';
import fs from 'fs';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Path to the built executable (Tauri builds to workspace root target directory)
const exePath = path.join(__dirname, '..', '..', 'target', 'release', 'phpvm-gui.exe');

if (!fs.existsSync(exePath)) {
  console.log('Executable not found, skipping copyright metadata update.');
  console.log('Expected path:', exePath);
  process.exit(0);
}

try {
  // Use rcedit to set copyright metadata
  // rcedit is a command-line tool, so we use execSync
  execSync(`npx rcedit "${exePath}" --set-version-string LegalCopyright "Copyleft 🄯 JMSIT.cloud"`, {
    stdio: 'inherit',
    cwd: path.join(__dirname, '..')
  });
  console.log('✓ Copyright metadata set successfully: Copyleft 🄯 JMSIT.cloud');
} catch (error) {
  console.log('Warning: Could not set copyright metadata using rcedit.');
  console.log('Error:', error.message);
  console.log('The build will continue, but copyright metadata will not be set.');
  console.log('You can manually set it using Resource Hacker if needed.');
}
