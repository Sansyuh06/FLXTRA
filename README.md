# Aegis Browser

A privacy-first, security-focused web browser built from scratch in Rust.

## Core Principles

- **Privacy by default** - No telemetry, no tracking, aggressive ad blocking
- **Security over convenience** - Sandboxed processes, no JIT by default
- **Minimalist UX** - Clean interface focused on content
- **Modular architecture** - Extensible without compromising security
- **Open source** - MPL-2.0 licensed, community-driven

## Architecture

```
aegis_browser     <- Main application entry point
├── aegis_ui      <- Windows native UI shell
├── aegis_render  <- GPU compositor and display
├── aegis_layout  <- CSS layout engine
├── aegis_html    <- HTML5 parser and DOM
├── aegis_css     <- CSS parser and styling
├── aegis_js      <- JavaScript interpreter
├── aegis_net     <- Network stack with DoH
├── aegis_filter  <- Ad/tracker blocking engine
├── aegis_sandbox <- Process isolation
├── aegis_mcp     <- AI assistant integration
└── aegis_core    <- Shared types and utilities
```

## Building

```bash
# Prerequisites: Rust 1.75+
cargo build --release
```

## Running

```bash
cargo run -p aegis_browser
```

## License

MPL-2.0 - See LICENSE file
