#!/bin/bash
set -e

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE}")/.." && pwd)"
cd "$PROJECT_DIR"

echo "[*] Building DoomLLM OS..."

# 1. Build kernel
echo "[*] Compiling kernel (Rust)..."
cd kernel
# Use absolute path for linker script to avoid lld errors
export RUSTFLAGS="-C link-arg=-T$(pwd)/linker.ld -C relocation-model=static"
cargo +nightly build --release -Z build-std=core,alloc
cd ..

# 2. Extract kernel binary
echo "[*] Extracting kernel binary..."
# Check if objcopy is available
if command -v objcopy &> /dev/null; then
    OBJCOPY=objcopy
elif command -v llvm-objcopy &> /dev/null; then
    OBJCOPY=llvm-objcopy
else
    echo "[!] objcopy not found. Please install binutils or llvm."
    exit 1
fi

# Try to find the kernel executable
KERNEL_ELF="target/x86_64-unknown-none/release/kernel"
if [ ! -f "$KERNEL_ELF" ]; then
    KERNEL_ELF="target/release/kernel"
fi

if [ ! -f "$KERNEL_ELF" ]; then
     # Try finding it
     KERNEL_ELF=$(find target -name "kernel" -type f | grep release | head -n 1)
fi

if [ -f "$KERNEL_ELF" ]; then
    $OBJCOPY -O binary "$KERNEL_ELF" kernel.bin
else
    echo "[!] Could not find kernel ELF binary."
    exit 1
fi

# 3. Create ISO structure
echo "[*] Creating ISO structure..."
mkdir -p iso/boot/grub
cp kernel.bin iso/boot/

# 4. Create GRUB config
cat > iso/boot/grub/grub.cfg <<'EOF'
menuentry 'DoomLLM OS' {
    multiboot2 /boot/kernel.bin
    boot
}

set default=0
set timeout=5
EOF

# 5. Create FAT32 filesystem (4GB)
echo "[*] Creating FAT32 filesystem..."
if [ ! -f "disk/fat32.img" ]; then
    # Create sparse file
    dd if=/dev/zero of=disk/fat32.img bs=1M count=0 seek=4096 2>/dev/null
    # Format
    if command -v mkfs.fat &> /dev/null; then
        mkfs.fat -F 32 disk/fat32.img
    else
        echo "[!] mkfs.fat not found. Skipping FAT32 creation."
    fi
fi

# 6. Mount and copy assets (Skipped in this automated script to avoid sudo requirement issues in some envs)
# The user can run specific asset scripts. We will just proceed to ISO.

# 7. Create ISO
echo "[*] Generating ISO with GRUB2..."
if command -v grub-mkrescue &> /dev/null; then
    grub-mkrescue -o os.iso iso/ disk/fat32.img 2>/dev/null
elif command -v xorriso &> /dev/null; then
    echo "[!] grub-mkrescue failed; using xorriso..."
    xorriso -as mkisofs -R -J -b isolinux.bin -no-emul-boot \
        -boot-load-size 4 -boot-info-table -o os.iso iso/
else
    echo "[!] Neither grub-mkrescue nor xorriso found."
    exit 1
fi

echo "[+] ISO generated: os.iso"
