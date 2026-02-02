# Flextra Browser

A privacy-first web browser with AI-powered features, built in Rust.

## Features

### 🛡️ Privacy
- **Auto-Clear**: All browsing data deleted on exit
- **Ephemeral Sessions**: No cookies or history persist
- **Private Mode**: Always-on privacy indicator

### ✨ AI Assistant (Marceline)
- Page summarization
- Plain-English explanations
- Context-aware Q&A

### 🎨 Premium UI
- Clean, modern design
- 8-bit retro theme available
- Smooth transitions

### 🔒 Security
- Built-in ad blocker (50+ domains)
- HTTPS auto-upgrade
- DNS-over-HTTPS (Cloudflare)

## Running

```bash
.\Flxtra.exe
```

## System Requirements

- Windows 10/11
- WebView2 Runtime (pre-installed on Windows 10 21H2+)

## Build from Source

```powershell
cd d:\fyeshi\project\OS
cargo build --release -p flxtra_browser
copy target\release\Flxtra.exe dist\
```

## License

MPL-2.0
