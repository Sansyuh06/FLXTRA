# 🧛‍♀️ Marceline AI Assistant

A voice-activated AI assistant powered by Google Gemini (with Ollama fallback), available for both **Linux** and **Windows/Cross-platform**.

## Features

- 🎤 **Voice Recognition** - Say "Hey Marceline" to activate
- 🔊 **Text-to-Speech** - Natural voice responses
- 🤖 **Dual AI Support** - Gemini API + Ollama fallback
- 🔧 **MCP Integration** - File operations, web search, memory
- 💾 **Conversation Memory** - SQLite-based persistence
- 🖥️ **Desktop App** - Electron-based with system tray
- 🐧 **Linux Native** - Python-based voice assistant

---

## 🐧 Linux Installation (Recommended)

### Quick Install
```bash
cd marceline-ai
chmod +x install_linux.sh
./install_linux.sh
```

### What Gets Installed
- **Ollama** with llama3.2:3b model
- **Speech Recognition** (Google Speech API)
- **espeak-ng** for text-to-speech
- **System tools** (screenshot, volume control)

### Running on Linux
```bash
marceline
# Or
~/marceline/run.sh
```

### Linux Capabilities
- 🎙️ "Hey Marceline" wake word activation
- 📅 "What time is it?" / "What's the date?"
- 🖥️ "Open Firefox" / "Open terminal"
- 📸 "Take a screenshot"
- 🔊 "Volume up" / "Volume down"
- 🔍 "Search for Python tutorials"
- 💬 Any AI question!

---

## 🌐 Cross-Platform (Windows/Mac/Linux)

### Prerequisites
- Node.js 18+
- Gemini API key

### Setup
```bash
cd marceline-ai

# Backend
cd backend
npm install
# Edit .env to add GEMINI_API_KEY
npm run dev

# Frontend (new terminal)
cd frontend
npm install
npm run dev
```

### Access
- **Web UI**: http://localhost:3000
- **API**: http://localhost:3001

---

## Configuration

### Gemini API Key

**Linux version**: Edit `~/marceline/.env`
```
GEMINI_API_KEY=your_key_here
```

**Node.js version**: Edit `backend/.env`
```
GEMINI_API_KEY=your_key_here
```

### MCP Servers (Node.js version)
Edit `mcp-config.json` to add capabilities:
- `filesystem` - Read/write files
- `brave-search` - Web search
- `memory` - Persistent memory
- `fetch` - Read URLs

---

## Project Structure

```
marceline-ai/
├── install_linux.sh      # 🐧 Linux installer
├── backend/              # Express + TypeScript server
├── frontend/             # React + Vite app
├── electron/             # Desktop wrapper
├── mcp-config.json       # MCP server configuration
└── README.md
```

---

## Tech Stack

| Component | Linux Version | Node.js Version |
|-----------|--------------|-----------------|
| AI | Ollama + Gemini | Gemini API |
| Voice | SpeechRecognition | Web Speech API |
| TTS | espeak-ng | Web Speech Synthesis |
| Backend | Python | Express/TypeScript |
| Frontend | Terminal | React/Vite |

---

## License

MIT

---

*"I'm not mean, I'm a thousand years old and I just lost track of my moral code."* - Marceline 🦇
