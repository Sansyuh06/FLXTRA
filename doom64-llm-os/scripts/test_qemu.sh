#!/bin/bash

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE}")/.." && pwd)"
cd "$PROJECT_DIR"

if [ ! -f "os.iso" ]; then
    echo "[!] os.iso not found. Run './scripts/build_iso.sh' first."
    exit 1
fi

echo "[*] Launching QEMU..."
echo "[*] Commands:"
echo "    /ai what is machine learning?"
echo "    /game"
echo "    /off"
echo ""

qemu-system-x86_64 \
    -m 16G \
    -smp 6 \
    -enable-kvm \
    -cdrom os.iso \
    -display gtk \
    -monitor stdio
