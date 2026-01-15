#!/bin/bash
# Generate minimal valid PNG icons for Tauri builds
# These are placeholder icons - replace with proper icons for production

create_minimal_png() {
    local size=$1
    local filename=$2
    
    # Minimal valid PNG: 1x1 transparent pixel (base64 encoded)
    # This is a valid PNG that Tauri can use
    # Use unquoted heredoc to allow bash variable expansion for $size and $filename
    if ! python3 << PYTHON_SCRIPT; then
import struct
import zlib
import sys

def create_png(size, filename):
    try:
        # PNG signature
        png = b'\x89PNG\r\n\x1a\n'
        
        # IHDR chunk
        ihdr_data = struct.pack('>II', size, size)  # width, height
        ihdr_data += b'\x08\x06\x00\x00\x00'  # bit depth=8, color type=RGBA, compression=0, filter=0, interlace=0
        crc = zlib.crc32(b'IHDR' + ihdr_data) & 0xffffffff
        png += struct.pack('>I', len(ihdr_data))
        png += b'IHDR' + ihdr_data
        png += struct.pack('>I', crc)
        
        # IDAT chunk (minimal transparent image data)
        # For a transparent image, we need minimal zlib-compressed data
        idat_data = b'\x00' * (size * size * 4 + size)  # RGBA data with filter bytes
        idat_compressed = zlib.compress(idat_data)
        crc = zlib.crc32(b'IDAT' + idat_compressed) & 0xffffffff
        png += struct.pack('>I', len(idat_compressed))
        png += b'IDAT' + idat_compressed
        png += struct.pack('>I', crc)
        
        # IEND chunk - construct byte string to avoid backtick interpretation by bash
        iend_chunk = b'\x00\x00\x00\x00IEND'
        iend_chunk += bytes([0xae, 0x42, 0x60, 0x82])
        png += iend_chunk
        
        with open(filename, 'wb') as f:
            f.write(png)
        print(f"Created {filename} ({len(png)} bytes)")
    except Exception as e:
        print(f"Error creating {filename}: {e}", file=sys.stderr)
        sys.exit(1)

create_png($size, "$filename")
PYTHON_SCRIPT
        echo "Error: Failed to create $filename using Python" >&2
        exit 1
    fi
}

cd "$(dirname "$0")"

# Function to check if a file is a valid PNG
is_valid_png() {
    local file=$1
    if [ ! -f "$file" ]; then
        return 1
    fi
    # Check PNG signature (first 8 bytes: 89 50 4E 47 0D 0A 1A 0A)
    python3 -c "import sys; f = open('$file', 'rb'); h = f.read(8); f.close(); sys.exit(0 if h[:4] == b'\x89PNG' else 1)" 2>/dev/null
}

# Create required icon files if they don't exist or are invalid
if [ ! -f icon.png ] || ! is_valid_png icon.png; then
    echo "Creating icon.png..."
    create_minimal_png 256 icon.png
fi
if [ ! -f 32x32.png ] || ! is_valid_png 32x32.png; then
    echo "Creating 32x32.png..."
    create_minimal_png 32 32x32.png
fi
if [ ! -f 128x128.png ] || ! is_valid_png 128x128.png; then
    echo "Creating 128x128.png..."
    create_minimal_png 128 128x128.png
fi
if [ ! -f 128x128@2x.png ] || ! is_valid_png 128x128@2x.png; then
    echo "Creating 128x128@2x.png..."
    create_minimal_png 256 128x128@2x.png
fi

# Verify all files exist
for icon in icon.png 32x32.png 128x128.png 128x128@2x.png; do
    if [ ! -f "$icon" ]; then
        echo "Error: $icon was not created!" >&2
        exit 1
    fi
done

echo "Icon files generated successfully"
