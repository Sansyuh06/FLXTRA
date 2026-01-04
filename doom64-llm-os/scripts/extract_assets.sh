#!/bin/bash

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE}")/.." && pwd)"
cd "$PROJECT_DIR"

echo "[*] Asset Extraction Helper"
echo ""

# Find Steam Doom64 remaster
echo "[*] Looking for DOOM 64 (Steam)..."
STEAM_DIR="$HOME/.local/share/Steam/steamapps/common/DOOM 64"

if [ -f "$STEAM_DIR/DOOM64.wad" ]; then
    echo "[+] Found DOOM64.wad"
    cp "$STEAM_DIR/DOOM64.wad" disk/DOOM64.wad
    echo "[+] Copied to disk/DOOM64.wad"
else
    echo "[!] DOOM64.wad not found"
    echo "[*] To get it:"
    echo "    1. Purchase DOOM 64 from Steam (~$10)"
    echo "    2. Install it"
    echo "    3. Run this script again"
fi

# Download Phi-3 Mini quantized model
echo ""
echo "[*] Downloading Phi-3 Mini Q4_K_M..."
cd disk

if [ ! -f "model.gguf" ]; then
    # Use HuggingFace URL (adjust as needed)
    echo "[*] Downloading model (this may take a while)..."
    wget "https://huggingface.co/microsoft/Phi-3-mini-128k-instruct-gguf/resolve/main/Phi-3-mini-128k-instruct-Q4_K_M.gguf" \
        -O model.gguf -q --show-progress || echo "[!] Download failed. Please download manually."
else
    echo "[+] Model already exists"
fi

cd ..
echo "[+] Assets ready!"
