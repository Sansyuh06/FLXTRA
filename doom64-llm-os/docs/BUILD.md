# DoomLLM OS Build Guide

This guide explains how to set up your environment to build the `os.iso` file.

## Prerequisites

You need a Unix-like environment (Linux, macOS, or **WSL on Windows**) with the following tools:

1.  **Rust (Nightly)**: The OS kernel requires Rust nightly features.
2.  **Build Tools**: `build-essential`, `llvm-tools-preview`.
3.  **ISO Tools**: `xorriso` or `grub-common` + `grub-pc-bin`.
4.  **QEMU**: For testing.

## Setup Instructions (Windows via WSL)

1.  **Install WSL (if not installed)**
    ```powershell
    wsl --install
    ```
    (Restart computer if required).

2.  **Open WSL Terminal**
    Open "Ubuntu" or your Linux distribution from the Start menu.

3.  **Install Dependencies**
    ```bash
    sudo apt update
    sudo apt install -y build-essential curl xorriso grub-common grub-pc-bin qemu-system-x86
    ```

4.  **Install Rust**
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    source "$HOME/.cargo/env"
    ```

5.  **Configure Rust for OSDev**
    ```bash
    rustup override set nightly
    rustup component add rust-src llvm-tools-preview
    ```

## Building the OS

1.  Navigate to the project directory (in WSL):
    ```bash
    # Assuming the project is on your D: drive
    cd /mnt/d/fyeshi/project/OS/doom64-llm-os
    ```

2.  Run the build script:
    ```bash
    ./scripts/build_iso.sh
    ```
    This will:
    *   Compile the kernel (Rust).
    *   Generate a FAT32 disk image (`disk/fat32.img`).
    *   Create `os.iso` with GRUB2 bootloader.

## Running

Run the QEMU test script:
```bash
./scripts/test_qemu.sh
```

## Troubleshooting

-   **`linker 'cc' not found`**: Run `sudo apt install build-essential`.
-   **`grub-mkrescue not found`**: Run `sudo apt install xorriso grub-common`.
-   **`cargo: command not found`**: restart your terminal or run `source $HOME/.cargo/env`.
