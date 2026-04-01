#!/usr/bin/env bash
# Portable launcher: runs generate_icons.py with a working Python 3 (not the Windows Store stub).
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"
PY_SCRIPT="${SCRIPT_DIR}/generate_icons.py"

if command -v python3 >/dev/null 2>&1 && python3 -c "import struct,zlib" >/dev/null 2>&1; then
  exec python3 "$PY_SCRIPT"
fi
if command -v python >/dev/null 2>&1 && python -c "import struct,zlib" >/dev/null 2>&1; then
  exec python "$PY_SCRIPT"
fi
if command -v py >/dev/null 2>&1 && py -3 -c "import struct,zlib" >/dev/null 2>&1; then
  exec py -3 "$PY_SCRIPT"
fi

echo "generate-icons.sh: need Python 3 (python3, python, or Windows 'py -3') with stdlib." >&2
exit 1
