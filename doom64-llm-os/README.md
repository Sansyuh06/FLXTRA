# DoomLLM OS

A minimal bare-metal x86-64 operating system that runs **Doom64** and a **local LLM** (Phi-3 Mini).

## Features

- **Pure kernel** (no Linux, no distro)
- **CLI interface** with 3 commands: `/ai`, `/game`, `/off`
- **Local LLM inference** (Phi-3 Mini Q4_K_M, ~2.3GB)
- **Doom64 port** with Alt+F4 exit
- **Rust no_std** (fully type-safe, memory-safe)
- **QEMU testable** (or real hardware with Ryzen 5 + GTX 1650 Ti)

## Quick Start
1. Build the OS:
   ```bash
   ./scripts/build_iso.sh
   ```
2. Run in QEMU:
    ```bash
    ./scripts/test_qemu.sh
    ```

## Commands
- `/ai <prompt>`: Ask the AI a question.
- `/game`: Play Doom64 (Mock/Stub for MVP).
- `/off`: Shutdown.
