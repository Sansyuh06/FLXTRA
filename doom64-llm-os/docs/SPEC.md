# DoomLLM OS Technical Specification

## Overview
A bare-metal x86-64 OS designed for a single purpose: Running Doom64 and a local LLM.

## Architecture
- **Language**: Rust (`no_std`)
- **Bootloader**: GRUB2 Multiboot2
- **Kernel**: Monolithic, specialized. 
- **Drivers**:
    - VGA (Text & Mode 13h)
    - PS/2 Keyboard
    - ACPI (Shutdown)
    - NVMe/AHCI (Stub/Planned for disk access)

## AI Subsystem
- **Model**: Phi-3 Mini Q4_K_M
- **Inference**: CPU-bound (MVP), planned GPU acceleration.
- **Implementation**: Custom GGUF loader.

## Game Subsystem
- **Engine**: Doom64EX-Plus port.
- **Rendering**: Software raycaster (VGA) or Vulkan (future).

## Build Pipeline
1. Compile Kernel (ELF).
2. Objcopy to binary.
3. Generate FAT32 disk with assets.
4. Bundle ISO.
