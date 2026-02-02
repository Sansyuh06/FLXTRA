# Flextra Browser

A privacy-first web browser with AI-powered features, built in Rust.

## Core Principles

- **Privacy by default** - Auto-clear on exit, no telemetry
- **AI-enhanced** - Marceline assistant for summaries and Q&A
- **Premium UX** - Clean, modern interface
- **Open source** - MPL-2.0 licensed

## Architecture

```
flxtra_browser     <- Main application (WebView2)
├── flxtra_ui      <- Windows native UI shell
├── flxtra_render  <- GPU compositor
├── flxtra_layout  <- CSS layout engine
├── flxtra_html    <- HTML5 parser
├── flxtra_css     <- CSS parser
├── flxtra_js      <- JavaScript interpreter
├── flxtra_net     <- Network stack with DoH
├── flxtra_filter  <- Ad/tracker blocking
└── flxtra_core    <- Shared utilities
```

## Features

- 🛡️ Auto-clear browsing data on exit
- ✨ Marceline AI assistant
- 🚫 Built-in ad blocker
- 🔒 HTTPS-only with auto-upgrade
- 🌐 DNS-over-HTTPS

## Building

```bash
cargo build --release -p flxtra_browser
```

## Running

```bash
.\dist\Flxtra.exe
```

## License

MPL-2.0
