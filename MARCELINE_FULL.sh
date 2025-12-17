#!/usr/bin/env bash
# ╔═══════════════════════════════════════════════════════════════════════════╗
# ║                                                                           ║
# ║   ███╗   ███╗ █████╗ ██████╗  ██████╗███████╗██╗     ██╗███╗   ██╗███████╗║
# ║   ████╗ ████║██╔══██╗██╔══██╗██╔════╝██╔════╝██║     ██║████╗  ██║██╔════╝║
# ║   ██╔████╔██║███████║██████╔╝██║     █████╗  ██║     ██║██╔██╗ ██║█████╗  ║
# ║   ██║╚██╔╝██║██╔══██║██╔══██╗██║     ██╔══╝  ██║     ██║██║╚██╗██║██╔══╝  ║
# ║   ██║ ╚═╝ ██║██║  ██║██║  ██║╚██████╗███████╗███████╗██║██║ ╚████║███████╗║
# ║   ╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝╚══════╝╚══════╝╚═╝╚═╝  ╚═══╝╚══════╝║
# ║                                                                           ║
# ║   MARCELINE OS - Complete AI Personal Assistant                           ║
# ║   Version 2.0 - Full Featured Installation                                ║
# ║                                                                           ║
# ║   Features:                                                               ║
# ║   • Voice Activation with Wake Word Detection                             ║
# ║   • AI-Powered Responses (Ollama + LLaMA 3.2)                             ║
# ║   • Speech-to-Text (Vosk + Google Fallback)                               ║
# ║   • Text-to-Speech (espeak-ng)                                            ║
# ║   • System Control (Apps, Volume, Screenshots)                            ║
# ║   • Web Search & Browser Control                                          ║
# ║   • Calendar & Reminders                                                  ║
# ║   • Note Taking System                                                    ║
# ║   • Email Integration                                                     ║
# ║   • Music & Media Control                                                 ║
# ║   • Smart Home Stubs                                                      ║
# ║   • Memory & Context Awareness                                            ║
# ║   • Modern GUI Overlay                                                    ║
# ║   • MCP Server Framework (17 Servers)                                     ║
# ║   • Automatic Startup on Boot                                             ║
# ║                                                                           ║
# ╚═══════════════════════════════════════════════════════════════════════════╝

set -euo pipefail

# ═══════════════════════════════════════════════════════════════════════════
# CONFIGURATION
# ═══════════════════════════════════════════════════════════════════════════

VERSION="2.0.0"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
NC='\033[0m'

# ═══════════════════════════════════════════════════════════════════════════
# HELPER FUNCTIONS
# ═══════════════════════════════════════════════════════════════════════════

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[✓]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[!]${NC} $1"; }
log_error() { echo -e "${RED}[✗]${NC} $1"; }
log_step() { echo -e "${PURPLE}[STEP]${NC} $1"; }

die() {
    log_error "$1"
    exit 1
}

check_root() {
    if [[ $EUID -ne 0 ]]; then
        die "This script must be run with sudo: sudo bash $0"
    fi
}

detect_user() {
    REAL_USER="${SUDO_USER:-$USER}"
    REAL_HOME=$(eval echo "~$REAL_USER")
    REAL_UID=$(id -u "$REAL_USER")
    REAL_GID=$(id -g "$REAL_USER")
    MAR_DIR="${REAL_HOME}/marceline"
    
    log_info "Installing for user: $REAL_USER"
    log_info "Home directory: $REAL_HOME"
    log_info "Marceline directory: $MAR_DIR"
}

run_as_user() {
    sudo -u "$REAL_USER" "$@"
}

# ═══════════════════════════════════════════════════════════════════════════
# BANNER
# ═══════════════════════════════════════════════════════════════════════════

show_banner() {
    clear
    echo -e "${PURPLE}"
    echo "╔═══════════════════════════════════════════════════════════════════════════╗"
    echo "║                                                                           ║"
    echo "║   ███╗   ███╗ █████╗ ██████╗  ██████╗███████╗██╗     ██╗███╗   ██╗███████╗║"
    echo "║   ████╗ ████║██╔══██╗██╔══██╗██╔════╝██╔════╝██║     ██║████╗  ██║██╔════╝║"
    echo "║   ██╔████╔██║███████║██████╔╝██║     █████╗  ██║     ██║██╔██╗ ██║█████╗  ║"
    echo "║   ██║╚██╔╝██║██╔══██║██╔══██╗██║     ██╔══╝  ██║     ██║██║╚██╗██║██╔══╝  ║"
    echo "║   ██║ ╚═╝ ██║██║  ██║██║  ██║╚██████╗███████╗███████╗██║██║ ╚████║███████╗║"
    echo "║   ╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝╚══════╝╚══════╝╚═╝╚═╝  ╚═══╝╚══════╝║"
    echo "║                                                                           ║"
    echo "║                    Personal AI Assistant v${VERSION}                         ║"
    echo "║                                                                           ║"
    echo "╚═══════════════════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
    echo ""
}

# ═══════════════════════════════════════════════════════════════════════════
# SYSTEM PACKAGES INSTALLATION
# ═══════════════════════════════════════════════════════════════════════════

install_system_packages() {
    log_step "Installing system packages..."
    
    export DEBIAN_FRONTEND=noninteractive
    
    # Update package lists
    log_info "Updating package lists..."
    apt-get update -qq || log_warning "apt update had issues"
    
    # Core packages
    CORE_PACKAGES=(
        # Python
        python3 python3-pip python3-venv python3-dev python3-tk
        # Audio
        espeak-ng espeak alsa-utils pulseaudio pavucontrol
        portaudio19-dev python3-pyaudio
        # System utilities
        curl wget unzip git jq bc
        # Desktop integration
        xclip scrot wmctrl xdotool libnotify-bin
        # Media
        ffmpeg imagemagick
        # Database
        sqlite3
        # Build tools
        build-essential
        # Network
        net-tools openssh-client
    )
    
    for pkg in "${CORE_PACKAGES[@]}"; do
        log_info "Installing $pkg..."
        if apt-get install -y --no-install-recommends "$pkg" > /dev/null 2>&1; then
            log_success "$pkg installed"
        else
            log_warning "$pkg could not be installed (may already exist or name differs)"
        fi
    done
    
    log_success "System packages installation complete"
}

# ═══════════════════════════════════════════════════════════════════════════
# OLLAMA AI INSTALLATION
# ═══════════════════════════════════════════════════════════════════════════

install_ollama() {
    log_step "Installing Ollama AI..."
    
    if command -v ollama &>/dev/null; then
        log_info "Ollama already installed"
    else
        log_info "Downloading and installing Ollama..."
        curl -fsSL https://ollama.com/install.sh | sh || die "Ollama installation failed"
    fi
    
    # Enable and start Ollama service
    systemctl enable ollama.service > /dev/null 2>&1 || true
    systemctl start ollama.service > /dev/null 2>&1 || true
    
    # Wait for Ollama to be ready
    log_info "Waiting for Ollama to start..."
    sleep 5
    
    # Pull AI models
    log_info "Pulling AI models (this may take several minutes)..."
    
    # Primary fast model
    log_info "Pulling llama3.2:3b (fast responses)..."
    run_as_user ollama pull llama3.2:3b > /dev/null 2>&1 || log_warning "Could not pull llama3.2:3b"
    
    log_success "Ollama installation complete"
}

# ═══════════════════════════════════════════════════════════════════════════
# DIRECTORY STRUCTURE
# ═══════════════════════════════════════════════════════════════════════════

create_directory_structure() {
    log_step "Creating directory structure..."
    
    # Main directories
    run_as_user mkdir -p "$MAR_DIR"/{assistant-core,mcp-servers,models,data,logs,config,themes,plugins,cache,temp}
    run_as_user mkdir -p "$MAR_DIR"/data/{notes,calendar,reminders,memory,conversations}
    run_as_user mkdir -p "$MAR_DIR"/mcp-servers/{file,system,clipboard,notification,websearch,textgen,git,window,camera,tts,stt,code,browser,email,calendar,extraction}
    
    # System directories
    mkdir -p /etc/marceline
    mkdir -p /var/log/marceline
    mkdir -p /var/lib/marceline
    
    # Set permissions
    chown -R "$REAL_USER:$REAL_USER" "$MAR_DIR"
    chown -R "$REAL_USER:$REAL_USER" /var/log/marceline
    chown -R "$REAL_USER:$REAL_USER" /var/lib/marceline
    chmod 755 /etc/marceline
    
    log_success "Directory structure created"
}

# ═══════════════════════════════════════════════════════════════════════════
# PYTHON VIRTUAL ENVIRONMENT
# ═══════════════════════════════════════════════════════════════════════════

setup_python_environment() {
    log_step "Setting up Python virtual environment..."
    
    cd "$MAR_DIR"
    
    # Create virtual environment
    run_as_user python3 -m venv venv
    
    # Install Python packages
    log_info "Installing Python packages..."
    run_as_user bash -c "source venv/bin/activate && pip install --upgrade pip"
    run_as_user bash -c "source venv/bin/activate && pip install -q \
        requests \
        psutil \
        vosk \
        SpeechRecognition \
        pyaudio \
        pyttsx3 \
        flask \
        flask-cors \
        websockets \
        aiohttp \
        asyncio \
        python-dateutil \
        schedule \
        sqlite-utils \
        beautifulsoup4 \
        lxml \
        Pillow \
        opencv-python-headless \
        duckduckgo-search \
        python-dotenv \
        pyyaml \
        rich \
        typer \
        httpx \\
        pytz"
    
    log_success "Python environment ready"
}

# ═══════════════════════════════════════════════════════════════════════════
# VOSK SPEECH MODEL
# ═══════════════════════════════════════════════════════════════════════════

install_vosk_model() {
    log_step "Installing Vosk speech recognition model..."
    
    MODEL_DIR="$MAR_DIR/models"
    MODEL_NAME="vosk-model-small-en-us-0.15"
    
    if [[ -d "$MODEL_DIR/$MODEL_NAME" ]]; then
        log_info "Vosk model already installed"
    else
        cd "$MODEL_DIR"
        log_info "Downloading Vosk model..."
        run_as_user wget -q "https://alphacephei.com/vosk/models/${MODEL_NAME}.zip"
        run_as_user unzip -q "${MODEL_NAME}.zip"
        rm -f "${MODEL_NAME}.zip"
    fi
    
    log_success "Vosk model ready"
}

# ═══════════════════════════════════════════════════════════════════════════
# PIPER NEURAL TTS (Natural Voice)
# ═══════════════════════════════════════════════════════════════════════════

install_piper_tts() {
    log_step "Installing Piper neural TTS (natural voice)..."
    
    PIPER_DIR="$MAR_DIR/models/piper"
    run_as_user mkdir -p "$PIPER_DIR"
    
    # Download piper binary
    PIPER_VERSION="2023.11.14-2"
    PIPER_URL="https://github.com/rhasspy/piper/releases/download/${PIPER_VERSION}/piper_linux_x86_64.tar.gz"
    
    if ! command -v piper &>/dev/null; then
        log_info "Downloading piper TTS engine..."
        cd /tmp
        wget -q "$PIPER_URL" -O piper.tar.gz || log_warning "Could not download piper"
        if [[ -f piper.tar.gz ]]; then
            tar -xzf piper.tar.gz
            cp piper/piper /usr/local/bin/
            chmod +x /usr/local/bin/piper
            rm -rf piper piper.tar.gz
            log_success "Piper TTS installed"
        fi
    else
        log_info "Piper already installed"
    fi
    
    # Download British female voice (Alba - natural sounding)
    VOICE_NAME="en_GB-alba-medium"
    VOICE_URL="https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/en/en_GB/alba/medium/en_GB-alba-medium.onnx"
    VOICE_JSON="https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/en/en_GB/alba/medium/en_GB-alba-medium.onnx.json"
    
    if [[ ! -f "$PIPER_DIR/${VOICE_NAME}.onnx" ]]; then
        log_info "Downloading Alba British female voice..."
        cd "$PIPER_DIR"
        run_as_user wget -q "$VOICE_URL" -O "${VOICE_NAME}.onnx" || log_warning "Could not download voice model"
        run_as_user wget -q "$VOICE_JSON" -O "${VOICE_NAME}.onnx.json" || log_warning "Could not download voice config"
        
        if [[ -f "${VOICE_NAME}.onnx" ]]; then
            log_success "Alba voice model ready (natural British female)"
        fi
    else
        log_info "Voice model already installed"
    fi
    
    # Set ownership
    chown -R "$REAL_USER:$REAL_USER" "$PIPER_DIR"
    
    log_success "Piper neural TTS ready"
}

# ═══════════════════════════════════════════════════════════════════════════
# MAIN CONFIGURATION FILE
# ═══════════════════════════════════════════════════════════════════════════

create_main_config() {
    log_step "Creating main configuration..."
    
    cat > /etc/marceline/config.json <<'CONFIGEOF'
{
    "version": "2.0.0",
    "name": "Marceline",
    "personality": {
        "greeting": "Hey! I'm Marceline, your personal AI assistant. How can I help you today?",
        "farewell": "Goodbye! Call me anytime you need help.",
        "error_response": "I'm having trouble with that. Could you try again?",
        "thinking_response": "Let me think about that...",
        "listening_response": "I'm listening...",
        "style": "friendly_casual"
    },
    "wake_words": [
        "marceline",
        "hey marceline",
        "marcy",
        "hey marcy",
        "computer",
        "hey computer",
        "ok marceline"
    ],
    "voice": {
        "engine": "espeak-ng",
        "voice": "en-gb+f4",
        "rate": 160,
        "pitch": 55,
        "volume": 100
    },
    "stt": {
        "engine": "vosk",
        "model_path": "${HOME}/marceline/models/vosk-model-small-en-us-0.15",
        "sample_rate": 16000,
        "silence_threshold": 500,
        "phrase_timeout": 3.0,
        "fallback": "google"
    },
    "llm": {
        "provider": "ollama",
        "model": "llama3.2:3b",
        "host": "http://localhost:11434",
        "temperature": 0.7,
        "max_tokens": 150,
        "context_window": 4096,
        "system_prompt": "You are Marceline, a helpful, friendly AI assistant. Be concise and natural."
    },
    "memory": {
        "enabled": true,
        "database": "${HOME}/marceline/data/memory/marceline.db",
        "max_context": 20,
        "remember_user_preferences": true
    },
    "features": {
        "voice_activation": true,
        "continuous_listening": true,
        "gui_overlay": true,
        "notifications": true,
        "calendar": true,
        "notes": true,
        "reminders": true,
        "web_search": true,
        "email": false,
        "smart_home": false
    },
    "mcp": {
        "enabled": true,
        "host": "localhost",
        "port": 8080,
        "servers": [
            "file", "system", "clipboard", "notification",
            "websearch", "textgen", "git", "window",
            "camera", "tts", "stt", "code", "browser",
            "email", "calendar", "extraction"
        ]
    },
    "ui": {
        "theme": "dark",
        "overlay_position": "bottom-right",
        "overlay_size": "medium",
        "transparency": 0.95,
        "animations": true
    },
    "logging": {
        "level": "INFO",
        "file": "/var/log/marceline/marceline.log",
        "max_size_mb": 10,
        "backup_count": 3
    },
    "security": {
        "require_wake_word": true,
        "timeout_seconds": 30,
        "max_command_length": 500
    }
}
CONFIGEOF
    
    chmod 644 /etc/marceline/config.json
    
    # Also create user-level config
    run_as_user mkdir -p "$MAR_DIR/config"
    cp /etc/marceline/config.json "$MAR_DIR/config/config.json"
    chown "$REAL_USER:$REAL_USER" "$MAR_DIR/config/config.json"
    
    log_success "Configuration created"
}

# ═══════════════════════════════════════════════════════════════════════════
# MARCELINE VOICE ASSISTANT - MAIN ENGINE
# ═══════════════════════════════════════════════════════════════════════════

create_voice_assistant() {
    log_step "Creating Marceline Voice Assistant..."
    
    cat > "$MAR_DIR/assistant-core/marceline.py" <<'ASSISTANTEOF'
#!/usr/bin/env python3
"""
═══════════════════════════════════════════════════════════════════════════════
MARCELINE - Personal AI Voice Assistant
═══════════════════════════════════════════════════════════════════════════════
A comprehensive voice-activated AI assistant with:
- Continuous wake word detection
- Fast speech-to-text (Vosk + Google fallback)
- Natural text-to-speech (espeak-ng)
- AI-powered responses (Ollama + LLaMA)
- System control and automation
- Memory and context awareness
- MCP server integration
═══════════════════════════════════════════════════════════════════════════════
"""

import os
import sys
import time
import json
import wave
import queue
import tempfile
import threading
import subprocess
import sqlite3
import hashlib
import requests
import asyncio
from pathlib import Path
from datetime import datetime, timedelta
from typing import Optional, Dict, List, Any, Callable
from dataclasses import dataclass, field
from enum import Enum
import signal

# ═══════════════════════════════════════════════════════════════════════════
# CONSTANTS AND CONFIGURATION
# ═══════════════════════════════════════════════════════════════════════════

VERSION = "2.0.0"
CONFIG_PATH = "/etc/marceline/config.json"
USER_CONFIG_PATH = os.path.expanduser("~/marceline/config/config.json")
EVENT_FILE = "/tmp/marceline_event.json"
LOG_FILE = "/var/log/marceline/marceline.log"
OLLAMA_URL = "http://localhost:11434/api/generate"
MCP_URL = "http://localhost:8080"
SAMPLE_RATE = 16000

class AssistantState(Enum):
    IDLE = "idle"
    LISTENING = "listening"
    PROCESSING = "processing"
    SPEAKING = "speaking"
    ERROR = "error"

@dataclass
class ConversationContext:
    messages: List[Dict[str, str]] = field(default_factory=list)
    user_name: Optional[str] = None
    last_topic: Optional[str] = None
    last_interaction: Optional[datetime] = None
    preferences: Dict[str, Any] = field(default_factory=dict)

# ═══════════════════════════════════════════════════════════════════════════
# LOGGING SYSTEM
# ═══════════════════════════════════════════════════════════════════════════

class Logger:
    def __init__(self, name: str = "Marceline"):
        self.name = name
        self.log_file = LOG_FILE
        
    def _format(self, level: str, msg: str) -> str:
        ts = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        return f"[{ts}] [{level}] {msg}"
    
    def _write(self, level: str, msg: str):
        formatted = self._format(level, msg)
        print(formatted)
        sys.stdout.flush()
        try:
            with open(self.log_file, "a") as f:
                f.write(formatted + "\n")
        except:
            pass
    
    def info(self, msg: str): self._write("INFO", msg)
    def warning(self, msg: str): self._write("WARN", msg)
    def error(self, msg: str): self._write("ERROR", msg)
    def debug(self, msg: str): self._write("DEBUG", msg)

log = Logger()

# ═══════════════════════════════════════════════════════════════════════════
# EVENT SYSTEM
# ═══════════════════════════════════════════════════════════════════════════

def send_event(event_type: str, **data):
    """Send event to GUI overlay and other listeners."""
    try:
        event = {
            "type": event_type,
            "timestamp": time.time(),
            "datetime": datetime.now().isoformat(),
            **data
        }
        with open(EVENT_FILE, "w") as f:
            json.dump(event, f)
    except Exception as e:
        log.error(f"Event send failed: {e}")

# ═══════════════════════════════════════════════════════════════════════════
# MEMORY ENGINE
# ═══════════════════════════════════════════════════════════════════════════

class MemoryEngine:
    """Persistent memory for context and user preferences."""
    
    def __init__(self, db_path: str = None):
        self.db_path = db_path or os.path.expanduser("~/marceline/data/memory/marceline.db")
        os.makedirs(os.path.dirname(self.db_path), exist_ok=True)
        self._init_db()
    
    def _init_db(self):
        conn = sqlite3.connect(self.db_path)
        c = conn.cursor()
        
        # Conversations table
        c.execute('''CREATE TABLE IF NOT EXISTS conversations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            user_input TEXT,
            assistant_response TEXT,
            topic TEXT,
            sentiment REAL
        )''')
        
        # User preferences table
        c.execute('''CREATE TABLE IF NOT EXISTS preferences (
            key TEXT PRIMARY KEY,
            value TEXT,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )''')
        
        # Notes table
        c.execute('''CREATE TABLE IF NOT EXISTS notes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT,
            content TEXT,
            tags TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )''')
        
        # Reminders table
        c.execute('''CREATE TABLE IF NOT EXISTS reminders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            message TEXT,
            remind_at DATETIME,
            recurring TEXT,
            completed INTEGER DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )''')
        
        # Calendar events table
        c.execute('''CREATE TABLE IF NOT EXISTS calendar_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT,
            description TEXT,
            start_time DATETIME,
            end_time DATETIME,
            location TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )''')
        
        conn.commit()
        conn.close()
        log.info("Memory engine initialized")
    
    def save_conversation(self, user_input: str, response: str, topic: str = None):
        conn = sqlite3.connect(self.db_path)
        c = conn.cursor()
        c.execute("INSERT INTO conversations (user_input, assistant_response, topic) VALUES (?, ?, ?)",
                  (user_input, response, topic))
        conn.commit()
        conn.close()
    
    def get_recent_conversations(self, limit: int = 10) -> List[Dict]:
        conn = sqlite3.connect(self.db_path)
        c = conn.cursor()
        c.execute("SELECT user_input, assistant_response, topic, timestamp FROM conversations ORDER BY id DESC LIMIT ?", (limit,))
        rows = c.fetchall()
        conn.close()
        return [{"user": r[0], "assistant": r[1], "topic": r[2], "time": r[3]} for r in rows]
    
    def set_preference(self, key: str, value: str):
        conn = sqlite3.connect(self.db_path)
        c = conn.cursor()
        c.execute("INSERT OR REPLACE INTO preferences (key, value, updated_at) VALUES (?, ?, CURRENT_TIMESTAMP)",
                  (key, value))
        conn.commit()
        conn.close()
    
    def get_preference(self, key: str, default: str = None) -> Optional[str]:
        conn = sqlite3.connect(self.db_path)
        c = conn.cursor()
        c.execute("SELECT value FROM preferences WHERE key = ?", (key,))
        row = c.fetchone()
        conn.close()
        return row[0] if row else default
    
    def add_note(self, title: str, content: str, tags: str = "") -> int:
        conn = sqlite3.connect(self.db_path)
        c = conn.cursor()
        c.execute("INSERT INTO notes (title, content, tags) VALUES (?, ?, ?)", (title, content, tags))
        note_id = c.lastrowid
        conn.commit()
        conn.close()
        return note_id
    
    def search_notes(self, query: str) -> List[Dict]:
        conn = sqlite3.connect(self.db_path)
        c = conn.cursor()
        c.execute("SELECT id, title, content, tags FROM notes WHERE title LIKE ? OR content LIKE ?",
                  (f"%{query}%", f"%{query}%"))
        rows = c.fetchall()
        conn.close()
        return [{"id": r[0], "title": r[1], "content": r[2], "tags": r[3]} for r in rows]
    
    def add_reminder(self, message: str, remind_at: datetime) -> int:
        conn = sqlite3.connect(self.db_path)
        c = conn.cursor()
        c.execute("INSERT INTO reminders (message, remind_at) VALUES (?, ?)",
                  (message, remind_at.isoformat()))
        reminder_id = c.lastrowid
        conn.commit()
        conn.close()
        return reminder_id
    
    def get_pending_reminders(self) -> List[Dict]:
        conn = sqlite3.connect(self.db_path)
        c = conn.cursor()
        now = datetime.now().isoformat()
        c.execute("SELECT id, message, remind_at FROM reminders WHERE remind_at <= ? AND completed = 0", (now,))
        rows = c.fetchall()
        conn.close()
        return [{"id": r[0], "message": r[1], "remind_at": r[2]} for r in rows]
    
    def complete_reminder(self, reminder_id: int):
        conn = sqlite3.connect(self.db_path)
        c = conn.cursor()
        c.execute("UPDATE reminders SET completed = 1 WHERE id = ?", (reminder_id,))
        conn.commit()
        conn.close()
    
    def add_calendar_event(self, title: str, start_time: datetime, end_time: datetime = None, 
                           description: str = "", location: str = "") -> int:
        conn = sqlite3.connect(self.db_path)
        c = conn.cursor()
        c.execute("""INSERT INTO calendar_events (title, description, start_time, end_time, location) 
                     VALUES (?, ?, ?, ?, ?)""",
                  (title, description, start_time.isoformat(), 
                   end_time.isoformat() if end_time else None, location))
        event_id = c.lastrowid
        conn.commit()
        conn.close()
        return event_id
    
    def get_upcoming_events(self, days: int = 7) -> List[Dict]:
        conn = sqlite3.connect(self.db_path)
        c = conn.cursor()
        now = datetime.now().isoformat()
        future = (datetime.now() + timedelta(days=days)).isoformat()
        c.execute("""SELECT id, title, description, start_time, end_time, location 
                     FROM calendar_events WHERE start_time >= ? AND start_time <= ?
                     ORDER BY start_time""", (now, future))
        rows = c.fetchall()
        conn.close()
        return [{"id": r[0], "title": r[1], "description": r[2], 
                 "start": r[3], "end": r[4], "location": r[5]} for r in rows]

# ═══════════════════════════════════════════════════════════════════════════
# ROBUST SPEECH ENGINE (Using SpeechRecognition)
# ═══════════════════════════════════════════════════════════════════════════

class SpeechEngine:
    """
    Robust speech recognition using SpeechRecognition library.
    Works with most microphones out of the box.
    """
    
    def __init__(self):
        import speech_recognition as sr
        self.recognizer = sr.Recognizer()
        self.microphone = None
        self._init_mic()
        
        # Adjust for ambient noise on startup
        self.recognizer.energy_threshold = 300
        self.recognizer.dynamic_energy_threshold = True
        self.recognizer.pause_threshold = 0.8
        
        log.info("Speech engine initialized")
    
    def _init_mic(self):
        """Initialize microphone with fallback."""
        import speech_recognition as sr
        try:
            # List available mics
            mic_list = sr.Microphone.list_microphone_names()
            log.info(f"Found {len(mic_list)} microphones")
            for i, name in enumerate(mic_list[:5]):
                log.info(f"  [{i}] {name}")
            
            # Try default mic first
            self.microphone = sr.Microphone()
            
            # Test it
            with self.microphone as source:
                self.recognizer.adjust_for_ambient_noise(source, duration=0.5)
            log.info("Using default microphone")
        except Exception as e:
            log.error(f"Microphone init error: {e}")
            # Try first available mic
            try:
                self.microphone = sr.Microphone(device_index=0)
                log.info("Using microphone index 0")
            except:
                log.error("No microphone available!")
    
    def listen(self, timeout: float = 3.0, phrase_limit: float = 6.0) -> str:
        """
        Listen for speech and return transcribed text.
        Uses Google Speech Recognition (free, no API key needed).
        """
        import speech_recognition as sr
        
        if not self.microphone:
            log.error("No microphone!")
            return ""
        
        try:
            with self.microphone as source:
                log.debug("Listening...")
                audio = self.recognizer.listen(
                    source, 
                    timeout=timeout,
                    phrase_time_limit=phrase_limit
                )
            
            # Try Google first (most reliable, free)
            try:
                text = self.recognizer.recognize_google(audio)
                log.info(f"Heard: {text}")
                return text
            except sr.UnknownValueError:
                log.debug("Could not understand audio")
                return ""
            except sr.RequestError as e:
                log.warning(f"Google API error: {e}")
                # Fallback to Sphinx (offline but less accurate)
                try:
                    text = self.recognizer.recognize_sphinx(audio)
                    log.info(f"Heard (offline): {text}")
                    return text
                except:
                    return ""
                    
        except sr.WaitTimeoutError:
            log.debug("No speech detected (timeout)")
            return ""
        except Exception as e:
            log.error(f"Listen error: {e}")
            return ""
    
    def listen_for_wake(self, wake_words: list, timeout: float = 2.0) -> tuple:
        """
        Listen specifically for wake words.
        Returns (detected: bool, full_text: str)
        """
        text = self.listen(timeout=timeout, phrase_limit=3.0)
        if not text:
            return False, ""
        
        text_lower = text.lower()
        for wake in wake_words:
            if wake.lower() in text_lower:
                return True, text
        
        return False, text
    
    def listen_for_command(self, timeout: float = 5.0) -> str:
        """Listen for a command after wake word."""
        return self.listen(timeout=timeout, phrase_limit=8.0)
    
    def calibrate(self, duration: float = 1.0):
        """Calibrate for ambient noise."""
        if self.microphone:
            try:
                with self.microphone as source:
                    log.info("Calibrating for ambient noise...")
                    self.recognizer.adjust_for_ambient_noise(source, duration=duration)
                    log.info(f"Energy threshold: {self.recognizer.energy_threshold}")
            except Exception as e:
                log.warning(f"Calibration error: {e}")

# ═══════════════════════════════════════════════════════════════════════════
# TEXT-TO-SPEECH ENGINE (Works Reliably)
# ═══════════════════════════════════════════════════════════════════════════

class TTSEngine:
    """Text-to-speech - uses espeak with better settings."""
    
    def __init__(self, voice: str = "en-gb+f4", rate: int = 150):
        self.voice = voice
        self.rate = rate
        self.engine = self._find_engine()
        log.info(f"TTS ready: {self.engine}")
    
    def _find_engine(self) -> str:
        """Find available TTS engine."""
        for eng in ["espeak-ng", "espeak"]:
            try:
                subprocess.run([eng, "--version"], capture_output=True, timeout=3)
                return eng
            except:
                pass
        return "espeak-ng"
    
    def speak(self, text: str):
        """Speak the given text with female British voice."""
        if not text or not text.strip():
            return
        
        # Clean text for speech
        text = text.replace('"', '').replace("'", "").strip()
        if len(text) > 500:
            text = text[:500] + "..."
        
        send_event("speaking", text=text[:100])
        log.info(f"Speaking: {text[:50]}...")
        
        try:
            # Use espeak-ng with British female voice
            # -v en-gb+f4 = British female variant 4
            # -s 150 = slightly slower for clarity
            # -p 55 = slightly higher pitch
            # -g 8 = word gap for natural pacing
            subprocess.run([
                self.engine,
                "-v", "en-gb+f4",
                "-s", "150",
                "-p", "55", 
                "-g", "8",
                text
            ], capture_output=True, timeout=60, check=False)
        except subprocess.TimeoutExpired:
            log.warning("Speech timed out")
        except Exception as e:
            log.error(f"TTS error: {e}")
            # Last resort fallback
            try:
                subprocess.run(["espeak", text], capture_output=True, timeout=30)
            except:
                pass

# ═══════════════════════════════════════════════════════════════════════════
# AI ENGINE (OLLAMA)
# ═══════════════════════════════════════════════════════════════════════════

class AIEngine:
    """AI response generation using Ollama."""
    
    def __init__(self, model: str = "llama3.2:3b", host: str = OLLAMA_URL):
        self.model = model
        self.host = host
        self.system_prompt = """You are Marceline, a helpful, friendly AI assistant.
Be concise and natural in your responses. Keep answers under 50 words unless more detail is needed.
You can help with questions, tasks, information, jokes, and general conversation.
Be warm, personable, and use casual language. You have a slightly playful personality."""
    
    def generate(self, prompt: str, context: List[Dict] = None) -> str:
        """Generate AI response."""
        try:
            # Build context from conversation history
            full_prompt = self.system_prompt + "\n\n"
            if context:
                for msg in context[-5:]:  # Last 5 messages for context
                    full_prompt += f"User: {msg.get('user', '')}\nMarceline: {msg.get('assistant', '')}\n"
            full_prompt += f"User: {prompt}\nMarceline:"
            
            response = requests.post(
                self.host,
                json={
                    "model": self.model,
                    "prompt": full_prompt,
                    "stream": False,
                    "options": {
                        "temperature": 0.7,
                        "num_predict": 150
                    }
                },
                timeout=30
            )
            
            if response.status_code == 200:
                data = response.json()
                answer = data.get("response", "").strip()
                # Clean up
                answer = answer.split("\n")[0]
                if len(answer) > 250:
                    answer = answer[:250] + "..."
                return answer
        except requests.exceptions.Timeout:
            log.warning("AI request timed out")
        except Exception as e:
            log.error(f"AI error: {e}")
        
        return "I'm having trouble thinking right now. Try again?"

# ═══════════════════════════════════════════════════════════════════════════
# COMMAND PROCESSOR
# ═══════════════════════════════════════════════════════════════════════════

class CommandProcessor:
    """Process and execute user commands."""
    
    def __init__(self, memory: MemoryEngine, ai: AIEngine, tts: TTSEngine):
        self.memory = memory
        self.ai = ai
        self.tts = tts
        self.commands = self._build_command_map()
    
    def _build_command_map(self) -> Dict[str, Callable]:
        return {
            # Time and Date
            "time": self.cmd_time,
            "date": self.cmd_date,
            "day": self.cmd_date,
            
            # System Control
            "open firefox": self.cmd_open_firefox,
            "open browser": self.cmd_open_firefox,
            "open terminal": self.cmd_open_terminal,
            "open files": self.cmd_open_files,
            "open settings": self.cmd_open_settings,
            "open calculator": self.cmd_open_calculator,
            "open text editor": self.cmd_open_editor,
            
            # Volume
            "volume up": self.cmd_volume_up,
            "louder": self.cmd_volume_up,
            "volume down": self.cmd_volume_down,
            "quieter": self.cmd_volume_down,
            "mute": self.cmd_mute,
            "unmute": self.cmd_unmute,
            
            # Screenshots
            "screenshot": self.cmd_screenshot,
            "take screenshot": self.cmd_screenshot,
            
            # Search
            "search": self.cmd_search,
            "google": self.cmd_search,
            "look up": self.cmd_search,
            
            # Web
            "youtube": self.cmd_youtube,
            "weather": self.cmd_weather,
            "news": self.cmd_news,
            "music": self.cmd_music,
            "spotify": self.cmd_music,
            
            # Notes
            "note": self.cmd_note,
            "remember": self.cmd_note,
            "save note": self.cmd_note,
            
            # Reminders
            "remind": self.cmd_reminder,
            "set reminder": self.cmd_reminder,
            
            # Calendar
            "calendar": self.cmd_calendar,
            "schedule": self.cmd_calendar,
            "events": self.cmd_calendar,
            
            # System Info
            "battery": self.cmd_battery,
            "system": self.cmd_system_info,
            
            # Control
            "stop": self.cmd_stop,
            "quiet": self.cmd_stop,
            "shut up": self.cmd_stop,
            "thank": self.cmd_thanks,
            "thanks": self.cmd_thanks,
            "hello": self.cmd_hello,
            "hi": self.cmd_hello,
            "help": self.cmd_help,
        }
    
    def process(self, text: str) -> str:
        """Process user input and return response."""
        text_lower = text.lower().strip()
        
        # Check for exact command matches
        for trigger, handler in self.commands.items():
            if trigger in text_lower:
                return handler(text)
        
        # Fall back to AI for everything else
        context = self.memory.get_recent_conversations(5)
        response = self.ai.generate(text, context)
        self.memory.save_conversation(text, response)
        return response
    
    # ─── Command Implementations ───────────────────────────────────────────
    
    def cmd_time(self, text: str) -> str:
        return f"It's {datetime.now().strftime('%I:%M %p')}"
    
    def cmd_date(self, text: str) -> str:
        return f"Today is {datetime.now().strftime('%A, %B %d, %Y')}"
    
    def cmd_open_firefox(self, text: str) -> str:
        subprocess.Popen(["firefox"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        return "Opening Firefox"
    
    def cmd_open_terminal(self, text: str) -> str:
        for term in ["gnome-terminal", "konsole", "xfce4-terminal", "xterm"]:
            try:
                subprocess.Popen([term], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
                return "Opening terminal"
            except:
                continue
        return "Couldn't find a terminal"
    
    def cmd_open_files(self, text: str) -> str:
        for fm in ["nautilus", "dolphin", "thunar", "pcmanfm"]:
            try:
                subprocess.Popen([fm], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
                return "Opening file manager"
            except:
                continue
        return "Couldn't find file manager"
    
    def cmd_open_settings(self, text: str) -> str:
        subprocess.Popen(["gnome-control-center"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        return "Opening settings"
    
    def cmd_open_calculator(self, text: str) -> str:
        subprocess.Popen(["gnome-calculator"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        return "Opening calculator"
    
    def cmd_open_editor(self, text: str) -> str:
        for editor in ["gedit", "kate", "code", "nano"]:
            try:
                subprocess.Popen([editor], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
                return "Opening text editor"
            except:
                continue
        return "Couldn't find text editor"
    
    def cmd_volume_up(self, text: str) -> str:
        subprocess.run(["amixer", "-q", "set", "Master", "15%+"], capture_output=True)
        return "Volume up"
    
    def cmd_volume_down(self, text: str) -> str:
        subprocess.run(["amixer", "-q", "set", "Master", "15%-"], capture_output=True)
        return "Volume down"
    
    def cmd_mute(self, text: str) -> str:
        subprocess.run(["amixer", "-q", "set", "Master", "mute"], capture_output=True)
        return "Muted"
    
    def cmd_unmute(self, text: str) -> str:
        subprocess.run(["amixer", "-q", "set", "Master", "unmute"], capture_output=True)
        return "Unmuted"
    
    def cmd_screenshot(self, text: str) -> str:
        path = os.path.expanduser(f"~/Pictures/screenshot_{int(time.time())}.png")
        subprocess.run(["scrot", path], capture_output=True)
        return "Screenshot saved"
    
    def cmd_search(self, text: str) -> str:
        query = text.lower()
        for word in ["search", "google", "look up", "find", "for"]:
            query = query.replace(word, "")
        query = query.strip()
        if query:
            subprocess.Popen(["firefox", f"https://www.google.com/search?q={query}"])
            return f"Searching for {query}"
        return "What would you like me to search for?"
    
    def cmd_youtube(self, text: str) -> str:
        query = text.lower().replace("youtube", "").replace("play", "").replace("watch", "").strip()
        if query:
            subprocess.Popen(["firefox", f"https://www.youtube.com/results?search_query={query}"])
            return f"Searching YouTube for {query}"
        subprocess.Popen(["firefox", "https://www.youtube.com"])
        return "Opening YouTube"
    
    def cmd_weather(self, text: str) -> str:
        subprocess.Popen(["firefox", "https://weather.google.com"])
        return "Opening weather"
    
    def cmd_news(self, text: str) -> str:
        subprocess.Popen(["firefox", "https://news.google.com"])
        return "Opening news"
    
    def cmd_music(self, text: str) -> str:
        subprocess.Popen(["firefox", "https://open.spotify.com"])
        return "Opening Spotify"
    
    def cmd_note(self, text: str) -> str:
        content = text.lower()
        for word in ["note", "remember", "save"]:
            content = content.replace(word, "")
        content = content.strip()
        if content:
            self.memory.add_note("Quick Note", content)
            return "I'll remember that"
        return "What would you like me to note?"
    
    def cmd_reminder(self, text: str) -> str:
        # Simple reminder - default to 1 hour
        content = text.lower()
        for word in ["remind", "reminder", "me to", "set"]:
            content = content.replace(word, "")
        content = content.strip()
        if content:
            remind_time = datetime.now() + timedelta(hours=1)
            self.memory.add_reminder(content, remind_time)
            return f"I'll remind you in an hour: {content}"
        return "What should I remind you about?"
    
    def cmd_calendar(self, text: str) -> str:
        events = self.memory.get_upcoming_events(7)
        if events:
            return f"You have {len(events)} upcoming events. The next one is {events[0]['title']}"
        return "No upcoming events scheduled"
    
    def cmd_battery(self, text: str) -> str:
        try:
            import psutil
            battery = psutil.sensors_battery()
            if battery:
                status = "charging" if battery.power_plugged else "on battery"
                return f"Battery is at {int(battery.percent)}%, {status}"
        except:
            pass
        return "Couldn't get battery info"
    
    def cmd_system_info(self, text: str) -> str:
        try:
            import psutil
            cpu = psutil.cpu_percent()
            mem = psutil.virtual_memory().percent
            return f"CPU is at {cpu}%, memory at {mem}%"
        except:
            pass
        return "Couldn't get system info"
    
    def cmd_stop(self, text: str) -> str:
        return "Okay, I'll be quiet"
    
    def cmd_thanks(self, text: str) -> str:
        return "You're welcome!"
    
    def cmd_hello(self, text: str) -> str:
        return "Hey! How can I help you?"
    
    def cmd_help(self, text: str) -> str:
        return "I can help with time, apps, volume, search, notes, reminders, and answer questions. Just ask!"

# ═══════════════════════════════════════════════════════════════════════════
# MAIN ASSISTANT CLASS
# ═══════════════════════════════════════════════════════════════════════════

class Marceline:
    """Main Marceline voice assistant - simplified and robust."""
    
    def __init__(self):
        log.info("═" * 60)
        log.info("MARCELINE - Personal AI Assistant")
        log.info(f"Version {VERSION}")
        log.info("═" * 60)
        
        self.config = self._load_config()
        self.state = AssistantState.IDLE
        self.running = True
        
        # Initialize components
        self.memory = MemoryEngine()
        self.speech = SpeechEngine()  # New unified speech engine
        self.tts = TTSEngine(
            voice=self.config.get("voice", {}).get("voice", "en+f3"),
            rate=self.config.get("voice", {}).get("rate", 175)
        )
        self.ai = AIEngine(
            model=self.config.get("llm", {}).get("model", "llama3.2:3b")
        )
        self.commands = CommandProcessor(self.memory, self.ai, self.tts)
        
        # Wake words - includes computer
        self.wake_words = ["marceline", "marcy", "computer", "hey marceline", "hey marcy", "hey computer"]
        
        # Signal handling
        signal.signal(signal.SIGINT, self._shutdown)
        signal.signal(signal.SIGTERM, self._shutdown)
        
        log.info(f"Wake words: {self.wake_words}")
        log.info("Marceline initialized successfully")
    
    def _load_config(self) -> Dict:
        for path in [USER_CONFIG_PATH, CONFIG_PATH]:
            try:
                with open(path) as f:
                    return json.load(f)
            except:
                continue
        return {}
    
    def _shutdown(self, signum, frame):
        log.info("Shutting down...")
        self.running = False
        send_event("shutdown")
        sys.exit(0)
    
    def _contains_wake_word(self, text: str) -> bool:
        if not text:
            return False
        text_lower = text.lower()
        for wake in self.wake_words:
            if wake in text_lower:
                return True
        return False
    
    def _extract_command(self, text: str) -> str:
        """Extract command from text, removing wake words."""
        if not text:
            return ""
        cmd = text.lower()
        for wake in self.wake_words:
            cmd = cmd.replace(wake, "")
        return cmd.strip()
    
    def _check_reminders(self):
        """Check for pending reminders."""
        try:
            reminders = self.memory.get_pending_reminders()
            for reminder in reminders:
                self.tts.speak(f"Reminder: {reminder['message']}")
                self.memory.complete_reminder(reminder['id'])
        except:
            pass
    
    def _process_command(self, command: str):
        """Process a user command."""
        if not command or len(command) < 2:
            self.tts.speak("I didn't catch that. Try again?")
            send_event("error", message="No command")
            return
        
        log.info(f"Processing: {command}")
        self.state = AssistantState.PROCESSING
        send_event("processing", command=command)
        
        response = self.commands.process(command)
        
        self.state = AssistantState.SPEAKING
        self.tts.speak(response)
        send_event("result", command=command, response=response)
    
    def run(self):
        """Main loop - simple and reliable."""
        log.info("Starting main loop...")
        send_event("starting")
        
        # Calibrate mic
        log.info("Calibrating microphone...")
        self.speech.calibrate(duration=1.0)
        
        # Greeting
        greeting = "Hey! I'm Marceline. Say my name or say computer to wake me up!"
        self.tts.speak(greeting)
        send_event("ready", greeting=greeting)
        
        log.info("=" * 40)
        log.info("Listening for: marceline, marcy, or computer")
        log.info("=" * 40)
        
        last_reminder_check = time.time()
        
        while self.running:
            try:
                # Check reminders every minute
                if time.time() - last_reminder_check > 60:
                    self._check_reminders()
                    last_reminder_check = time.time()
                
                # Listen for wake word
                self.state = AssistantState.IDLE
                send_event("idle")
                
                detected, text = self.speech.listen_for_wake(self.wake_words, timeout=3.0)
                
                if detected:
                    log.info(f"Wake word detected! Text: {text}")
                    
                    # Check if command is included
                    command = self._extract_command(text)
                    
                    if len(command) > 3:
                        # Command was in same utterance
                        log.info(f"Command in wake phrase: {command}")
                        self._process_command(command)
                    else:
                        # Need to listen for command
                        self.tts.speak("Yes?")
                        self.state = AssistantState.LISTENING
                        send_event("listening")
                        
                        command = self.speech.listen_for_command(timeout=5.0)
                        self._process_command(command)
                    
                    log.info("Ready for next wake word...")
                
            except KeyboardInterrupt:
                break
            except Exception as e:
                log.error(f"Loop error: {e}")
                time.sleep(0.5)
        
        log.info("Marceline stopped")

# ═══════════════════════════════════════════════════════════════════════════
# ENTRY POINT
# ═══════════════════════════════════════════════════════════════════════════

if __name__ == "__main__":
    try:
        assistant = Marceline()
        assistant.run()
    except KeyboardInterrupt:
        log.info("Interrupted by user")
    except Exception as e:
        log.error(f"Fatal error: {e}")
        sys.exit(1)
ASSISTANTEOF
    
    chmod +x "$MAR_DIR/assistant-core/marceline.py"
    chown "$REAL_USER:$REAL_USER" "$MAR_DIR/assistant-core/marceline.py"
    
    log_success "Voice assistant created"
}

# ═══════════════════════════════════════════════════════════════════════════
# GUI OVERLAY
# ═══════════════════════════════════════════════════════════════════════════

create_gui_overlay() {
    log_step "Creating GUI overlay..."
    
    cat > "$MAR_DIR/assistant-core/overlay.py" <<'OVERLAYEOF'
#!/usr/bin/env python3
"""
═══════════════════════════════════════════════════════════════════════════════
MARCELINE - Modern GUI Overlay
═══════════════════════════════════════════════════════════════════════════════
A sleek, Gemini-like floating assistant UI with:
- Real-time status updates
- Conversation display
- Draggable window
- Smooth animations
═══════════════════════════════════════════════════════════════════════════════
"""

import json
import os
import time
import tkinter as tk
from tkinter import font as tkfont
from datetime import datetime

EVENT_FILE = "/tmp/marceline_event.json"

class Colors:
    BG_DARK = "#0f0f1a"
    BG_CARD = "#1a1a2e"
    TEXT_PRIMARY = "#ffffff"
    TEXT_SECONDARY = "#888888"
    ACCENT_GREEN = "#00ff88"
    ACCENT_RED = "#ff6b6b"
    ACCENT_YELLOW = "#ffd93d"
    ACCENT_BLUE = "#6bcbff"
    ACCENT_PURPLE = "#b388ff"
    BORDER = "#2a2a4a"

class MarcelineOverlay(tk.Tk):
    def __init__(self):
        super().__init__()
        
        # Window setup
        self.title("Marceline")
        self.geometry("380x140+1320+820")
        self.configure(bg=Colors.BG_DARK)
        self.overrideredirect(True)
        self.attributes("-topmost", True)
        self.attributes("-alpha", 0.95)
        
        # Dragging
        self.bind("<Button-1>", self.start_drag)
        self.bind("<B1-Motion>", self.do_drag)
        
        # Main container with border effect
        self.container = tk.Frame(self, bg=Colors.BORDER, padx=1, pady=1)
        self.container.pack(fill="both", expand=True)
        
        self.main = tk.Frame(self.container, bg=Colors.BG_CARD, padx=15, pady=12)
        self.main.pack(fill="both", expand=True)
        
        # Header row
        self.header = tk.Frame(self.main, bg=Colors.BG_CARD)
        self.header.pack(fill="x")
        
        # Status indicator
        self.status_dot = tk.Label(
            self.header, text="●", fg=Colors.ACCENT_GREEN,
            bg=Colors.BG_CARD, font=("Helvetica", 16)
        )
        self.status_dot.pack(side="left")
        
        # Status text
        self.status_text = tk.Label(
            self.header, text="Marceline ready",
            fg=Colors.TEXT_PRIMARY, bg=Colors.BG_CARD,
            font=("Helvetica", 12, "bold")
        )
        self.status_text.pack(side="left", padx=(8, 0))
        
        # Time
        self.time_label = tk.Label(
            self.header, text="",
            fg=Colors.TEXT_SECONDARY, bg=Colors.BG_CARD,
            font=("Helvetica", 10)
        )
        self.time_label.pack(side="right")
        
        # Separator
        tk.Frame(self.main, bg=Colors.BORDER, height=1).pack(fill="x", pady=10)
        
        # Response area
        self.response_frame = tk.Frame(self.main, bg=Colors.BG_CARD)
        self.response_frame.pack(fill="both", expand=True)
        
        self.response_label = tk.Label(
            self.response_frame,
            text='Say "Hey Marceline" to start',
            fg=Colors.TEXT_SECONDARY, bg=Colors.BG_CARD,
            font=("Helvetica", 11),
            wraplength=350, justify="left", anchor="w"
        )
        self.response_label.pack(fill="x")
        
        # Command echo (smaller, secondary)
        self.command_label = tk.Label(
            self.response_frame, text="",
            fg=Colors.TEXT_SECONDARY, bg=Colors.BG_CARD,
            font=("Helvetica", 9),
            wraplength=350, justify="left", anchor="w"
        )
        self.command_label.pack(fill="x", pady=(5, 0))
        
        # State
        self.last_event = {}
        self.animation_frame = 0
        
        # Start updates
        self.update_time()
        self.refresh()
    
    def start_drag(self, event):
        self.x = event.x
        self.y = event.y
    
    def do_drag(self, event):
        x = self.winfo_x() + event.x - self.x
        y = self.winfo_y() + event.y - self.y
        self.geometry(f"+{x}+{y}")
    
    def update_time(self):
        self.time_label.config(text=datetime.now().strftime("%H:%M"))
        self.after(1000, self.update_time)
    
    def set_status(self, text: str, color: str, response: str = None, command: str = None):
        self.status_dot.config(fg=color)
        self.status_text.config(text=text)
        
        if response:
            self.response_label.config(text=response, fg=Colors.TEXT_PRIMARY)
        
        if command:
            self.command_label.config(text=f"You said: {command}")
        else:
            self.command_label.config(text="")
    
    def refresh(self):
        try:
            if os.path.exists(EVENT_FILE):
                with open(EVENT_FILE) as f:
                    event = json.load(f)
                
                if event != self.last_event:
                    self.last_event = event
                    event_type = event.get("type", "")
                    
                    if event_type == "idle":
                        self.set_status("Ready", Colors.ACCENT_GREEN,
                                       'Say "Hey Marceline" to start')
                    
                    elif event_type == "listening":
                        self.set_status("Listening...", Colors.ACCENT_RED,
                                       "🎤 Speak now...")
                    
                    elif event_type == "processing":
                        cmd = event.get("command", "")[:60]
                        self.set_status("Thinking...", Colors.ACCENT_YELLOW,
                                       "Processing your request...", cmd)
                    
                    elif event_type == "speaking":
                        text = event.get("text", "")[:80]
                        self.set_status("Speaking", Colors.ACCENT_BLUE, text)
                    
                    elif event_type == "result":
                        response = event.get("response", "")[:80]
                        command = event.get("command", "")[:40]
                        self.set_status("Done", Colors.ACCENT_GREEN, response, command)
                    
                    elif event_type == "error":
                        msg = event.get("message", "Something went wrong")
                        self.set_status("Error", Colors.ACCENT_RED, msg)
                    
                    elif event_type == "shutdown":
                        self.set_status("Offline", Colors.TEXT_SECONDARY, "Marceline is offline")
        
        except Exception:
            pass
        
        self.after(250, self.refresh)

if __name__ == "__main__":
    app = MarcelineOverlay()
    app.mainloop()
OVERLAYEOF
    
    chmod +x "$MAR_DIR/assistant-core/overlay.py"
    chown "$REAL_USER:$REAL_USER" "$MAR_DIR/assistant-core/overlay.py"
    
    log_success "GUI overlay created"
}

# ═══════════════════════════════════════════════════════════════════════════
# SYSTEMD SERVICES
# ═══════════════════════════════════════════════════════════════════════════

create_systemd_services() {
    log_step "Creating systemd services..."
    
    # Voice assistant service
    cat > /etc/systemd/system/marceline-voice.service <<EOF
[Unit]
Description=Marceline Voice Assistant
After=network.target sound.target ollama.service
Wants=ollama.service

[Service]
Type=simple
User=$REAL_USER
Group=$REAL_USER
WorkingDirectory=$MAR_DIR
Environment=DISPLAY=:0
Environment=PULSE_SERVER=unix:/run/user/$REAL_UID/pulse/native
Environment=XDG_RUNTIME_DIR=/run/user/$REAL_UID
ExecStart=$MAR_DIR/venv/bin/python $MAR_DIR/assistant-core/marceline.py
Restart=on-failure
RestartSec=5
StandardOutput=append:/var/log/marceline/voice.log
StandardError=append:/var/log/marceline/voice.log

[Install]
WantedBy=multi-user.target
EOF
    
    # Overlay service
    cat > /etc/systemd/system/marceline-overlay.service <<EOF
[Unit]
Description=Marceline GUI Overlay
After=graphical.target marceline-voice.service
Wants=marceline-voice.service

[Service]
Type=simple
User=$REAL_USER
Group=$REAL_USER
WorkingDirectory=$MAR_DIR
Environment=DISPLAY=:0
Environment=XDG_RUNTIME_DIR=/run/user/$REAL_UID
ExecStart=$MAR_DIR/venv/bin/python $MAR_DIR/assistant-core/overlay.py
Restart=on-failure
RestartSec=5
StandardOutput=append:/var/log/marceline/overlay.log
StandardError=append:/var/log/marceline/overlay.log

[Install]
WantedBy=graphical.target
EOF
    
    # Reload and enable
    systemctl daemon-reload
    systemctl enable marceline-voice.service marceline-overlay.service
    
    log_success "Systemd services created"
}

# ═══════════════════════════════════════════════════════════════════════════
# AUTOSTART
# ═══════════════════════════════════════════════════════════════════════════

create_autostart() {
    log_step "Creating autostart entries..."
    
    mkdir -p "$REAL_HOME/.config/autostart"
    
    # Voice assistant autostart
    cat > "$REAL_HOME/.config/autostart/marceline-voice.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Marceline Voice Assistant
Comment=Personal AI Voice Assistant
Exec=$MAR_DIR/venv/bin/python $MAR_DIR/assistant-core/marceline.py
Terminal=false
X-GNOME-Autostart-enabled=true
X-GNOME-Autostart-Delay=10
Hidden=false
EOF
    
    # Overlay autostart
    cat > "$REAL_HOME/.config/autostart/marceline-overlay.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Marceline Overlay
Comment=Marceline GUI Overlay
Exec=$MAR_DIR/venv/bin/python $MAR_DIR/assistant-core/overlay.py
Terminal=false
X-GNOME-Autostart-enabled=true
X-GNOME-Autostart-Delay=12
Hidden=false
EOF
    
    chown "$REAL_USER:$REAL_USER" "$REAL_HOME/.config/autostart/marceline-voice.desktop"
    chown "$REAL_USER:$REAL_USER" "$REAL_HOME/.config/autostart/marceline-overlay.desktop"
    
    log_success "Autostart entries created"
}

# ═══════════════════════════════════════════════════════════════════════════
# CLI LAUNCHER
# ═══════════════════════════════════════════════════════════════════════════

create_launchers() {
    log_step "Creating command-line launchers..."
    
    # Main launcher
    cat > /usr/local/bin/marceline <<EOF
#!/usr/bin/env bash
cd "$MAR_DIR"
source venv/bin/activate
python assistant-core/marceline.py "\$@"
EOF
    chmod +x /usr/local/bin/marceline
    
    # Overlay launcher
    cat > /usr/local/bin/marceline-overlay <<EOF
#!/usr/bin/env bash
cd "$MAR_DIR"
source venv/bin/activate
python assistant-core/overlay.py
EOF
    chmod +x /usr/local/bin/marceline-overlay
    
    # Status command
    cat > /usr/local/bin/marceline-status <<EOF
#!/usr/bin/env bash
echo "=== Marceline Status ==="
echo ""
echo "Voice Service:"
systemctl status marceline-voice.service --no-pager -l | head -10
echo ""
echo "Overlay Service:"
systemctl status marceline-overlay.service --no-pager -l | head -10
EOF
    chmod +x /usr/local/bin/marceline-status
    
    # Control command
    cat > /usr/local/bin/marceline-ctl <<EOF
#!/usr/bin/env bash
case "\$1" in
    start)
        sudo systemctl start marceline-voice.service marceline-overlay.service
        echo "Marceline started"
        ;;
    stop)
        sudo systemctl stop marceline-voice.service marceline-overlay.service
        echo "Marceline stopped"
        ;;
    restart)
        sudo systemctl restart marceline-voice.service marceline-overlay.service
        echo "Marceline restarted"
        ;;
    status)
        marceline-status
        ;;
    logs)
        sudo journalctl -u marceline-voice.service -f
        ;;
    *)
        echo "Usage: marceline-ctl {start|stop|restart|status|logs}"
        exit 1
        ;;
esac
EOF
    chmod +x /usr/local/bin/marceline-ctl
    
    log_success "Launchers created"
}

# ═══════════════════════════════════════════════════════════════════════════
# AUDIO GROUP
# ═══════════════════════════════════════════════════════════════════════════

setup_audio_permissions() {
    log_step "Setting up audio permissions..."
    
    usermod -aG audio "$REAL_USER" 2>/dev/null || true
    usermod -aG pulse "$REAL_USER" 2>/dev/null || true
    usermod -aG pulse-access "$REAL_USER" 2>/dev/null || true
    
    log_success "Audio permissions configured"
}

# ═══════════════════════════════════════════════════════════════════════════
# START SERVICES
# ═══════════════════════════════════════════════════════════════════════════

start_services() {
    log_step "Starting Marceline services..."
    
    systemctl restart marceline-voice.service || log_warning "Voice service may need reboot"
    systemctl restart marceline-overlay.service || log_warning "Overlay service may need reboot"
    
    log_success "Services started"
}

# ═══════════════════════════════════════════════════════════════════════════
# SHOW COMPLETION MESSAGE
# ═══════════════════════════════════════════════════════════════════════════

show_completion() {
    echo ""
    echo -e "${GREEN}"
    echo "╔═══════════════════════════════════════════════════════════════════════════╗"
    echo "║                                                                           ║"
    echo "║   ✅ MARCELINE INSTALLED SUCCESSFULLY!                                    ║"
    echo "║                                                                           ║"
    echo "╠═══════════════════════════════════════════════════════════════════════════╣"
    echo "║                                                                           ║"
    echo "║   🎤 VOICE COMMANDS:                                                      ║"
    echo "║      Say 'Hey Marceline' or 'Marcy' to activate                           ║"
    echo "║                                                                           ║"
    echo "║   📋 WHAT YOU CAN ASK:                                                    ║"
    echo "║      • 'What time is it?'                                                 ║"
    echo "║      • 'Open Firefox / Terminal / Files'                                  ║"
    echo "║      • 'Search for something'                                             ║"
    echo "║      • 'Take a screenshot'                                                ║"
    echo "║      • 'Volume up / down'                                                 ║"
    echo "║      • 'Remind me to...'                                                  ║"
    echo "║      • 'Note: remember this...'                                           ║"
    echo "║      • Any question - she uses AI to answer!                              ║"
    echo "║                                                                           ║"
    echo "║   🔧 COMMANDS:                                                            ║"
    echo "║      marceline         - Run voice assistant manually                     ║"
    echo "║      marceline-overlay - Show GUI overlay                                 ║"
    echo "║      marceline-status  - Check service status                             ║"
    echo "║      marceline-ctl     - Control services (start/stop/restart)            ║"
    echo "║                                                                           ║"
    echo "║   📁 FILES:                                                               ║"
    echo "║      Config: $MAR_DIR/config/config.json                ║"
    echo "║      Logs:   /var/log/marceline/                                          ║"
    echo "║                                                                           ║"
    echo "╠═══════════════════════════════════════════════════════════════════════════╣"
    echo "║                                                                           ║"
    echo "║   🔁 REBOOT NOW FOR AUTO-START:                                           ║"
    echo "║      sudo reboot                                                          ║"
    echo "║                                                                           ║"
    echo "╚═══════════════════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

# ═══════════════════════════════════════════════════════════════════════════
# MCP SERVER FRAMEWORK
# ═══════════════════════════════════════════════════════════════════════════

create_mcp_servers() {
    log_step "Creating MCP Server Framework..."
    
    # MCP Router
    cat > "$MAR_DIR/mcp-servers/router.py" <<'MCPROUTER'
#!/usr/bin/env python3
"""MCP Server Router - Routes requests to appropriate servers"""
import json
import asyncio
from flask import Flask, request, jsonify
from flask_cors import CORS
from typing import Dict, Any
import importlib
import os
import sys

app = Flask(__name__)
CORS(app)

class MCPRouter:
    def __init__(self):
        self.servers = {}
        self._load_servers()
    
    def _load_servers(self):
        server_dir = os.path.dirname(__file__)
        for name in ['file', 'system', 'clipboard', 'notification', 'websearch', 
                     'textgen', 'window', 'browser', 'calendar', 'notes']:
            try:
                module = importlib.import_module(f'mcp_servers.{name}_server')
                self.servers[name] = getattr(module, f'{name.title()}Server')()
                print(f"Loaded MCP server: {name}")
            except Exception as e:
                print(f"Could not load {name} server: {e}")
    
    def call(self, server: str, tool: str, params: Dict = None) -> Dict[str, Any]:
        if server not in self.servers:
            return {"success": False, "error": f"Unknown server: {server}"}
        try:
            srv = self.servers[server]
            if not hasattr(srv, tool):
                return {"success": False, "error": f"Unknown tool: {tool}"}
            result = getattr(srv, tool)(**(params or {}))
            return {"success": True, "data": result}
        except Exception as e:
            return {"success": False, "error": str(e)}

router = MCPRouter()

@app.route('/health', methods=['GET'])
def health():
    return jsonify({"status": "ok", "servers": list(router.servers.keys())})

@app.route('/call', methods=['POST'])
def call_tool():
    data = request.json
    result = router.call(data.get('server'), data.get('tool'), data.get('params'))
    return jsonify(result)

@app.route('/servers', methods=['GET'])
def list_servers():
    return jsonify({"servers": list(router.servers.keys())})

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=8080)
MCPROUTER

    # File Server
    cat > "$MAR_DIR/mcp-servers/file_server.py" <<'FILSERVER'
#!/usr/bin/env python3
"""MCP File Server - File system operations"""
import os
import shutil
import json
from pathlib import Path
from typing import List, Dict

class FileServer:
    def list_directory(self, path: str = ".") -> List[Dict]:
        items = []
        for item in Path(path).iterdir():
            items.append({
                "name": item.name,
                "type": "directory" if item.is_dir() else "file",
                "size": item.stat().st_size if item.is_file() else 0,
                "path": str(item.absolute())
            })
        return items
    
    def read_file(self, path: str) -> str:
        return Path(path).read_text()
    
    def write_file(self, path: str, content: str) -> bool:
        Path(path).write_text(content)
        return True
    
    def create_directory(self, path: str) -> bool:
        Path(path).mkdir(parents=True, exist_ok=True)
        return True
    
    def delete(self, path: str) -> bool:
        p = Path(path)
        if p.is_dir():
            shutil.rmtree(p)
        else:
            p.unlink()
        return True
    
    def copy(self, src: str, dst: str) -> bool:
        if Path(src).is_dir():
            shutil.copytree(src, dst)
        else:
            shutil.copy2(src, dst)
        return True
    
    def move(self, src: str, dst: str) -> bool:
        shutil.move(src, dst)
        return True
    
    def search(self, path: str, pattern: str) -> List[str]:
        return [str(p) for p in Path(path).rglob(pattern)]
FILSERVER

    # System Server
    cat > "$MAR_DIR/mcp-servers/system_server.py" <<'SYSSERVER'
#!/usr/bin/env python3
"""MCP System Server - System operations"""
import subprocess
import psutil
import platform
import os
from datetime import datetime

class SystemServer:
    def get_info(self) -> dict:
        return {
            "os": platform.system(),
            "release": platform.release(),
            "hostname": platform.node(),
            "cpu_count": psutil.cpu_count(),
            "memory_total": psutil.virtual_memory().total,
            "uptime": datetime.now().timestamp() - psutil.boot_time()
        }
    
    def get_cpu(self) -> dict:
        return {"percent": psutil.cpu_percent(interval=1), "count": psutil.cpu_count()}
    
    def get_memory(self) -> dict:
        m = psutil.virtual_memory()
        return {"total": m.total, "used": m.used, "percent": m.percent}
    
    def get_disk(self, path: str = "/") -> dict:
        d = psutil.disk_usage(path)
        return {"total": d.total, "used": d.used, "percent": d.percent}
    
    def get_battery(self) -> dict:
        b = psutil.sensors_battery()
        if b:
            return {"percent": b.percent, "plugged": b.power_plugged}
        return {"error": "No battery"}
    
    def get_processes(self, limit: int = 10) -> list:
        procs = []
        for p in psutil.process_iter(['pid', 'name', 'cpu_percent', 'memory_percent']):
            procs.append(p.info)
        return sorted(procs, key=lambda x: x.get('cpu_percent', 0), reverse=True)[:limit]
    
    def run_command(self, command: str, timeout: int = 30) -> dict:
        try:
            result = subprocess.run(command, shell=True, capture_output=True, 
                                   text=True, timeout=timeout)
            return {"stdout": result.stdout, "stderr": result.stderr, "code": result.returncode}
        except subprocess.TimeoutExpired:
            return {"error": "Command timed out"}
    
    def open_app(self, app: str) -> bool:
        subprocess.Popen([app], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        return True
    
    def set_volume(self, level: int) -> bool:
        subprocess.run(["amixer", "-q", "set", "Master", f"{level}%"], capture_output=True)
        return True
    
    def get_volume(self) -> int:
        result = subprocess.run(["amixer", "get", "Master"], capture_output=True, text=True)
        for line in result.stdout.split('\n'):
            if '%' in line:
                import re
                match = re.search(r'\[(\d+)%\]', line)
                if match:
                    return int(match.group(1))
        return 50
SYSSERVER

    # Clipboard Server
    cat > "$MAR_DIR/mcp-servers/clipboard_server.py" <<'CLIPSERVER'
#!/usr/bin/env python3
"""MCP Clipboard Server"""
import subprocess

class ClipboardServer:
    def get(self) -> str:
        result = subprocess.run(["xclip", "-selection", "clipboard", "-o"], 
                               capture_output=True, text=True)
        return result.stdout
    
    def set(self, content: str) -> bool:
        process = subprocess.Popen(["xclip", "-selection", "clipboard"], 
                                  stdin=subprocess.PIPE)
        process.communicate(content.encode())
        return True
    
    def clear(self) -> bool:
        return self.set("")
CLIPSERVER

    # Notification Server
    cat > "$MAR_DIR/mcp-servers/notification_server.py" <<'NOTIFSERVER'
#!/usr/bin/env python3
"""MCP Notification Server"""
import subprocess

class NotificationServer:
    def send(self, title: str, message: str, urgency: str = "normal") -> bool:
        subprocess.run(["notify-send", "-u", urgency, title, message])
        return True
    
    def alert(self, message: str) -> bool:
        return self.send("Marceline Alert", message, "critical")
NOTIFSERVER

    # Web Search Server
    cat > "$MAR_DIR/mcp-servers/websearch_server.py" <<'SEARCHSERVER'
#!/usr/bin/env python3
"""MCP Web Search Server"""
try:
    from duckduckgo_search import DDGS
except:
    DDGS = None
import subprocess

class WebsearchServer:
    def search(self, query: str, limit: int = 5) -> list:
        if DDGS:
            try:
                with DDGS() as ddgs:
                    return list(ddgs.text(query, max_results=limit))
            except:
                pass
        return [{"title": "Search externally", "href": f"https://duckduckgo.com/?q={query}"}]
    
    def open_url(self, url: str) -> bool:
        subprocess.Popen(["firefox", url], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        return True
    
    def google(self, query: str) -> bool:
        return self.open_url(f"https://www.google.com/search?q={query}")
SEARCHSERVER

    # Text Generation Server
    cat > "$MAR_DIR/mcp-servers/textgen_server.py" <<'TEXTGENSERVER'
#!/usr/bin/env python3
"""MCP Text Generation Server - Ollama integration"""
import requests

OLLAMA_URL = "http://localhost:11434/api/generate"

class TextgenServer:
    def __init__(self, model: str = "llama3.2:3b"):
        self.model = model
    
    def generate(self, prompt: str, max_tokens: int = 150) -> dict:
        try:
            response = requests.post(OLLAMA_URL, json={
                "model": self.model,
                "prompt": prompt,
                "stream": False,
                "options": {"num_predict": max_tokens}
            }, timeout=60)
            if response.status_code == 200:
                return {"text": response.json().get("response", "")}
        except Exception as e:
            return {"error": str(e)}
        return {"error": "Generation failed"}
    
    def summarize(self, text: str) -> dict:
        return self.generate(f"Summarize this in 2-3 sentences:\n\n{text}")
    
    def translate(self, text: str, target_lang: str = "Spanish") -> dict:
        return self.generate(f"Translate to {target_lang}:\n\n{text}")
    
    def rewrite(self, text: str, style: str = "professional") -> dict:
        return self.generate(f"Rewrite this in a {style} style:\n\n{text}")
TEXTGENSERVER

    # Window Server
    cat > "$MAR_DIR/mcp-servers/window_server.py" <<'WINSERVER'
#!/usr/bin/env python3
"""MCP Window Server - Window management"""
import subprocess
import re

class WindowServer:
    def list_windows(self) -> list:
        result = subprocess.run(["wmctrl", "-l"], capture_output=True, text=True)
        windows = []
        for line in result.stdout.strip().split('\n'):
            if line:
                parts = line.split(None, 3)
                if len(parts) >= 4:
                    windows.append({"id": parts[0], "desktop": parts[1], "title": parts[3]})
        return windows
    
    def focus(self, window_id: str) -> bool:
        subprocess.run(["wmctrl", "-i", "-a", window_id])
        return True
    
    def close(self, window_id: str) -> bool:
        subprocess.run(["wmctrl", "-i", "-c", window_id])
        return True
    
    def minimize(self, window_id: str) -> bool:
        subprocess.run(["xdotool", "windowminimize", window_id])
        return True
    
    def maximize(self, window_id: str) -> bool:
        subprocess.run(["wmctrl", "-i", "-r", window_id, "-b", "add,maximized_vert,maximized_horz"])
        return True
WINSERVER

    # Calendar Server  
    cat > "$MAR_DIR/mcp-servers/calendar_server.py" <<'CALSERVER'
#!/usr/bin/env python3
"""MCP Calendar Server"""
import sqlite3
import os
from datetime import datetime, timedelta

DB_PATH = os.path.expanduser("~/marceline/data/memory/marceline.db")

class CalendarServer:
    def add_event(self, title: str, start: str, end: str = None, 
                  description: str = "", location: str = "") -> int:
        conn = sqlite3.connect(DB_PATH)
        c = conn.cursor()
        c.execute("""INSERT INTO calendar_events (title, description, start_time, end_time, location)
                     VALUES (?, ?, ?, ?, ?)""", (title, description, start, end, location))
        event_id = c.lastrowid
        conn.commit()
        conn.close()
        return event_id
    
    def get_events(self, days: int = 7) -> list:
        conn = sqlite3.connect(DB_PATH)
        c = conn.cursor()
        now = datetime.now().isoformat()
        future = (datetime.now() + timedelta(days=days)).isoformat()
        c.execute("""SELECT id, title, start_time, end_time, location FROM calendar_events
                     WHERE start_time >= ? AND start_time <= ? ORDER BY start_time""", (now, future))
        rows = c.fetchall()
        conn.close()
        return [{"id": r[0], "title": r[1], "start": r[2], "end": r[3], "location": r[4]} for r in rows]
    
    def delete_event(self, event_id: int) -> bool:
        conn = sqlite3.connect(DB_PATH)
        c = conn.cursor()
        c.execute("DELETE FROM calendar_events WHERE id = ?", (event_id,))
        conn.commit()
        conn.close()
        return True
CALSERVER

    # Notes Server
    cat > "$MAR_DIR/mcp-servers/notes_server.py" <<'NOTESERVER'
#!/usr/bin/env python3
"""MCP Notes Server"""
import sqlite3
import os
from datetime import datetime

DB_PATH = os.path.expanduser("~/marceline/data/memory/marceline.db")

class NotesServer:
    def add(self, title: str, content: str, tags: str = "") -> int:
        conn = sqlite3.connect(DB_PATH)
        c = conn.cursor()
        c.execute("INSERT INTO notes (title, content, tags) VALUES (?, ?, ?)", 
                  (title, content, tags))
        note_id = c.lastrowid
        conn.commit()
        conn.close()
        return note_id
    
    def get(self, note_id: int) -> dict:
        conn = sqlite3.connect(DB_PATH)
        c = conn.cursor()
        c.execute("SELECT id, title, content, tags FROM notes WHERE id = ?", (note_id,))
        row = c.fetchone()
        conn.close()
        if row:
            return {"id": row[0], "title": row[1], "content": row[2], "tags": row[3]}
        return {}
    
    def search(self, query: str) -> list:
        conn = sqlite3.connect(DB_PATH)
        c = conn.cursor()
        c.execute("SELECT id, title, content FROM notes WHERE title LIKE ? OR content LIKE ?",
                  (f"%{query}%", f"%{query}%"))
        rows = c.fetchall()
        conn.close()
        return [{"id": r[0], "title": r[1], "content": r[2][:100]} for r in rows]
    
    def list_all(self, limit: int = 20) -> list:
        conn = sqlite3.connect(DB_PATH)
        c = conn.cursor()
        c.execute("SELECT id, title, created_at FROM notes ORDER BY created_at DESC LIMIT ?", (limit,))
        rows = c.fetchall()
        conn.close()
        return [{"id": r[0], "title": r[1], "created": r[2]} for r in rows]
    
    def delete(self, note_id: int) -> bool:
        conn = sqlite3.connect(DB_PATH)
        c = conn.cursor()
        c.execute("DELETE FROM notes WHERE id = ?", (note_id,))
        conn.commit()
        conn.close()
        return True
NOTESERVER

    # Set permissions
    chmod +x "$MAR_DIR"/mcp-servers/*.py
    chown -R "$REAL_USER:$REAL_USER" "$MAR_DIR/mcp-servers"
    
    log_success "MCP servers created (10 servers)"
}

# ═══════════════════════════════════════════════════════════════════════════
# WEB API
# ═══════════════════════════════════════════════════════════════════════════

create_web_api() {
    log_step "Creating Web API..."
    
    cat > "$MAR_DIR/assistant-core/api.py" <<'WEBAPI'
#!/usr/bin/env python3
"""Marceline Web API - REST interface"""
from flask import Flask, request, jsonify, render_template_string
from flask_cors import CORS
import json
import os
import sys
sys.path.insert(0, os.path.dirname(__file__))

app = Flask(__name__)
CORS(app)

EVENT_FILE = "/tmp/marceline_event.json"
CONFIG_PATH = os.path.expanduser("~/marceline/config/config.json")

HTML_TEMPLATE = '''
<!DOCTYPE html>
<html>
<head>
    <title>Marceline - Web Interface</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
               background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
               min-height: 100vh; color: #fff; }
        .container { max-width: 800px; margin: 0 auto; padding: 40px 20px; }
        h1 { font-size: 2.5rem; margin-bottom: 10px; color: #00ff88; }
        .subtitle { color: #888; margin-bottom: 40px; }
        .card { background: rgba(255,255,255,0.05); border-radius: 16px;
                padding: 24px; margin-bottom: 20px; border: 1px solid rgba(255,255,255,0.1); }
        .status { display: flex; align-items: center; gap: 12px; }
        .dot { width: 12px; height: 12px; border-radius: 50%; background: #00ff88; }
        .dot.listening { background: #ff6b6b; animation: pulse 1s infinite; }
        @keyframes pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.5; } }
        input { width: 100%; padding: 16px; border-radius: 12px; border: none;
                background: rgba(255,255,255,0.1); color: #fff; font-size: 1rem;
                margin-top: 20px; }
        input:focus { outline: 2px solid #00ff88; }
        button { padding: 16px 32px; border-radius: 12px; border: none;
                 background: #00ff88; color: #000; font-weight: bold; cursor: pointer;
                 margin-top: 10px; }
        button:hover { background: #00cc6a; }
        .response { margin-top: 20px; padding: 16px; background: rgba(0,255,136,0.1);
                    border-radius: 12px; display: none; }
        .commands { display: grid; gap: 10px; margin-top: 20px; }
        .cmd { padding: 12px 16px; background: rgba(255,255,255,0.05); border-radius: 8px;
               cursor: pointer; transition: background 0.2s; }
        .cmd:hover { background: rgba(255,255,255,0.1); }
    </style>
</head>
<body>
    <div class="container">
        <h1>🌙 Marceline</h1>
        <p class="subtitle">Personal AI Assistant - Web Interface</p>
        
        <div class="card">
            <div class="status">
                <div class="dot" id="statusDot"></div>
                <span id="statusText">Ready</span>
            </div>
        </div>
        
        <div class="card">
            <h3>Ask Marceline</h3>
            <input type="text" id="query" placeholder="Type your question...">
            <button onclick="ask()">Send</button>
            <div class="response" id="response"></div>
        </div>
        
        <div class="card">
            <h3>Quick Commands</h3>
            <div class="commands">
                <div class="cmd" onclick="sendCmd('What time is it?')">🕐 What time is it?</div>
                <div class="cmd" onclick="sendCmd('Take a screenshot')">📸 Take screenshot</div>
                <div class="cmd" onclick="sendCmd('Volume up')">🔊 Volume up</div>
                <div class="cmd" onclick="sendCmd('Open Firefox')">🌐 Open Firefox</div>
            </div>
        </div>
    </div>
    <script>
        function ask() {
            const q = document.getElementById('query').value;
            if (!q) return;
            fetch('/api/ask', {method: 'POST', headers: {'Content-Type': 'application/json'},
                body: JSON.stringify({query: q})})
                .then(r => r.json())
                .then(d => {
                    document.getElementById('response').style.display = 'block';
                    document.getElementById('response').textContent = d.response || d.error;
                });
        }
        function sendCmd(cmd) {
            document.getElementById('query').value = cmd;
            ask();
        }
        function updateStatus() {
            fetch('/api/status').then(r => r.json()).then(d => {
                document.getElementById('statusText').textContent = d.status || 'Ready';
                document.getElementById('statusDot').className = 
                    'dot' + (d.status === 'Listening' ? ' listening' : '');
            }).catch(() => {});
            setTimeout(updateStatus, 1000);
        }
        updateStatus();
    </script>
</body>
</html>
'''

@app.route('/')
def index():
    return render_template_string(HTML_TEMPLATE)

@app.route('/api/status')
def status():
    try:
        with open(EVENT_FILE) as f:
            event = json.load(f)
        return jsonify({"status": event.get("type", "idle").title()})
    except:
        return jsonify({"status": "Ready"})

@app.route('/api/ask', methods=['POST'])
def ask():
    data = request.json
    query = data.get('query', '')
    if not query:
        return jsonify({"error": "No query provided"})
    
    # Import and use command processor
    try:
        from marceline import Marceline
        m = Marceline()
        response = m.commands.process(query)
        return jsonify({"response": response})
    except Exception as e:
        return jsonify({"error": str(e)})

@app.route('/api/config')
def get_config():
    try:
        with open(CONFIG_PATH) as f:
            return jsonify(json.load(f))
    except:
        return jsonify({})

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
WEBAPI

    chmod +x "$MAR_DIR/assistant-core/api.py"
    chown "$REAL_USER:$REAL_USER" "$MAR_DIR/assistant-core/api.py"
    
    # Web API service
    cat > /etc/systemd/system/marceline-api.service <<EOF
[Unit]
Description=Marceline Web API
After=network.target

[Service]
User=$REAL_USER
Group=$REAL_USER
WorkingDirectory=$MAR_DIR
ExecStart=$MAR_DIR/venv/bin/python $MAR_DIR/assistant-core/api.py
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

    systemctl daemon-reload
    systemctl enable marceline-api.service
    
    log_success "Web API created (http://localhost:5000)"
}

# ═══════════════════════════════════════════════════════════════════════════
# PLUGIN SYSTEM
# ═══════════════════════════════════════════════════════════════════════════

create_plugin_system() {
    log_step "Creating plugin system..."
    
    # Plugin loader
    cat > "$MAR_DIR/plugins/__init__.py" <<'PLUGINLOADER'
#!/usr/bin/env python3
"""Marceline Plugin System - Load and manage plugins"""
import os
import json
import importlib.util
from pathlib import Path
from typing import Dict, List, Any, Callable
from dataclasses import dataclass

PLUGIN_DIR = Path(__file__).parent
CONFIG_FILE = PLUGIN_DIR / "plugins.json"

@dataclass
class Plugin:
    name: str
    version: str
    description: str
    author: str
    enabled: bool
    commands: Dict[str, Callable]
    hooks: Dict[str, Callable]

class PluginManager:
    def __init__(self):
        self.plugins: Dict[str, Plugin] = {}
        self.commands: Dict[str, Callable] = {}
        self.hooks: Dict[str, List[Callable]] = {
            "on_wake": [], "on_command": [], "on_response": [],
            "on_startup": [], "on_shutdown": []
        }
        self._load_plugins()
    
    def _load_plugins(self):
        if not PLUGIN_DIR.exists():
            return
        for item in PLUGIN_DIR.iterdir():
            if item.is_dir() and (item / "plugin.py").exists():
                self._load_plugin(item)
    
    def _load_plugin(self, path: Path):
        try:
            spec = importlib.util.spec_from_file_location(
                path.name, path / "plugin.py"
            )
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
            
            if hasattr(module, "register"):
                plugin = module.register(self)
                self.plugins[path.name] = plugin
                print(f"Loaded plugin: {path.name}")
        except Exception as e:
            print(f"Failed to load plugin {path.name}: {e}")
    
    def register_command(self, trigger: str, handler: Callable):
        self.commands[trigger] = handler
    
    def register_hook(self, hook_name: str, handler: Callable):
        if hook_name in self.hooks:
            self.hooks[hook_name].append(handler)
    
    def call_hook(self, hook_name: str, *args, **kwargs):
        results = []
        for handler in self.hooks.get(hook_name, []):
            try:
                results.append(handler(*args, **kwargs))
            except Exception as e:
                print(f"Hook error: {e}")
        return results
    
    def get_command(self, text: str) -> Callable:
        for trigger, handler in self.commands.items():
            if trigger in text.lower():
                return handler
        return None

manager = PluginManager()
PLUGINLOADER

    # Example plugin: Weather
    mkdir -p "$MAR_DIR/plugins/weather"
    cat > "$MAR_DIR/plugins/weather/plugin.py" <<'WEATHERPLUGIN'
#!/usr/bin/env python3
"""Weather Plugin for Marceline"""
import subprocess
import requests

def get_weather(location: str = "auto") -> str:
    try:
        # Use wttr.in for simple weather
        url = f"https://wttr.in/{location}?format=3"
        response = requests.get(url, timeout=5)
        if response.status_code == 200:
            return response.text.strip()
    except:
        pass
    return "Couldn't fetch weather. Try opening weather.google.com"

def open_weather(text: str) -> str:
    subprocess.Popen(["firefox", "https://weather.google.com"], 
                     stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    return "Opening weather"

def register(manager):
    manager.register_command("weather", lambda t: get_weather())
    manager.register_command("forecast", open_weather)
    return {
        "name": "Weather",
        "version": "1.0.0",
        "description": "Get weather information",
        "author": "Marceline Team"
    }
WEATHERPLUGIN

    # Example plugin: Calculator
    mkdir -p "$MAR_DIR/plugins/calculator"
    cat > "$MAR_DIR/plugins/calculator/plugin.py" <<'CALCPLUGIN'
#!/usr/bin/env python3
"""Calculator Plugin for Marceline"""
import re
import math

def calculate(text: str) -> str:
    # Extract math expression
    expr = text.lower()
    for word in ["calculate", "what is", "compute", "solve", "="]:
        expr = expr.replace(word, "")
    expr = expr.strip()
    
    if not expr:
        return "What would you like me to calculate?"
    
    try:
        # Safe eval with math functions
        allowed = {"__builtins__": {}, "math": math, "abs": abs, 
                   "round": round, "min": min, "max": max, "sum": sum,
                   "pow": pow, "sqrt": math.sqrt, "sin": math.sin,
                   "cos": math.cos, "tan": math.tan, "pi": math.pi}
        result = eval(expr, allowed)
        return f"The answer is {result}"
    except Exception as e:
        return f"I couldn't calculate that: {e}"

def register(manager):
    manager.register_command("calculate", calculate)
    manager.register_command("what is", calculate)
    manager.register_command("compute", calculate)
    return {
        "name": "Calculator",
        "version": "1.0.0",
        "description": "Math calculations",
        "author": "Marceline Team"
    }
CALCPLUGIN

    # Example plugin: Timer
    mkdir -p "$MAR_DIR/plugins/timer"
    cat > "$MAR_DIR/plugins/timer/plugin.py" <<'TIMERPLUGIN'
#!/usr/bin/env python3
"""Timer Plugin for Marceline"""
import threading
import time
import subprocess
import re

active_timers = []

def notify(message: str):
    subprocess.run(["notify-send", "-u", "critical", "Marceline Timer", message])
    subprocess.run(["espeak-ng", message], capture_output=True)

def set_timer(text: str) -> str:
    # Parse time from text
    match = re.search(r'(\d+)\s*(second|minute|hour|min|sec|hr)s?', text.lower())
    if not match:
        return "Please specify a time, like 'set timer for 5 minutes'"
    
    amount = int(match.group(1))
    unit = match.group(2)
    
    if unit.startswith("min"):
        seconds = amount * 60
    elif unit.startswith("hour") or unit.startswith("hr"):
        seconds = amount * 3600
    else:
        seconds = amount
    
    def timer_callback():
        time.sleep(seconds)
        notify(f"Timer complete! {amount} {unit}s have passed.")
    
    thread = threading.Thread(target=timer_callback, daemon=True)
    thread.start()
    active_timers.append(thread)
    
    return f"Timer set for {amount} {unit}s"

def register(manager):
    manager.register_command("timer", set_timer)
    manager.register_command("set timer", set_timer)
    manager.register_command("alarm", set_timer)
    return {
        "name": "Timer",
        "version": "1.0.0",
        "description": "Set timers and alarms",
        "author": "Marceline Team"
    }
TIMERPLUGIN

    # Example plugin: Jokes
    mkdir -p "$MAR_DIR/plugins/jokes"
    cat > "$MAR_DIR/plugins/jokes/plugin.py" <<'JOKEPLUGIN'
#!/usr/bin/env python3
"""Jokes Plugin for Marceline"""
import random
import requests

JOKES = [
    "Why do programmers prefer dark mode? Because light attracts bugs!",
    "Why did the developer go broke? Because he used up all his cache!",
    "There are only 10 types of people: those who understand binary and those who don't.",
    "A SQL query walks into a bar, walks up to two tables and asks... 'Can I join you?'",
    "Why do Java developers wear glasses? Because they can't C#!",
    "What's a programmer's favorite hangout place? Foo Bar!",
    "Why was the JavaScript developer sad? Because he didn't Node how to Express himself!",
    "How many programmers does it take to change a light bulb? None, that's a hardware problem!",
]

def tell_joke(text: str) -> str:
    # Try to get a joke from API first
    try:
        response = requests.get("https://official-joke-api.appspot.com/random_joke", timeout=3)
        if response.status_code == 200:
            joke = response.json()
            return f"{joke['setup']} ... {joke['punchline']}"
    except:
        pass
    return random.choice(JOKES)

def register(manager):
    manager.register_command("joke", tell_joke)
    manager.register_command("tell me a joke", tell_joke)
    manager.register_command("make me laugh", tell_joke)
    return {
        "name": "Jokes",
        "version": "1.0.0", 
        "description": "Tell random jokes",
        "author": "Marceline Team"
    }
JOKEPLUGIN

    chown -R "$REAL_USER:$REAL_USER" "$MAR_DIR/plugins"
    log_success "Plugin system created (4 plugins)"
}

# ═══════════════════════════════════════════════════════════════════════════
# THEME SYSTEM
# ═══════════════════════════════════════════════════════════════════════════

create_theme_system() {
    log_step "Creating theme system..."
    
    # Theme manager
    cat > "$MAR_DIR/themes/theme_manager.py" <<'THEMEMGR'
#!/usr/bin/env python3
"""Marceline Theme Manager"""
import json
import os
from pathlib import Path
from typing import Dict, Any

THEMES_DIR = Path(__file__).parent
ACTIVE_THEME_FILE = THEMES_DIR / "active.json"

class ThemeManager:
    def __init__(self):
        self.themes = self._load_themes()
        self.active = self._load_active()
    
    def _load_themes(self) -> Dict[str, Dict]:
        themes = {}
        for f in THEMES_DIR.glob("*.json"):
            if f.name != "active.json":
                try:
                    themes[f.stem] = json.loads(f.read_text())
                except:
                    pass
        return themes
    
    def _load_active(self) -> str:
        try:
            data = json.loads(ACTIVE_THEME_FILE.read_text())
            return data.get("theme", "dark")
        except:
            return "dark"
    
    def get_theme(self, name: str = None) -> Dict:
        name = name or self.active
        return self.themes.get(name, self.themes.get("dark", {}))
    
    def set_theme(self, name: str) -> bool:
        if name in self.themes:
            self.active = name
            ACTIVE_THEME_FILE.write_text(json.dumps({"theme": name}))
            return True
        return False
    
    def list_themes(self) -> list:
        return list(self.themes.keys())

manager = ThemeManager()
THEMEMGR

    # Dark theme (default)
    cat > "$MAR_DIR/themes/dark.json" <<'DARKTHEME'
{
    "name": "Dark",
    "author": "Marceline Team",
    "colors": {
        "bg_primary": "#0f0f1a",
        "bg_secondary": "#1a1a2e",
        "bg_card": "#16213e",
        "text_primary": "#ffffff",
        "text_secondary": "#888888",
        "accent": "#00ff88",
        "accent_secondary": "#6bcbff",
        "error": "#ff6b6b",
        "warning": "#ffd93d",
        "success": "#00ff88",
        "border": "#2a2a4a"
    },
    "fonts": {
        "primary": "Helvetica",
        "monospace": "Consolas"
    },
    "overlay": {
        "width": 380,
        "height": 140,
        "opacity": 0.95,
        "position": "bottom-right",
        "border_radius": 16
    }
}
DARKTHEME

    # Light theme
    cat > "$MAR_DIR/themes/light.json" <<'LIGHTTHEME'
{
    "name": "Light",
    "author": "Marceline Team",
    "colors": {
        "bg_primary": "#f5f5f5",
        "bg_secondary": "#ffffff",
        "bg_card": "#ffffff",
        "text_primary": "#1a1a1a",
        "text_secondary": "#666666",
        "accent": "#0066cc",
        "accent_secondary": "#00aa88",
        "error": "#cc0000",
        "warning": "#cc8800",
        "success": "#00aa44",
        "border": "#dddddd"
    },
    "fonts": {
        "primary": "Helvetica",
        "monospace": "Consolas"
    },
    "overlay": {
        "width": 380,
        "height": 140,
        "opacity": 0.98,
        "position": "bottom-right",
        "border_radius": 16
    }
}
LIGHTTHEME

    # Cyberpunk theme
    cat > "$MAR_DIR/themes/cyberpunk.json" <<'CYBERTHEME'
{
    "name": "Cyberpunk",
    "author": "Marceline Team",
    "colors": {
        "bg_primary": "#0a0a0f",
        "bg_secondary": "#12121a",
        "bg_card": "#1a1a25",
        "text_primary": "#f0f0f0",
        "text_secondary": "#888888",
        "accent": "#ff00ff",
        "accent_secondary": "#00ffff",
        "error": "#ff3366",
        "warning": "#ffcc00",
        "success": "#00ff66",
        "border": "#ff00ff44"
    },
    "fonts": {
        "primary": "Helvetica",
        "monospace": "Consolas"
    },
    "overlay": {
        "width": 400,
        "height": 150,
        "opacity": 0.92,
        "position": "bottom-right",
        "border_radius": 8
    }
}
CYBERTHEME

    # Ocean theme
    cat > "$MAR_DIR/themes/ocean.json" <<'OCEANTHEME'
{
    "name": "Ocean",
    "author": "Marceline Team",
    "colors": {
        "bg_primary": "#0a192f",
        "bg_secondary": "#112240",
        "bg_card": "#1d3557",
        "text_primary": "#ccd6f6",
        "text_secondary": "#8892b0",
        "accent": "#64ffda",
        "accent_secondary": "#57cbff",
        "error": "#ff6b6b",
        "warning": "#ffd93d",
        "success": "#64ffda",
        "border": "#233554"
    },
    "fonts": {
        "primary": "Helvetica",
        "monospace": "Consolas"
    },
    "overlay": {
        "width": 380,
        "height": 140,
        "opacity": 0.95,
        "position": "bottom-right",
        "border_radius": 12
    }
}
OCEANTHEME

    # Active theme file
    echo '{"theme": "dark"}' > "$MAR_DIR/themes/active.json"
    
    chown -R "$REAL_USER:$REAL_USER" "$MAR_DIR/themes"
    log_success "Theme system created (4 themes)"
}

# ═══════════════════════════════════════════════════════════════════════════
# SMART HOME INTEGRATION (STUBS)
# ═══════════════════════════════════════════════════════════════════════════

create_smart_home_stubs() {
    log_step "Creating smart home integration stubs..."
    
    cat > "$MAR_DIR/mcp-servers/smarthome_server.py" <<'SMARTHOME'
#!/usr/bin/env python3
"""
MCP Smart Home Server - Integration stubs for various smart home platforms
Supports: Home Assistant, Philips Hue, MQTT, and generic device control
"""
import json
import os
import requests
from typing import Dict, List, Any, Optional
from dataclasses import dataclass
from enum import Enum

CONFIG_PATH = os.path.expanduser("~/marceline/config/smarthome.json")

class DeviceType(Enum):
    LIGHT = "light"
    SWITCH = "switch"
    THERMOSTAT = "thermostat"
    LOCK = "lock"
    SENSOR = "sensor"
    CAMERA = "camera"
    COVER = "cover"
    FAN = "fan"
    MEDIA = "media"

@dataclass
class Device:
    id: str
    name: str
    type: DeviceType
    state: Dict[str, Any]
    room: str = ""
    platform: str = "generic"

class SmarthomeServer:
    """Smart home control server - provides stubs for home automation"""
    
    def __init__(self):
        self.config = self._load_config()
        self.devices: Dict[str, Device] = {}
        self._init_demo_devices()
    
    def _load_config(self) -> Dict:
        try:
            with open(CONFIG_PATH) as f:
                return json.load(f)
        except:
            return {"platforms": {}, "demo_mode": True}
    
    def _init_demo_devices(self):
        """Initialize demo devices for testing"""
        demo = [
            Device("light_1", "Living Room Light", DeviceType.LIGHT, 
                   {"on": False, "brightness": 100}, "Living Room"),
            Device("light_2", "Bedroom Light", DeviceType.LIGHT,
                   {"on": True, "brightness": 50}, "Bedroom"),
            Device("switch_1", "Kitchen Switch", DeviceType.SWITCH,
                   {"on": False}, "Kitchen"),
            Device("thermo_1", "Main Thermostat", DeviceType.THERMOSTAT,
                   {"temperature": 22, "target": 21, "mode": "heat"}, "Hallway"),
            Device("lock_1", "Front Door", DeviceType.LOCK,
                   {"locked": True}, "Entrance"),
            Device("sensor_1", "Motion Sensor", DeviceType.SENSOR,
                   {"motion": False, "battery": 85}, "Living Room"),
        ]
        for d in demo:
            self.devices[d.id] = d
    
    # ─── Device Control ────────────────────────────────────────────────────
    
    def list_devices(self, room: str = None, type: str = None) -> List[Dict]:
        """List all devices, optionally filtered by room or type"""
        result = []
        for d in self.devices.values():
            if room and d.room.lower() != room.lower():
                continue
            if type and d.type.value != type:
                continue
            result.append({
                "id": d.id, "name": d.name, "type": d.type.value,
                "room": d.room, "state": d.state
            })
        return result
    
    def get_device(self, device_id: str) -> Optional[Dict]:
        """Get device details"""
        d = self.devices.get(device_id)
        if d:
            return {"id": d.id, "name": d.name, "type": d.type.value,
                    "room": d.room, "state": d.state}
        return None
    
    def turn_on(self, device_id: str) -> Dict:
        """Turn on a device"""
        if device_id in self.devices:
            self.devices[device_id].state["on"] = True
            return {"success": True, "device": device_id, "action": "on"}
        return {"success": False, "error": "Device not found"}
    
    def turn_off(self, device_id: str) -> Dict:
        """Turn off a device"""
        if device_id in self.devices:
            self.devices[device_id].state["on"] = False
            return {"success": True, "device": device_id, "action": "off"}
        return {"success": False, "error": "Device not found"}
    
    def toggle(self, device_id: str) -> Dict:
        """Toggle a device"""
        if device_id in self.devices:
            current = self.devices[device_id].state.get("on", False)
            self.devices[device_id].state["on"] = not current
            return {"success": True, "device": device_id, "state": not current}
        return {"success": False, "error": "Device not found"}
    
    def set_brightness(self, device_id: str, level: int) -> Dict:
        """Set brightness level (0-100)"""
        if device_id in self.devices:
            self.devices[device_id].state["brightness"] = max(0, min(100, level))
            return {"success": True, "device": device_id, "brightness": level}
        return {"success": False, "error": "Device not found"}
    
    def set_temperature(self, device_id: str, temp: float) -> Dict:
        """Set thermostat target temperature"""
        if device_id in self.devices:
            self.devices[device_id].state["target"] = temp
            return {"success": True, "device": device_id, "target": temp}
        return {"success": False, "error": "Device not found"}
    
    def lock(self, device_id: str) -> Dict:
        """Lock a door"""
        if device_id in self.devices:
            self.devices[device_id].state["locked"] = True
            return {"success": True, "device": device_id, "locked": True}
        return {"success": False, "error": "Device not found"}
    
    def unlock(self, device_id: str) -> Dict:
        """Unlock a door"""
        if device_id in self.devices:
            self.devices[device_id].state["locked"] = False
            return {"success": True, "device": device_id, "locked": False}
        return {"success": False, "error": "Device not found"}
    
    # ─── Room Control ──────────────────────────────────────────────────────
    
    def list_rooms(self) -> List[str]:
        """List all rooms"""
        return list(set(d.room for d in self.devices.values() if d.room))
    
    def room_on(self, room: str) -> Dict:
        """Turn on all devices in a room"""
        count = 0
        for d in self.devices.values():
            if d.room.lower() == room.lower() and d.type in [DeviceType.LIGHT, DeviceType.SWITCH]:
                d.state["on"] = True
                count += 1
        return {"success": True, "room": room, "devices_affected": count}
    
    def room_off(self, room: str) -> Dict:
        """Turn off all devices in a room"""
        count = 0
        for d in self.devices.values():
            if d.room.lower() == room.lower() and d.type in [DeviceType.LIGHT, DeviceType.SWITCH]:
                d.state["on"] = False
                count += 1
        return {"success": True, "room": room, "devices_affected": count}
    
    # ─── Scenes ────────────────────────────────────────────────────────────
    
    def activate_scene(self, scene: str) -> Dict:
        """Activate a predefined scene"""
        scenes = {
            "movie": [("light_1", {"on": True, "brightness": 20}),
                      ("light_2", {"on": False})],
            "bright": [("light_1", {"on": True, "brightness": 100}),
                       ("light_2", {"on": True, "brightness": 100})],
            "night": [("light_1", {"on": False}),
                      ("light_2", {"on": True, "brightness": 10})],
            "away": [("light_1", {"on": False}),
                     ("light_2", {"on": False}),
                     ("lock_1", {"locked": True})]
        }
        if scene.lower() in scenes:
            for device_id, state in scenes[scene.lower()]:
                if device_id in self.devices:
                    self.devices[device_id].state.update(state)
            return {"success": True, "scene": scene}
        return {"success": False, "error": f"Unknown scene: {scene}"}
    
    def list_scenes(self) -> List[str]:
        """List available scenes"""
        return ["movie", "bright", "night", "away"]
    
    # ─── Status ────────────────────────────────────────────────────────────
    
    def get_status(self) -> Dict:
        """Get overall smart home status"""
        lights_on = sum(1 for d in self.devices.values() 
                       if d.type == DeviceType.LIGHT and d.state.get("on"))
        return {
            "total_devices": len(self.devices),
            "lights_on": lights_on,
            "rooms": self.list_rooms(),
            "demo_mode": self.config.get("demo_mode", True)
        }
SMARTHOME

    # Smart home config
    cat > "$MAR_DIR/config/smarthome.json" <<'SHCONFIG'
{
    "demo_mode": true,
    "platforms": {
        "home_assistant": {
            "enabled": false,
            "url": "http://localhost:8123",
            "token": ""
        },
        "philips_hue": {
            "enabled": false,
            "bridge_ip": "",
            "username": ""
        },
        "mqtt": {
            "enabled": false,
            "broker": "localhost",
            "port": 1883,
            "username": "",
            "password": ""
        }
    }
}
SHCONFIG

    chmod +x "$MAR_DIR/mcp-servers/smarthome_server.py"
    chown -R "$REAL_USER:$REAL_USER" "$MAR_DIR/mcp-servers"
    chown -R "$REAL_USER:$REAL_USER" "$MAR_DIR/config"
    
    log_success "Smart home integration created"
}

# ═══════════════════════════════════════════════════════════════════════════
# BACKUP AND RESTORE
# ═══════════════════════════════════════════════════════════════════════════

create_backup_system() {
    log_step "Creating backup system..."
    
    cat > /usr/local/bin/marceline-backup <<'BACKUPSCRIPT'
#!/usr/bin/env bash
# Marceline Backup Script
set -euo pipefail

BACKUP_DIR="${HOME}/marceline-backups"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_NAME="marceline_backup_${TIMESTAMP}"
MAR_DIR="${HOME}/marceline"

mkdir -p "$BACKUP_DIR"

echo "🔄 Creating backup: $BACKUP_NAME"

# Create backup archive
tar -czf "${BACKUP_DIR}/${BACKUP_NAME}.tar.gz" \
    -C "$HOME" \
    marceline/config \
    marceline/data \
    marceline/themes \
    marceline/plugins \
    2>/dev/null || true

# Keep only last 5 backups
cd "$BACKUP_DIR"
ls -t marceline_backup_*.tar.gz 2>/dev/null | tail -n +6 | xargs -r rm --

echo "✅ Backup created: ${BACKUP_DIR}/${BACKUP_NAME}.tar.gz"
echo "📁 Total backups: $(ls marceline_backup_*.tar.gz 2>/dev/null | wc -l)"
BACKUPSCRIPT

    cat > /usr/local/bin/marceline-restore <<'RESTORESCRIPT'
#!/usr/bin/env bash
# Marceline Restore Script
set -euo pipefail

BACKUP_DIR="${HOME}/marceline-backups"
MAR_DIR="${HOME}/marceline"

if [[ $# -lt 1 ]]; then
    echo "Available backups:"
    ls -la "$BACKUP_DIR"/marceline_backup_*.tar.gz 2>/dev/null || echo "No backups found"
    echo ""
    echo "Usage: marceline-restore <backup_file.tar.gz>"
    exit 1
fi

BACKUP_FILE="$1"
if [[ ! -f "$BACKUP_FILE" ]]; then
    BACKUP_FILE="${BACKUP_DIR}/$1"
fi

if [[ ! -f "$BACKUP_FILE" ]]; then
    echo "❌ Backup file not found: $1"
    exit 1
fi

echo "🔄 Restoring from: $BACKUP_FILE"
echo "⚠️  This will overwrite current config, data, themes, and plugins."
read -p "Continue? (y/N) " -n 1 -r
echo

if [[ $REPLY =~ ^[Yy]$ ]]; then
    # Stop services
    sudo systemctl stop marceline-voice.service 2>/dev/null || true
    sudo systemctl stop marceline-overlay.service 2>/dev/null || true
    
    # Restore
    tar -xzf "$BACKUP_FILE" -C "$HOME"
    
    # Restart services
    sudo systemctl start marceline-voice.service 2>/dev/null || true
    sudo systemctl start marceline-overlay.service 2>/dev/null || true
    
    echo "✅ Restore complete!"
else
    echo "Cancelled."
fi
RESTORESCRIPT

    chmod +x /usr/local/bin/marceline-backup
    chmod +x /usr/local/bin/marceline-restore
    
    log_success "Backup system created"
}

# ═══════════════════════════════════════════════════════════════════════════
# UPDATE MECHANISM
# ═══════════════════════════════════════════════════════════════════════════

create_update_mechanism() {
    log_step "Creating update mechanism..."
    
    cat > /usr/local/bin/marceline-update <<'UPDATESCRIPT'
#!/usr/bin/env bash
# Marceline Update Script
set -euo pipefail

MAR_DIR="${HOME}/marceline"
VERSION_FILE="${MAR_DIR}/VERSION"

echo "🔄 Checking for Marceline updates..."

# Get current version
if [[ -f "$VERSION_FILE" ]]; then
    CURRENT=$(cat "$VERSION_FILE")
else
    CURRENT="unknown"
fi

echo "Current version: $CURRENT"

# Update Python packages
echo "📦 Updating Python packages..."
source "${MAR_DIR}/venv/bin/activate"
pip install --upgrade pip > /dev/null 2>&1
pip install --upgrade requests psutil vosk SpeechRecognition flask flask-cors > /dev/null 2>&1

# Update Ollama model
echo "🤖 Updating AI model..."
ollama pull llama3.2:3b > /dev/null 2>&1 || true

# Restart services
echo "🔁 Restarting services..."
sudo systemctl restart marceline-voice.service 2>/dev/null || true
sudo systemctl restart marceline-overlay.service 2>/dev/null || true

echo "✅ Update complete!"
UPDATESCRIPT

    chmod +x /usr/local/bin/marceline-update
    
    # Version file
    echo "2.0.0" > "$MAR_DIR/VERSION"
    chown "$REAL_USER:$REAL_USER" "$MAR_DIR/VERSION"
    
    log_success "Update mechanism created"
}

# ═══════════════════════════════════════════════════════════════════════════
# ADDITIONAL UTILITIES
# ═══════════════════════════════════════════════════════════════════════════

create_utilities() {
    log_step "Creating utility scripts..."
    
    # Diagnostic tool
    cat > /usr/local/bin/marceline-diag <<'DIAGSCRIPT'
#!/usr/bin/env bash
# Marceline Diagnostic Tool

echo "═══════════════════════════════════════════════════════════════"
echo "                  MARCELINE DIAGNOSTICS"
echo "═══════════════════════════════════════════════════════════════"
echo ""

MAR_DIR="${HOME}/marceline"

# System info
echo "📊 SYSTEM INFO:"
echo "   OS: $(uname -s) $(uname -r)"
echo "   User: $USER"
echo "   Home: $HOME"
echo ""

# Installation check
echo "📁 INSTALLATION:"
if [[ -d "$MAR_DIR" ]]; then
    echo "   ✓ Marceline directory exists"
    echo "   Size: $(du -sh "$MAR_DIR" 2>/dev/null | cut -f1)"
else
    echo "   ✗ Marceline directory NOT FOUND"
fi

if [[ -f "$MAR_DIR/venv/bin/python" ]]; then
    echo "   ✓ Python venv exists"
else
    echo "   ✗ Python venv NOT FOUND"
fi

if [[ -d "$MAR_DIR/models/vosk-model-small-en-us-0.15" ]]; then
    echo "   ✓ Vosk model exists"
else
    echo "   ✗ Vosk model NOT FOUND"
fi
echo ""

# Services
echo "🔧 SERVICES:"
for svc in marceline-voice marceline-overlay marceline-api ollama; do
    status=$(systemctl is-active $svc.service 2>/dev/null || echo "not found")
    if [[ "$status" == "active" ]]; then
        echo "   ✓ $svc: $status"
    else
        echo "   ✗ $svc: $status"
    fi
done
echo ""

# Audio
echo "🎤 AUDIO:"
if arecord -l 2>/dev/null | grep -q "card"; then
    echo "   ✓ Microphone detected"
    arecord -l 2>/dev/null | grep "card" | head -1 | sed 's/^/   /'
else
    echo "   ✗ No microphone detected"
fi

if command -v espeak-ng &>/dev/null; then
    echo "   ✓ espeak-ng installed"
else
    echo "   ✗ espeak-ng NOT installed"
fi
echo ""

# Ollama
echo "🤖 AI:"
if command -v ollama &>/dev/null; then
    echo "   ✓ Ollama installed"
    if ollama list 2>/dev/null | grep -q "llama"; then
        echo "   ✓ LLM model available"
    else
        echo "   ✗ LLM model NOT pulled"
    fi
else
    echo "   ✗ Ollama NOT installed"
fi
echo ""

# Ports
echo "🌐 PORTS:"
for port in 5000 8080 11434; do
    if ss -tlnp 2>/dev/null | grep -q ":$port"; then
        echo "   ✓ Port $port in use"
    else
        echo "   ○ Port $port free"
    fi
done
echo ""

echo "═══════════════════════════════════════════════════════════════"
DIAGSCRIPT

    chmod +x /usr/local/bin/marceline-diag
    
    # Quick test
    cat > /usr/local/bin/marceline-test <<'TESTSCRIPT'
#!/usr/bin/env bash
# Quick test for Marceline

echo "🧪 Testing Marceline components..."
echo ""

MAR_DIR="${HOME}/marceline"

# Test TTS
echo "🔊 Testing text-to-speech..."
espeak-ng "Marceline test successful" 2>/dev/null && echo "   ✓ TTS works" || echo "   ✗ TTS failed"

# Test microphone
echo "🎤 Testing microphone (2 seconds)..."
if arecord -d 1 -f S16_LE -r 16000 /tmp/marceline_test.wav 2>/dev/null; then
    echo "   ✓ Microphone works"
    rm -f /tmp/marceline_test.wav
else
    echo "   ✗ Microphone failed"
fi

# Test Ollama
echo "🤖 Testing AI..."
if curl -s http://localhost:11434/api/tags | grep -q "llama"; then
    echo "   ✓ Ollama running with model"
else
    echo "   ✗ Ollama not ready"
fi

# Test config
echo "📋 Testing config..."
if [[ -f "/etc/marceline/config.json" ]]; then
    echo "   ✓ Config file exists"
else
    echo "   ✗ Config file missing"
fi

echo ""
echo "✅ Tests complete!"
TESTSCRIPT

    chmod +x /usr/local/bin/marceline-test
    
    log_success "Utility scripts created"
}

# ═══════════════════════════════════════════════════════════════════════════
# EMAIL CLIENT SERVER
# ═══════════════════════════════════════════════════════════════════════════

create_email_server() {
    log_step "Creating email server..."
    
    cat > "$MAR_DIR/mcp-servers/email_server.py" <<'EMAILSERVER'
#!/usr/bin/env python3
"""MCP Email Server - IMAP/SMTP email operations"""
import imaplib
import smtplib
import email
from email.mime.text import MIMEText
from email.mime.multipart import MIMEMultipart
from email.header import decode_header
import json
import os
from typing import List, Dict, Optional
from datetime import datetime

CONFIG_PATH = os.path.expanduser("~/marceline/config/email.json")

class EmailServer:
    def __init__(self):
        self.config = self._load_config()
        self.imap = None
        self.smtp = None
    
    def _load_config(self) -> Dict:
        try:
            with open(CONFIG_PATH) as f:
                return json.load(f)
        except:
            return {"configured": False}
    
    def _connect_imap(self) -> bool:
        if not self.config.get("configured"):
            return False
        try:
            self.imap = imaplib.IMAP4_SSL(self.config["imap_server"], self.config.get("imap_port", 993))
            self.imap.login(self.config["email"], self.config["password"])
            return True
        except Exception as e:
            print(f"IMAP error: {e}")
            return False
    
    def _connect_smtp(self) -> bool:
        if not self.config.get("configured"):
            return False
        try:
            self.smtp = smtplib.SMTP_SSL(self.config["smtp_server"], self.config.get("smtp_port", 465))
            self.smtp.login(self.config["email"], self.config["password"])
            return True
        except Exception as e:
            print(f"SMTP error: {e}")
            return False
    
    def get_inbox(self, limit: int = 10) -> List[Dict]:
        if not self._connect_imap():
            return [{"error": "Email not configured"}]
        try:
            self.imap.select("INBOX")
            _, messages = self.imap.search(None, "ALL")
            email_ids = messages[0].split()[-limit:]
            emails = []
            for eid in reversed(email_ids):
                _, msg = self.imap.fetch(eid, "(RFC822)")
                email_msg = email.message_from_bytes(msg[0][1])
                subject = decode_header(email_msg["Subject"])[0][0]
                if isinstance(subject, bytes):
                    subject = subject.decode()
                emails.append({
                    "id": eid.decode(),
                    "from": email_msg["From"],
                    "subject": subject,
                    "date": email_msg["Date"]
                })
            self.imap.logout()
            return emails
        except Exception as e:
            return [{"error": str(e)}]
    
    def read_email(self, email_id: str) -> Dict:
        if not self._connect_imap():
            return {"error": "Email not configured"}
        try:
            self.imap.select("INBOX")
            _, msg = self.imap.fetch(email_id.encode(), "(RFC822)")
            email_msg = email.message_from_bytes(msg[0][1])
            body = ""
            if email_msg.is_multipart():
                for part in email_msg.walk():
                    if part.get_content_type() == "text/plain":
                        body = part.get_payload(decode=True).decode()
                        break
            else:
                body = email_msg.get_payload(decode=True).decode()
            self.imap.logout()
            return {"from": email_msg["From"], "subject": email_msg["Subject"], 
                    "date": email_msg["Date"], "body": body[:2000]}
        except Exception as e:
            return {"error": str(e)}
    
    def send_email(self, to: str, subject: str, body: str) -> Dict:
        if not self._connect_smtp():
            return {"error": "Email not configured"}
        try:
            msg = MIMEMultipart()
            msg["From"] = self.config["email"]
            msg["To"] = to
            msg["Subject"] = subject
            msg.attach(MIMEText(body, "plain"))
            self.smtp.send_message(msg)
            self.smtp.quit()
            return {"success": True, "to": to, "subject": subject}
        except Exception as e:
            return {"error": str(e)}
    
    def count_unread(self) -> Dict:
        if not self._connect_imap():
            return {"error": "Email not configured"}
        try:
            self.imap.select("INBOX")
            _, messages = self.imap.search(None, "UNSEEN")
            count = len(messages[0].split())
            self.imap.logout()
            return {"unread": count}
        except Exception as e:
            return {"error": str(e)}
    
    def is_configured(self) -> bool:
        return self.config.get("configured", False)
EMAILSERVER

    # Email config template
    cat > "$MAR_DIR/config/email.json" <<'EMAILCONFIG'
{
    "configured": false,
    "email": "",
    "password": "",
    "imap_server": "imap.gmail.com",
    "imap_port": 993,
    "smtp_server": "smtp.gmail.com",
    "smtp_port": 465,
    "name": "User"
}
EMAILCONFIG

    chmod 600 "$MAR_DIR/config/email.json"
    chown "$REAL_USER:$REAL_USER" "$MAR_DIR/config/email.json"
    chmod +x "$MAR_DIR/mcp-servers/email_server.py"
    log_success "Email server created"
}

# ═══════════════════════════════════════════════════════════════════════════
# VISION/CAMERA SERVER
# ═══════════════════════════════════════════════════════════════════════════

create_vision_server() {
    log_step "Creating vision server..."
    
    cat > "$MAR_DIR/mcp-servers/vision_server.py" <<'VISIONSERVER'
#!/usr/bin/env python3
"""MCP Vision Server - Camera and image analysis"""
import subprocess
import base64
import os
import json
import requests
from datetime import datetime
from typing import Dict, Optional
from pathlib import Path

CAPTURE_DIR = Path.home() / "marceline" / "data" / "captures"
CAPTURE_DIR.mkdir(parents=True, exist_ok=True)

class VisionServer:
    def __init__(self):
        self.ollama_url = "http://localhost:11434/api/generate"
    
    def capture_photo(self, name: str = None) -> Dict:
        if not name:
            name = f"photo_{datetime.now().strftime('%Y%m%d_%H%M%S')}"
        filepath = CAPTURE_DIR / f"{name}.jpg"
        try:
            result = subprocess.run(
                ["fswebcam", "-r", "1280x720", "--no-banner", "-S", "3", str(filepath)],
                capture_output=True, timeout=10
            )
            if filepath.exists():
                return {"success": True, "path": str(filepath), "size": filepath.stat().st_size}
            return {"success": False, "error": "Capture failed"}
        except Exception as e:
            return {"success": False, "error": str(e)}
    
    def take_screenshot(self, name: str = None) -> Dict:
        if not name:
            name = f"screen_{datetime.now().strftime('%Y%m%d_%H%M%S')}"
        filepath = CAPTURE_DIR / f"{name}.png"
        try:
            subprocess.run(["scrot", str(filepath)], capture_output=True, timeout=5)
            if filepath.exists():
                return {"success": True, "path": str(filepath)}
            return {"success": False, "error": "Screenshot failed"}
        except Exception as e:
            return {"success": False, "error": str(e)}
    
    def analyze_image(self, image_path: str, question: str = "Describe this image") -> Dict:
        try:
            with open(image_path, "rb") as f:
                image_data = base64.b64encode(f.read()).decode()
            response = requests.post(self.ollama_url, json={
                "model": "llava",
                "prompt": question,
                "images": [image_data],
                "stream": False
            }, timeout=120)
            if response.status_code == 200:
                return {"analysis": response.json().get("response", "")}
            return {"error": "Analysis failed"}
        except Exception as e:
            return {"error": str(e)}
    
    def list_captures(self, limit: int = 20) -> list:
        files = sorted(CAPTURE_DIR.glob("*.*"), key=lambda x: x.stat().st_mtime, reverse=True)
        return [{"name": f.name, "path": str(f), "size": f.stat().st_size} for f in files[:limit]]
    
    def delete_capture(self, name: str) -> Dict:
        filepath = CAPTURE_DIR / name
        if filepath.exists():
            filepath.unlink()
            return {"success": True}
        return {"success": False, "error": "File not found"}
    
    def get_camera_info(self) -> Dict:
        try:
            result = subprocess.run(["v4l2-ctl", "--list-devices"], capture_output=True, text=True)
            return {"devices": result.stdout}
        except:
            return {"devices": "v4l2-ctl not available"}
VISIONSERVER

    chmod +x "$MAR_DIR/mcp-servers/vision_server.py"
    log_success "Vision server created"
}

# ═══════════════════════════════════════════════════════════════════════════
# CODE EXECUTION SERVER
# ═══════════════════════════════════════════════════════════════════════════

create_code_server() {
    log_step "Creating code execution server..."
    
    cat > "$MAR_DIR/mcp-servers/code_server.py" <<'CODESERVER'
#!/usr/bin/env python3
"""MCP Code Execution Server - Safe code runner"""
import subprocess
import tempfile
import os
import signal
from typing import Dict
from pathlib import Path

SANDBOX_DIR = Path.home() / "marceline" / "data" / "sandbox"
SANDBOX_DIR.mkdir(parents=True, exist_ok=True)

class CodeServer:
    def __init__(self):
        self.timeout = 30
        self.max_output = 10000
    
    def run_python(self, code: str, timeout: int = None) -> Dict:
        timeout = timeout or self.timeout
        with tempfile.NamedTemporaryFile(mode='w', suffix='.py', dir=SANDBOX_DIR, delete=False) as f:
            f.write(code)
            filepath = f.name
        try:
            result = subprocess.run(
                ["python3", filepath],
                capture_output=True, text=True, timeout=timeout,
                cwd=str(SANDBOX_DIR)
            )
            return {
                "stdout": result.stdout[:self.max_output],
                "stderr": result.stderr[:self.max_output],
                "returncode": result.returncode
            }
        except subprocess.TimeoutExpired:
            return {"error": f"Execution timed out after {timeout}s"}
        except Exception as e:
            return {"error": str(e)}
        finally:
            os.unlink(filepath)
    
    def run_bash(self, command: str, timeout: int = None) -> Dict:
        timeout = timeout or self.timeout
        try:
            result = subprocess.run(
                ["bash", "-c", command],
                capture_output=True, text=True, timeout=timeout,
                cwd=str(SANDBOX_DIR)
            )
            return {
                "stdout": result.stdout[:self.max_output],
                "stderr": result.stderr[:self.max_output],
                "returncode": result.returncode
            }
        except subprocess.TimeoutExpired:
            return {"error": f"Command timed out after {timeout}s"}
        except Exception as e:
            return {"error": str(e)}
    
    def run_node(self, code: str, timeout: int = None) -> Dict:
        timeout = timeout or self.timeout
        with tempfile.NamedTemporaryFile(mode='w', suffix='.js', dir=SANDBOX_DIR, delete=False) as f:
            f.write(code)
            filepath = f.name
        try:
            result = subprocess.run(
                ["node", filepath],
                capture_output=True, text=True, timeout=timeout
            )
            return {
                "stdout": result.stdout[:self.max_output],
                "stderr": result.stderr[:self.max_output],
                "returncode": result.returncode
            }
        except subprocess.TimeoutExpired:
            return {"error": f"Execution timed out after {timeout}s"}
        except Exception as e:
            return {"error": str(e)}
        finally:
            os.unlink(filepath)
    
    def eval_math(self, expression: str) -> Dict:
        code = f"print({expression})"
        return self.run_python(code, timeout=5)
    
    def format_python(self, code: str) -> Dict:
        try:
            result = subprocess.run(
                ["python3", "-m", "black", "-c", code],
                capture_output=True, text=True, timeout=10
            )
            return {"formatted": result.stdout or code}
        except:
            return {"formatted": code}
    
    def lint_python(self, code: str) -> Dict:
        with tempfile.NamedTemporaryFile(mode='w', suffix='.py', dir=SANDBOX_DIR, delete=False) as f:
            f.write(code)
            filepath = f.name
        try:
            result = subprocess.run(
                ["python3", "-m", "pylint", "--output-format=json", filepath],
                capture_output=True, text=True, timeout=30
            )
            return {"issues": result.stdout}
        except Exception as e:
            return {"error": str(e)}
        finally:
            os.unlink(filepath)
CODESERVER

    chmod +x "$MAR_DIR/mcp-servers/code_server.py"
    log_success "Code execution server created"
}

# ═══════════════════════════════════════════════════════════════════════════
# MUSIC/MEDIA SERVER
# ═══════════════════════════════════════════════════════════════════════════

create_music_server() {
    log_step "Creating music server..."
    
    cat > "$MAR_DIR/mcp-servers/music_server.py" <<'MUSICSERVER'
#!/usr/bin/env python3
"""MCP Music Server - Media playback control"""
import subprocess
import dbus
from typing import Dict, List, Optional
import os

class MusicServer:
    def __init__(self):
        self.players = ["spotify", "rhythmbox", "vlc", "audacious"]
    
    def _get_player(self) -> Optional[str]:
        try:
            bus = dbus.SessionBus()
            for name in bus.list_names():
                if "org.mpris.MediaPlayer2" in name:
                    return name
        except:
            pass
        return None
    
    def _mpris_cmd(self, cmd: str) -> bool:
        player = self._get_player()
        if not player:
            return False
        try:
            bus = dbus.SessionBus()
            proxy = bus.get_object(player, "/org/mpris/MediaPlayer2")
            iface = dbus.Interface(proxy, "org.mpris.MediaPlayer2.Player")
            getattr(iface, cmd)()
            return True
        except:
            return False
    
    def play(self) -> Dict:
        return {"success": self._mpris_cmd("Play")}
    
    def pause(self) -> Dict:
        return {"success": self._mpris_cmd("Pause")}
    
    def play_pause(self) -> Dict:
        return {"success": self._mpris_cmd("PlayPause")}
    
    def next_track(self) -> Dict:
        return {"success": self._mpris_cmd("Next")}
    
    def previous_track(self) -> Dict:
        return {"success": self._mpris_cmd("Previous")}
    
    def stop(self) -> Dict:
        return {"success": self._mpris_cmd("Stop")}
    
    def get_current(self) -> Dict:
        player = self._get_player()
        if not player:
            return {"error": "No player found"}
        try:
            bus = dbus.SessionBus()
            proxy = bus.get_object(player, "/org/mpris/MediaPlayer2")
            props = dbus.Interface(proxy, "org.freedesktop.DBus.Properties")
            metadata = props.Get("org.mpris.MediaPlayer2.Player", "Metadata")
            return {
                "title": str(metadata.get("xesam:title", "Unknown")),
                "artist": str(metadata.get("xesam:artist", ["Unknown"])[0]) if metadata.get("xesam:artist") else "Unknown",
                "album": str(metadata.get("xesam:album", "Unknown"))
            }
        except Exception as e:
            return {"error": str(e)}
    
    def open_spotify(self) -> Dict:
        subprocess.Popen(["spotify"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        return {"success": True, "player": "spotify"}
    
    def open_youtube(self, query: str = "") -> Dict:
        url = f"https://www.youtube.com/results?search_query={query}" if query else "https://www.youtube.com"
        subprocess.Popen(["firefox", url], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        return {"success": True, "url": url}
    
    def set_volume(self, level: int) -> Dict:
        subprocess.run(["amixer", "-q", "set", "Master", f"{level}%"], capture_output=True)
        return {"success": True, "volume": level}
    
    def volume_up(self, step: int = 10) -> Dict:
        subprocess.run(["amixer", "-q", "set", "Master", f"{step}%+"], capture_output=True)
        return {"success": True}
    
    def volume_down(self, step: int = 10) -> Dict:
        subprocess.run(["amixer", "-q", "set", "Master", f"{step}%-"], capture_output=True)
        return {"success": True}
    
    def mute(self) -> Dict:
        subprocess.run(["amixer", "-q", "set", "Master", "toggle"], capture_output=True)
        return {"success": True}
MUSICSERVER

    chmod +x "$MAR_DIR/mcp-servers/music_server.py"
    log_success "Music server created"
}

# ═══════════════════════════════════════════════════════════════════════════
# BROWSER AUTOMATION SERVER
# ═══════════════════════════════════════════════════════════════════════════

create_browser_server() {
    log_step "Creating browser automation server..."
    
    cat > "$MAR_DIR/mcp-servers/browser_server.py" <<'BROWSERSERVER'
#!/usr/bin/env python3
"""MCP Browser Server - Web browser control"""
import subprocess
import webbrowser
from typing import Dict, List

class BrowserServer:
    def __init__(self):
        self.default_browser = "firefox"
    
    def open_url(self, url: str) -> Dict:
        if not url.startswith(("http://", "https://")):
            url = "https://" + url
        subprocess.Popen([self.default_browser, url], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        return {"success": True, "url": url}
    
    def google_search(self, query: str) -> Dict:
        url = f"https://www.google.com/search?q={query.replace(' ', '+')}"
        return self.open_url(url)
    
    def youtube_search(self, query: str) -> Dict:
        url = f"https://www.youtube.com/results?search_query={query.replace(' ', '+')}"
        return self.open_url(url)
    
    def wikipedia(self, topic: str) -> Dict:
        url = f"https://en.wikipedia.org/wiki/{topic.replace(' ', '_')}"
        return self.open_url(url)
    
    def github(self, repo: str = "") -> Dict:
        url = f"https://github.com/{repo}" if repo else "https://github.com"
        return self.open_url(url)
    
    def maps(self, location: str) -> Dict:
        url = f"https://www.google.com/maps/search/{location.replace(' ', '+')}"
        return self.open_url(url)
    
    def translate(self, text: str, target: str = "en") -> Dict:
        url = f"https://translate.google.com/?sl=auto&tl={target}&text={text.replace(' ', '+')}"
        return self.open_url(url)
    
    def weather(self, location: str = "") -> Dict:
        url = f"https://weather.google.com/weather?q={location}" if location else "https://weather.google.com"
        return self.open_url(url)
    
    def news(self) -> Dict:
        return self.open_url("https://news.google.com")
    
    def email(self) -> Dict:
        return self.open_url("https://mail.google.com")
    
    def calendar(self) -> Dict:
        return self.open_url("https://calendar.google.com")
    
    def drive(self) -> Dict:
        return self.open_url("https://drive.google.com")
    
    def docs(self) -> Dict:
        return self.open_url("https://docs.google.com")
    
    def sheets(self) -> Dict:
        return self.open_url("https://sheets.google.com")
    
    def amazon(self, search: str = "") -> Dict:
        url = f"https://www.amazon.com/s?k={search.replace(' ', '+')}" if search else "https://www.amazon.com"
        return self.open_url(url)
    
    def reddit(self, subreddit: str = "") -> Dict:
        url = f"https://www.reddit.com/r/{subreddit}" if subreddit else "https://www.reddit.com"
        return self.open_url(url)
    
    def twitter(self) -> Dict:
        return self.open_url("https://twitter.com")
    
    def linkedin(self) -> Dict:
        return self.open_url("https://www.linkedin.com")
    
    def stackoverflow(self, query: str = "") -> Dict:
        url = f"https://stackoverflow.com/search?q={query.replace(' ', '+')}" if query else "https://stackoverflow.com"
        return self.open_url(url)
BROWSERSERVER

    chmod +x "$MAR_DIR/mcp-servers/browser_server.py"
    log_success "Browser server created"
}

# ═══════════════════════════════════════════════════════════════════════════
# DOWNLOAD MANAGER SERVER  
# ═══════════════════════════════════════════════════════════════════════════

create_download_server() {
    log_step "Creating download server..."
    
    cat > "$MAR_DIR/mcp-servers/download_server.py" <<'DOWNLOADSERVER'
#!/usr/bin/env python3
"""MCP Download Server - File download manager"""
import subprocess
import os
from pathlib import Path
from typing import Dict, List
import threading
import json

DOWNLOAD_DIR = Path.home() / "Downloads"
STATUS_FILE = Path.home() / "marceline" / "data" / "downloads.json"

class DownloadServer:
    def __init__(self):
        self.active_downloads = {}
    
    def download_file(self, url: str, filename: str = None) -> Dict:
        if not filename:
            filename = url.split("/")[-1].split("?")[0] or "download"
        filepath = DOWNLOAD_DIR / filename
        try:
            process = subprocess.Popen(
                ["wget", "-O", str(filepath), url],
                stdout=subprocess.PIPE, stderr=subprocess.PIPE
            )
            self.active_downloads[url] = {"path": str(filepath), "pid": process.pid}
            return {"success": True, "path": str(filepath), "pid": process.pid}
        except Exception as e:
            return {"error": str(e)}
    
    def download_youtube(self, url: str, audio_only: bool = False) -> Dict:
        try:
            args = ["yt-dlp", "-o", f"{DOWNLOAD_DIR}/%(title)s.%(ext)s"]
            if audio_only:
                args.extend(["-x", "--audio-format", "mp3"])
            args.append(url)
            process = subprocess.Popen(args, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            return {"success": True, "pid": process.pid, "audio_only": audio_only}
        except Exception as e:
            return {"error": str(e)}
    
    def list_downloads(self, limit: int = 20) -> List[Dict]:
        files = sorted(DOWNLOAD_DIR.glob("*"), key=lambda x: x.stat().st_mtime, reverse=True)
        return [{"name": f.name, "size": f.stat().st_size, "path": str(f)} for f in files[:limit] if f.is_file()]
    
    def cancel_download(self, pid: int) -> Dict:
        try:
            os.kill(pid, 9)
            return {"success": True}
        except Exception as e:
            return {"error": str(e)}
    
    def get_progress(self, url: str) -> Dict:
        if url in self.active_downloads:
            filepath = Path(self.active_downloads[url]["path"])
            if filepath.exists():
                return {"path": str(filepath), "size": filepath.stat().st_size}
        return {"status": "unknown"}
DOWNLOADSERVER

    chmod +x "$MAR_DIR/mcp-servers/download_server.py"
    log_success "Download server created"
}

# ═══════════════════════════════════════════════════════════════════════════
# TIME SERVER (Reference implementation)
# ═══════════════════════════════════════════════════════════════════════════

create_time_server() {
    log_step "Creating time server..."
    
    cat > "$MAR_DIR/mcp-servers/time_server.py" <<'TIMESERVER'
#!/usr/bin/env python3
"""MCP Time Server - Time and timezone operations"""
from datetime import datetime, timedelta
import pytz
from typing import Dict, List

class TimeServer:
    def get_current_time(self, timezone: str = "local") -> Dict:
        if timezone == "local":
            now = datetime.now()
        else:
            try:
                tz = pytz.timezone(timezone)
                now = datetime.now(tz)
            except:
                now = datetime.now()
        return {
            "time": now.strftime("%H:%M:%S"),
            "date": now.strftime("%Y-%m-%d"),
            "day": now.strftime("%A"),
            "timezone": timezone,
            "iso": now.isoformat()
        }
    
    def convert_timezone(self, time_str: str, from_tz: str, to_tz: str) -> Dict:
        try:
            from_zone = pytz.timezone(from_tz)
            to_zone = pytz.timezone(to_tz)
            dt = datetime.strptime(time_str, "%H:%M")
            dt = from_zone.localize(dt)
            converted = dt.astimezone(to_zone)
            return {"original": time_str, "converted": converted.strftime("%H:%M"), "timezone": to_tz}
        except Exception as e:
            return {"error": str(e)}
    
    def add_time(self, hours: int = 0, minutes: int = 0, days: int = 0) -> Dict:
        future = datetime.now() + timedelta(hours=hours, minutes=minutes, days=days)
        return {"result": future.strftime("%Y-%m-%d %H:%M:%S")}
    
    def get_timezones(self) -> List[str]:
        return pytz.common_timezones[:50]
    
    def is_dst(self, timezone: str) -> Dict:
        try:
            tz = pytz.timezone(timezone)
            now = datetime.now(tz)
            return {"timezone": timezone, "dst": bool(now.dst())}
        except:
            return {"error": "Invalid timezone"}
TIMESERVER

    chmod +x "$MAR_DIR/mcp-servers/time_server.py"
    log_success "Time server created"
}

# ═══════════════════════════════════════════════════════════════════════════
# FETCH SERVER (Web content)
# ═══════════════════════════════════════════════════════════════════════════

create_fetch_server() {
    log_step "Creating fetch server..."
    
    cat > "$MAR_DIR/mcp-servers/fetch_server.py" <<'FETCHSERVER'
#!/usr/bin/env python3
"""MCP Fetch Server - Web content fetching"""
import requests
from bs4 import BeautifulSoup
from typing import Dict
import json

class FetchServer:
    def __init__(self):
        self.timeout = 10
        self.headers = {"User-Agent": "Marceline/1.0"}
    
    def fetch_url(self, url: str, format: str = "text") -> Dict:
        try:
            response = requests.get(url, headers=self.headers, timeout=self.timeout)
            if format == "json":
                return {"data": response.json(), "status": response.status_code}
            elif format == "html":
                return {"html": response.text[:5000], "status": response.status_code}
            else:
                soup = BeautifulSoup(response.text, 'html.parser')
                text = soup.get_text(separator='\n', strip=True)
                return {"text": text[:3000], "status": response.status_code}
        except Exception as e:
            return {"error": str(e)}
    
    def get_headers(self, url: str) -> Dict:
        try:
            response = requests.head(url, headers=self.headers, timeout=self.timeout)
            return {"headers": dict(response.headers), "status": response.status_code}
        except Exception as e:
            return {"error": str(e)}
    
    def check_url(self, url: str) -> Dict:
        try:
            response = requests.head(url, headers=self.headers, timeout=5)
            return {"reachable": True, "status": response.status_code}
        except:
            return {"reachable": False}
    
    def get_title(self, url: str) -> Dict:
        try:
            response = requests.get(url, headers=self.headers, timeout=self.timeout)
            soup = BeautifulSoup(response.text, 'html.parser')
            title = soup.find('title')
            return {"title": title.text if title else "No title found"}
        except Exception as e:
            return {"error": str(e)}
FETCHSERVER

    chmod +x "$MAR_DIR/mcp-servers/fetch_server.py"
    log_success "Fetch server created"
}

# ═══════════════════════════════════════════════════════════════════════════
# SCREENSHOT SERVER
# ═══════════════════════════════════════════════════════════════════════════

create_screenshot_server() {
    log_step "Creating screenshot server..."
    
    cat > "$MAR_DIR/mcp-servers/screenshot_server.py" <<'SCREENSHOTSERVER'
#!/usr/bin/env python3
"""MCP Screenshot Server - Screen capture operations"""
import subprocess
from pathlib import Path
from datetime import datetime
from typing import Dict

CAPTURE_DIR = Path.home() / "Pictures" / "Screenshots"
CAPTURE_DIR.mkdir(parents=True, exist_ok=True)

class ScreenshotServer:
    def capture_full(self, name: str = None) -> Dict:
        if not name:
            name = f"screenshot_{datetime.now().strftime('%Y%m%d_%H%M%S')}"
        path = CAPTURE_DIR / f"{name}.png"
        try:
            subprocess.run(["scrot", str(path)], capture_output=True, timeout=5)
            if path.exists():
                return {"success": True, "path": str(path)}
        except Exception as e:
            return {"error": str(e)}
        return {"success": False}
    
    def capture_window(self, name: str = None) -> Dict:
        if not name:
            name = f"window_{datetime.now().strftime('%Y%m%d_%H%M%S')}"
        path = CAPTURE_DIR / f"{name}.png"
        try:
            subprocess.run(["scrot", "-u", str(path)], capture_output=True, timeout=5)
            if path.exists():
                return {"success": True, "path": str(path)}
        except Exception as e:
            return {"error": str(e)}
        return {"success": False}
    
    def capture_selection(self, name: str = None) -> Dict:
        if not name:
            name = f"selection_{datetime.now().strftime('%Y%m%d_%H%M%S')}"
        path = CAPTURE_DIR / f"{name}.png"
        try:
            subprocess.run(["scrot", "-s", str(path)], capture_output=True, timeout=30)
            if path.exists():
                return {"success": True, "path": str(path)}
        except Exception as e:
            return {"error": str(e)}
        return {"success": False}
    
    def list_screenshots(self, limit: int = 20) -> list:
        files = sorted(CAPTURE_DIR.glob("*.png"), key=lambda x: x.stat().st_mtime, reverse=True)
        return [{"name": f.name, "path": str(f), "size": f.stat().st_size} for f in files[:limit]]
SCREENSHOTSERVER

    chmod +x "$MAR_DIR/mcp-servers/screenshot_server.py"
    log_success "Screenshot server created"
}

# ═══════════════════════════════════════════════════════════════════════════
# PROCESS SERVER
# ═══════════════════════════════════════════════════════════════════════════

create_process_server() {
    log_step "Creating process server..."
    
    cat > "$MAR_DIR/mcp-servers/process_server.py" <<'PROCESSSERVER'
#!/usr/bin/env python3
"""MCP Process Server - Process management"""
import psutil
import subprocess
from typing import Dict, List

class ProcessServer:
    def list_processes(self, limit: int = 20) -> List[Dict]:
        procs = []
        for p in psutil.process_iter(['pid', 'name', 'cpu_percent', 'memory_percent', 'status']):
            try:
                procs.append(p.info)
            except:
                pass
        return sorted(procs, key=lambda x: x.get('cpu_percent', 0), reverse=True)[:limit]
    
    def find_process(self, name: str) -> List[Dict]:
        results = []
        for p in psutil.process_iter(['pid', 'name', 'cmdline']):
            try:
                if name.lower() in p.info['name'].lower():
                    results.append(p.info)
            except:
                pass
        return results
    
    def kill_process(self, pid: int) -> Dict:
        try:
            p = psutil.Process(pid)
            p.terminate()
            return {"success": True, "pid": pid}
        except Exception as e:
            return {"error": str(e)}
    
    def kill_by_name(self, name: str) -> Dict:
        killed = 0
        for p in psutil.process_iter(['pid', 'name']):
            try:
                if name.lower() in p.info['name'].lower():
                    psutil.Process(p.info['pid']).terminate()
                    killed += 1
            except:
                pass
        return {"killed": killed, "name": name}
    
    def get_process_info(self, pid: int) -> Dict:
        try:
            p = psutil.Process(pid)
            return {
                "pid": pid,
                "name": p.name(),
                "status": p.status(),
                "cpu": p.cpu_percent(),
                "memory": p.memory_percent(),
                "threads": p.num_threads(),
                "created": p.create_time()
            }
        except Exception as e:
            return {"error": str(e)}
PROCESSSERVER

    chmod +x "$MAR_DIR/mcp-servers/process_server.py"
    log_success "Process server created"
}

# ═══════════════════════════════════════════════════════════════════════════
# NETWORK SERVER
# ═══════════════════════════════════════════════════════════════════════════

create_network_server() {
    log_step "Creating network server..."
    
    cat > "$MAR_DIR/mcp-servers/network_server.py" <<'NETWORKSERVER'
#!/usr/bin/env python3
"""MCP Network Server - Network operations"""
import subprocess
import socket
import psutil
from typing import Dict, List

class NetworkServer:
    def get_ip(self) -> Dict:
        try:
            s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
            s.connect(("8.8.8.8", 80))
            local_ip = s.getsockname()[0]
            s.close()
            return {"local_ip": local_ip}
        except:
            return {"error": "Cannot get IP"}
    
    def get_public_ip(self) -> Dict:
        import requests
        try:
            response = requests.get("https://api.ipify.org?format=json", timeout=5)
            return response.json()
        except:
            return {"error": "Cannot get public IP"}
    
    def ping(self, host: str, count: int = 4) -> Dict:
        try:
            result = subprocess.run(["ping", "-c", str(count), host], capture_output=True, text=True, timeout=30)
            return {"success": result.returncode == 0, "output": result.stdout[:500]}
        except Exception as e:
            return {"error": str(e)}
    
    def get_interfaces(self) -> List[Dict]:
        interfaces = []
        for name, addrs in psutil.net_if_addrs().items():
            for addr in addrs:
                if addr.family == socket.AF_INET:
                    interfaces.append({"name": name, "ip": addr.address, "netmask": addr.netmask})
        return interfaces
    
    def get_connections(self, limit: int = 20) -> List[Dict]:
        conns = []
        for c in psutil.net_connections()[:limit]:
            try:
                conns.append({
                    "local": f"{c.laddr.ip}:{c.laddr.port}" if c.laddr else None,
                    "remote": f"{c.raddr.ip}:{c.raddr.port}" if c.raddr else None,
                    "status": c.status
                })
            except:
                pass
        return conns
    
    def check_port(self, host: str, port: int) -> Dict:
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.settimeout(3)
        result = sock.connect_ex((host, port))
        sock.close()
        return {"host": host, "port": port, "open": result == 0}
NETWORKSERVER

    chmod +x "$MAR_DIR/mcp-servers/network_server.py"
    log_success "Network server created"
}

# ═══════════════════════════════════════════════════════════════════════════
# CURRENCY SERVER
# ═══════════════════════════════════════════════════════════════════════════

create_currency_server() {
    log_step "Creating currency server..."
    
    cat > "$MAR_DIR/mcp-servers/currency_server.py" <<'CURRENCYSERVER'
#!/usr/bin/env python3
"""MCP Currency Server - Currency conversion"""
import requests
from typing import Dict

class CurrencyServer:
    def __init__(self):
        self.api_url = "https://api.exchangerate-api.com/v4/latest"
    
    def convert(self, amount: float, from_curr: str, to_curr: str) -> Dict:
        try:
            response = requests.get(f"{self.api_url}/{from_curr.upper()}", timeout=5)
            if response.status_code == 200:
                data = response.json()
                rate = data["rates"].get(to_curr.upper())
                if rate:
                    result = amount * rate
                    return {"amount": amount, "from": from_curr, "to": to_curr, "result": round(result, 2), "rate": rate}
            return {"error": "Currency not found"}
        except Exception as e:
            return {"error": str(e)}
    
    def get_rates(self, base: str = "USD") -> Dict:
        try:
            response = requests.get(f"{self.api_url}/{base.upper()}", timeout=5)
            if response.status_code == 200:
                data = response.json()
                return {"base": base, "rates": data["rates"]}
            return {"error": "Cannot fetch rates"}
        except Exception as e:
            return {"error": str(e)}
    
    def list_currencies(self) -> list:
        return ["USD", "EUR", "GBP", "JPY", "AUD", "CAD", "CHF", "CNY", "INR", "KRW", "MXN", "BRL", "RUB", "ZAR"]
CURRENCYSERVER

    chmod +x "$MAR_DIR/mcp-servers/currency_server.py"
    log_success "Currency server created"
}

# ═══════════════════════════════════════════════════════════════════════════
# WEATHER API SERVER
# ═══════════════════════════════════════════════════════════════════════════

create_weather_api_server() {
    log_step "Creating weather API server..."
    
    cat > "$MAR_DIR/mcp-servers/weather_api_server.py" <<'WEATHERAPISERVER'
#!/usr/bin/env python3
"""MCP Weather API Server - Weather data"""
import requests
from typing import Dict

class WeatherAPIServer:
    def __init__(self):
        self.base_url = "https://wttr.in"
    
    def get_weather(self, location: str = "") -> Dict:
        try:
            url = f"{self.base_url}/{location}?format=j1"
            response = requests.get(url, timeout=10)
            if response.status_code == 200:
                data = response.json()
                current = data.get("current_condition", [{}])[0]
                return {
                    "location": location or "Current location",
                    "temp_c": current.get("temp_C"),
                    "temp_f": current.get("temp_F"),
                    "condition": current.get("weatherDesc", [{}])[0].get("value"),
                    "humidity": current.get("humidity"),
                    "wind_kmph": current.get("windspeedKmph")
                }
            return {"error": "Cannot fetch weather"}
        except Exception as e:
            return {"error": str(e)}
    
    def get_forecast(self, location: str = "", days: int = 3) -> Dict:
        try:
            url = f"{self.base_url}/{location}?format=j1"
            response = requests.get(url, timeout=10)
            if response.status_code == 200:
                data = response.json()
                forecasts = []
                for day in data.get("weather", [])[:days]:
                    forecasts.append({
                        "date": day.get("date"),
                        "max_c": day.get("maxtempC"),
                        "min_c": day.get("mintempC"),
                        "condition": day.get("hourly", [{}])[4].get("weatherDesc", [{}])[0].get("value")
                    })
                return {"location": location, "forecast": forecasts}
            return {"error": "Cannot fetch forecast"}
        except Exception as e:
            return {"error": str(e)}
    
    def get_simple(self, location: str = "") -> str:
        try:
            url = f"{self.base_url}/{location}?format=3"
            response = requests.get(url, timeout=5)
            return response.text.strip()
        except:
            return "Cannot fetch weather"
WEATHERAPISERVER

    chmod +x "$MAR_DIR/mcp-servers/weather_api_server.py"
    log_success "Weather API server created"
}

# ═══════════════════════════════════════════════════════════════════════════
# CRYPTO SERVER
# ═══════════════════════════════════════════════════════════════════════════

create_crypto_server() {
    log_step "Creating crypto server..."
    
    cat > "$MAR_DIR/mcp-servers/crypto_server.py" <<'CRYPTOSERVER'
#!/usr/bin/env python3
"""MCP Crypto Server - Cryptocurrency data"""
import requests
from typing import Dict, List

class CryptoServer:
    def __init__(self):
        self.api_url = "https://api.coingecko.com/api/v3"
    
    def get_price(self, coin: str = "bitcoin", currency: str = "usd") -> Dict:
        try:
            url = f"{self.api_url}/simple/price?ids={coin}&vs_currencies={currency}&include_24hr_change=true"
            response = requests.get(url, timeout=5)
            if response.status_code == 200:
                data = response.json()
                if coin in data:
                    return {"coin": coin, "price": data[coin].get(currency), "change_24h": data[coin].get(f"{currency}_24h_change")}
            return {"error": "Coin not found"}
        except Exception as e:
            return {"error": str(e)}
    
    def get_top_coins(self, limit: int = 10) -> List[Dict]:
        try:
            url = f"{self.api_url}/coins/markets?vs_currency=usd&order=market_cap_desc&per_page={limit}"
            response = requests.get(url, timeout=5)
            if response.status_code == 200:
                return [{"name": c["name"], "symbol": c["symbol"], "price": c["current_price"], "change_24h": c["price_change_percentage_24h"]} for c in response.json()]
            return []
        except:
            return []
    
    def search_coin(self, query: str) -> List[Dict]:
        try:
            url = f"{self.api_url}/search?query={query}"
            response = requests.get(url, timeout=5)
            if response.status_code == 200:
                return [{"id": c["id"], "name": c["name"], "symbol": c["symbol"]} for c in response.json().get("coins", [])[:10]]
            return []
        except:
            return []
CRYPTOSERVER

    chmod +x "$MAR_DIR/mcp-servers/crypto_server.py"
    log_success "Crypto server created"
}

# ═══════════════════════════════════════════════════════════════════════════
# NEWS SERVER
# ═══════════════════════════════════════════════════════════════════════════

create_news_server() {
    log_step "Creating news server..."
    
    cat > "$MAR_DIR/mcp-servers/news_server.py" <<'NEWSSERVER'
#!/usr/bin/env python3
"""MCP News Server - News headlines"""
import requests
from bs4 import BeautifulSoup
from typing import Dict, List

class NewsServer:
    def get_headlines(self, source: str = "bbc") -> List[Dict]:
        sources = {
            "bbc": "https://feeds.bbci.co.uk/news/rss.xml",
            "cnn": "http://rss.cnn.com/rss/edition.rss",
            "reuters": "https://www.reutersagency.com/feed/"
        }
        url = sources.get(source.lower(), sources["bbc"])
        try:
            response = requests.get(url, timeout=10)
            soup = BeautifulSoup(response.content, 'xml')
            items = soup.find_all('item')[:10]
            return [{"title": item.find('title').text, "link": item.find('link').text} for item in items if item.find('title')]
        except Exception as e:
            return [{"error": str(e)}]
    
    def search_news(self, query: str) -> List[Dict]:
        from duckduckgo_search import DDGS
        try:
            with DDGS() as ddgs:
                results = list(ddgs.news(query, max_results=10))
                return [{"title": r["title"], "url": r["url"], "source": r.get("source")} for r in results]
        except Exception as e:
            return [{"error": str(e)}]
NEWSSERVER

    chmod +x "$MAR_DIR/mcp-servers/news_server.py"
    log_success "News server created"
}

# ═══════════════════════════════════════════════════════════════════════════
# GITHUB SERVER
# ═══════════════════════════════════════════════════════════════════════════

create_github_server() {
    log_step "Creating GitHub server..."
    
    cat > "$MAR_DIR/mcp-servers/github_server.py" <<'GITHUBSERVER'
#!/usr/bin/env python3
"""MCP GitHub Server - GitHub API operations"""
import requests
from typing import Dict, List

class GitHubServer:
    def __init__(self):
        self.api_url = "https://api.github.com"
        self.headers = {"Accept": "application/vnd.github.v3+json"}
    
    def get_user(self, username: str) -> Dict:
        try:
            response = requests.get(f"{self.api_url}/users/{username}", headers=self.headers, timeout=5)
            if response.status_code == 200:
                data = response.json()
                return {"login": data["login"], "name": data.get("name"), "repos": data["public_repos"], "followers": data["followers"]}
            return {"error": "User not found"}
        except Exception as e:
            return {"error": str(e)}
    
    def get_repos(self, username: str, limit: int = 10) -> List[Dict]:
        try:
            response = requests.get(f"{self.api_url}/users/{username}/repos?sort=updated&per_page={limit}", headers=self.headers, timeout=5)
            if response.status_code == 200:
                return [{"name": r["name"], "description": r.get("description", "")[:100], "stars": r["stargazers_count"], "language": r.get("language")} for r in response.json()]
            return []
        except:
            return []
    
    def search_repos(self, query: str, limit: int = 10) -> List[Dict]:
        try:
            response = requests.get(f"{self.api_url}/search/repositories?q={query}&per_page={limit}", headers=self.headers, timeout=5)
            if response.status_code == 200:
                return [{"name": r["full_name"], "description": r.get("description", "")[:100], "stars": r["stargazers_count"]} for r in response.json().get("items", [])]
            return []
        except:
            return []
    
    def trending(self) -> List[Dict]:
        # Get trending via search
        return self.search_repos("stars:>1000 pushed:>2024-01-01", 10)
GITHUBSERVER

    chmod +x "$MAR_DIR/mcp-servers/github_server.py"
    log_success "GitHub server created"
}

# ═══════════════════════════════════════════════════════════════════════════
# ADDITIONAL PLUGINS
# ═══════════════════════════════════════════════════════════════════════════

create_additional_plugins() {
    log_step "Creating additional plugins..."
    
    # Todo plugin
    mkdir -p "$MAR_DIR/plugins/todo"
    cat > "$MAR_DIR/plugins/todo/plugin.py" <<'TODOPLUGIN'
#!/usr/bin/env python3
"""Todo Plugin for Marceline"""
import sqlite3
import os
from datetime import datetime

DB_PATH = os.path.expanduser("~/marceline/data/memory/marceline.db")

def add_todo(text: str) -> str:
    item = text.lower().replace("add todo", "").replace("add task", "").strip()
    if not item:
        return "What would you like me to add to your todo list?"
    conn = sqlite3.connect(DB_PATH)
    c = conn.cursor()
    c.execute("INSERT INTO todos (task, created_at) VALUES (?, ?)", (item, datetime.now().isoformat()))
    conn.commit()
    conn.close()
    return f"Added to todo: {item}"

def list_todos(text: str) -> str:
    conn = sqlite3.connect(DB_PATH)
    c = conn.cursor()
    c.execute("SELECT id, task, completed FROM todos WHERE completed = 0 ORDER BY created_at DESC LIMIT 10")
    rows = c.fetchall()
    conn.close()
    if not rows:
        return "Your todo list is empty!"
    items = [f"{i+1}. {r[1]}" for i, r in enumerate(rows)]
    return "Your todos:\n" + "\n".join(items)

def complete_todo(text: str) -> str:
    import re
    match = re.search(r'(\d+)', text)
    if match:
        num = int(match.group(1))
        conn = sqlite3.connect(DB_PATH)
        c = conn.cursor()
        c.execute("SELECT id FROM todos WHERE completed = 0 ORDER BY created_at DESC LIMIT 10")
        rows = c.fetchall()
        if 0 < num <= len(rows):
            c.execute("UPDATE todos SET completed = 1, completed_at = ? WHERE id = ?", 
                      (datetime.now().isoformat(), rows[num-1][0]))
            conn.commit()
            conn.close()
            return f"Marked todo {num} as complete!"
        conn.close()
    return "Which todo number do you want to complete?"

def register(manager):
    manager.register_command("add todo", add_todo)
    manager.register_command("add task", add_todo)
    manager.register_command("my todos", list_todos)
    manager.register_command("todo list", list_todos)
    manager.register_command("complete todo", complete_todo)
    manager.register_command("done todo", complete_todo)
    return {"name": "Todo", "version": "1.0.0", "description": "Task management", "author": "Marceline Team"}
TODOPLUGIN

    # Translator plugin
    mkdir -p "$MAR_DIR/plugins/translator"
    cat > "$MAR_DIR/plugins/translator/plugin.py" <<'TRANSLATEPLUGIN'
#!/usr/bin/env python3
"""Translator Plugin for Marceline"""
import subprocess
import re

def translate(text: str) -> str:
    # Parse: translate [text] to [language]
    match = re.search(r'translate\s+(.+?)\s+to\s+(\w+)', text.lower())
    if match:
        phrase, lang = match.groups()
        url = f"https://translate.google.com/?sl=auto&tl={lang}&text={phrase.replace(' ', '+')}"
        subprocess.Popen(["firefox", url], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        return f"Opening translation for '{phrase}' to {lang}"
    return "Say 'translate [text] to [language]'"

def register(manager):
    manager.register_command("translate", translate)
    return {"name": "Translator", "version": "1.0.0", "description": "Translation", "author": "Marceline Team"}
TRANSLATEPLUGIN

    # Wikipedia plugin
    mkdir -p "$MAR_DIR/plugins/wiki"
    cat > "$MAR_DIR/plugins/wiki/plugin.py" <<'WIKIPLUGIN'
#!/usr/bin/env python3
"""Wikipedia Plugin for Marceline"""
import subprocess
import requests

def wiki_search(text: str) -> str:
    query = text.lower().replace("wikipedia", "").replace("wiki", "").replace("search", "").strip()
    if not query:
        return "What would you like me to look up on Wikipedia?"
    try:
        url = f"https://en.wikipedia.org/api/rest_v1/page/summary/{query.replace(' ', '_')}"
        response = requests.get(url, timeout=5)
        if response.status_code == 200:
            data = response.json()
            return data.get("extract", "No summary available")[:500]
    except:
        pass
    subprocess.Popen(["firefox", f"https://en.wikipedia.org/wiki/{query.replace(' ', '_')}"],
                     stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    return f"Opening Wikipedia page for {query}"

def register(manager):
    manager.register_command("wikipedia", wiki_search)
    manager.register_command("wiki", wiki_search)
    manager.register_command("look up", wiki_search)
    return {"name": "Wikipedia", "version": "1.0.0", "description": "Wikipedia search", "author": "Marceline Team"}
WIKIPLUGIN

    # Password generator plugin
    mkdir -p "$MAR_DIR/plugins/password"
    cat > "$MAR_DIR/plugins/password/plugin.py" <<'PASSPLUGIN'
#!/usr/bin/env python3
"""Password Generator Plugin for Marceline"""
import random
import string
import subprocess
import re

def generate_password(text: str) -> str:
    match = re.search(r'(\d+)', text)
    length = int(match.group(1)) if match else 16
    length = max(8, min(64, length))
    chars = string.ascii_letters + string.digits + "!@#$%^&*"
    password = ''.join(random.choice(chars) for _ in range(length))
    # Copy to clipboard
    try:
        process = subprocess.Popen(["xclip", "-selection", "clipboard"], stdin=subprocess.PIPE)
        process.communicate(password.encode())
    except:
        pass
    return f"Generated {length}-character password (copied to clipboard): {password}"

def register(manager):
    manager.register_command("generate password", generate_password)
    manager.register_command("new password", generate_password)
    manager.register_command("password", generate_password)
    return {"name": "Password", "version": "1.0.0", "description": "Password generator", "author": "Marceline Team"}
PASSPLUGIN

    # Quote of the day plugin
    mkdir -p "$MAR_DIR/plugins/quotes"
    cat > "$MAR_DIR/plugins/quotes/plugin.py" <<'QUOTEPLUGIN'
#!/usr/bin/env python3
"""Quotes Plugin for Marceline"""
import random
import requests

QUOTES = [
    "The only way to do great work is to love what you do. - Steve Jobs",
    "Innovation distinguishes between a leader and a follower. - Steve Jobs",
    "Stay hungry, stay foolish. - Steve Jobs",
    "Life is what happens when you're busy making other plans. - John Lennon",
    "The future belongs to those who believe in their dreams. - Eleanor Roosevelt",
    "It does not matter how slowly you go as long as you do not stop. - Confucius",
    "Success is not final, failure is not fatal. - Winston Churchill",
    "Be the change you wish to see in the world. - Gandhi",
]

def get_quote(text: str) -> str:
    try:
        response = requests.get("https://api.quotable.io/random", timeout=3)
        if response.status_code == 200:
            data = response.json()
            return f'"{data["content"]}" - {data["author"]}'
    except:
        pass
    return random.choice(QUOTES)

def register(manager):
    manager.register_command("quote", get_quote)
    manager.register_command("inspire", get_quote)
    manager.register_command("motivation", get_quote)
    return {"name": "Quotes", "version": "1.0.0", "description": "Inspirational quotes", "author": "Marceline Team"}
QUOTEPLUGIN

    # Dice/random plugin
    mkdir -p "$MAR_DIR/plugins/random"
    cat > "$MAR_DIR/plugins/random/plugin.py" <<'RANDOMPLUGIN'
#!/usr/bin/env python3
"""Random Plugin for Marceline"""
import random
import re

def roll_dice(text: str) -> str:
    match = re.search(r'(\d+)d(\d+)', text.lower())
    if match:
        num, sides = int(match.group(1)), int(match.group(2))
        num = min(num, 100)
        sides = min(sides, 1000)
        rolls = [random.randint(1, sides) for _ in range(num)]
        total = sum(rolls)
        return f"Rolling {num}d{sides}: {rolls} = {total}"
    return f"Rolling d20: {random.randint(1, 20)}"

def flip_coin(text: str) -> str:
    return f"Coin flip: {'Heads' if random.random() < 0.5 else 'Tails'}"

def random_number(text: str) -> str:
    match = re.search(r'(\d+)\s*(?:to|and|-)\s*(\d+)', text)
    if match:
        low, high = int(match.group(1)), int(match.group(2))
        return f"Random number between {low} and {high}: {random.randint(low, high)}"
    return f"Random number (1-100): {random.randint(1, 100)}"

def register(manager):
    manager.register_command("roll", roll_dice)
    manager.register_command("dice", roll_dice)
    manager.register_command("flip a coin", flip_coin)
    manager.register_command("coin flip", flip_coin)
    manager.register_command("random number", random_number)
    manager.register_command("pick number", random_number)
    return {"name": "Random", "version": "1.0.0", "description": "Dice, coins, random", "author": "Marceline Team"}
RANDOMPLUGIN

    # Unit converter plugin
    mkdir -p "$MAR_DIR/plugins/convert"
    cat > "$MAR_DIR/plugins/convert/plugin.py" <<'CONVERTPLUGIN'
#!/usr/bin/env python3
"""Unit Converter Plugin for Marceline"""
import re

CONVERSIONS = {
    ("km", "miles"): lambda x: x * 0.621371,
    ("miles", "km"): lambda x: x * 1.60934,
    ("kg", "lbs"): lambda x: x * 2.20462,
    ("lbs", "kg"): lambda x: x * 0.453592,
    ("celsius", "fahrenheit"): lambda x: x * 9/5 + 32,
    ("fahrenheit", "celsius"): lambda x: (x - 32) * 5/9,
    ("meters", "feet"): lambda x: x * 3.28084,
    ("feet", "meters"): lambda x: x * 0.3048,
    ("liters", "gallons"): lambda x: x * 0.264172,
    ("gallons", "liters"): lambda x: x * 3.78541,
}

def convert(text: str) -> str:
    match = re.search(r'(\d+\.?\d*)\s*(\w+)\s+(?:to|in)\s+(\w+)', text.lower())
    if match:
        value, from_unit, to_unit = float(match.group(1)), match.group(2), match.group(3)
        for (f, t), func in CONVERSIONS.items():
            if from_unit.startswith(f[:3]) and to_unit.startswith(t[:3]):
                result = func(value)
                return f"{value} {from_unit} = {result:.2f} {to_unit}"
        return f"I don't know how to convert {from_unit} to {to_unit}"
    return "Say 'convert [number] [unit] to [unit]'"

def register(manager):
    manager.register_command("convert", convert)
    return {"name": "Convert", "version": "1.0.0", "description": "Unit conversion", "author": "Marceline Team"}
CONVERTPLUGIN

    # Pomodoro timer plugin
    mkdir -p "$MAR_DIR/plugins/pomodoro"
    cat > "$MAR_DIR/plugins/pomodoro/plugin.py" <<'POMODORO'
#!/usr/bin/env python3
"""Pomodoro Timer Plugin for Marceline"""
import threading
import time
import subprocess

active_pomodoro = None

def notify(title: str, message: str):
    subprocess.run(["notify-send", "-u", "critical", title, message])
    subprocess.run(["espeak-ng", message], capture_output=True)

def start_pomodoro(text: str) -> str:
    global active_pomodoro
    if active_pomodoro and active_pomodoro.is_alive():
        return "A pomodoro session is already running!"
    
    def pomodoro_session():
        notify("Pomodoro Started", "Focus for 25 minutes!")
        time.sleep(25 * 60)
        notify("Pomodoro Complete!", "Take a 5 minute break!")
        time.sleep(5 * 60)
        notify("Break Over", "Ready for another pomodoro?")
    
    active_pomodoro = threading.Thread(target=pomodoro_session, daemon=True)
    active_pomodoro.start()
    return "Starting 25-minute pomodoro focus session!"

def stop_pomodoro(text: str) -> str:
    return "Current pomodoro will complete. Stay focused!"

def register(manager):
    manager.register_command("start pomodoro", start_pomodoro)
    manager.register_command("pomodoro", start_pomodoro)
    manager.register_command("focus mode", start_pomodoro)
    manager.register_command("stop pomodoro", stop_pomodoro)
    return {"name": "Pomodoro", "version": "1.0.0", "description": "Pomodoro timer", "author": "Marceline Team"}
POMODORO

    # Ensure todo table exists
    cat > "$MAR_DIR/data/init_todos.sql" <<'INITSQL'
CREATE TABLE IF NOT EXISTS todos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task TEXT NOT NULL,
    completed INTEGER DEFAULT 0,
    created_at TEXT,
    completed_at TEXT
);
INITSQL

    chown -R "$REAL_USER:$REAL_USER" "$MAR_DIR/plugins"
    log_success "Additional plugins created (8 more plugins)"
}

# ═══════════════════════════════════════════════════════════════════════════
# SYSTEM MONITOR SERVER
# ═══════════════════════════════════════════════════════════════════════════

create_system_monitor() {
    log_step "Creating system monitor..."
    
    cat > "$MAR_DIR/mcp-servers/monitor_server.py" <<'MONITORSERVER'
#!/usr/bin/env python3
"""MCP System Monitor Server - Real-time system stats"""
import psutil
import subprocess
from datetime import datetime, timedelta
from typing import Dict, List

class MonitorServer:
    def get_full_status(self) -> Dict:
        cpu = psutil.cpu_percent(interval=1)
        mem = psutil.virtual_memory()
        disk = psutil.disk_usage("/")
        boot = datetime.fromtimestamp(psutil.boot_time())
        uptime = datetime.now() - boot
        
        return {
            "cpu_percent": cpu,
            "memory": {"total_gb": mem.total / (1024**3), "used_percent": mem.percent},
            "disk": {"total_gb": disk.total / (1024**3), "used_percent": disk.percent},
            "uptime": str(uptime).split(".")[0],
            "timestamp": datetime.now().isoformat()
        }
    
    def get_top_processes(self, limit: int = 10) -> List[Dict]:
        procs = []
        for p in psutil.process_iter(['pid', 'name', 'cpu_percent', 'memory_percent']):
            try:
                procs.append(p.info)
            except:
                pass
        return sorted(procs, key=lambda x: x.get('cpu_percent', 0), reverse=True)[:limit]
    
    def get_network(self) -> Dict:
        net = psutil.net_io_counters()
        return {
            "bytes_sent": net.bytes_sent,
            "bytes_recv": net.bytes_recv,
            "packets_sent": net.packets_sent,
            "packets_recv": net.packets_recv
        }
    
    def get_disk_io(self) -> Dict:
        io = psutil.disk_io_counters()
        return {
            "read_bytes": io.read_bytes,
            "write_bytes": io.write_bytes,
            "read_count": io.read_count,
            "write_count": io.write_count
        }
    
    def get_temps(self) -> Dict:
        try:
            temps = psutil.sensors_temperatures()
            return {k: [{"label": s.label, "current": s.current} for s in v] for k, v in temps.items()}
        except:
            return {"error": "Temperature sensors not available"}
    
    def get_battery(self) -> Dict:
        bat = psutil.sensors_battery()
        if bat:
            return {
                "percent": bat.percent,
                "plugged": bat.power_plugged,
                "time_left": str(timedelta(seconds=bat.secsleft)) if bat.secsleft > 0 else "Charging"
            }
        return {"error": "No battery"}
    
    def get_users(self) -> List[Dict]:
        return [{"name": u.name, "terminal": u.terminal, "host": u.host} for u in psutil.users()]
    
    def kill_process(self, pid: int) -> Dict:
        try:
            p = psutil.Process(pid)
            p.terminate()
            return {"success": True, "pid": pid}
        except Exception as e:
            return {"error": str(e)}
MONITORSERVER

    chmod +x "$MAR_DIR/mcp-servers/monitor_server.py"
    log_success "System monitor created"
}

# ═══════════════════════════════════════════════════════════════════════════
# EXTENDED VOICE COMMANDS
# ═══════════════════════════════════════════════════════════════════════════

create_extended_commands() {
    log_step "Creating extended voice commands..."
    
    cat > "$MAR_DIR/assistant-core/commands.py" <<'EXTCMDS'
#!/usr/bin/env python3
"""Extended Command Set for Marceline"""
import subprocess
import os
import json
from datetime import datetime
from typing import Dict, Callable, Optional

class ExtendedCommands:
    def __init__(self):
        self.commands: Dict[str, Callable] = {}
        self._register_all()
    
    def _register_all(self):
        # System
        self.commands["shutdown"] = lambda: self._system_cmd("shutdown now")
        self.commands["restart"] = lambda: self._system_cmd("reboot")
        self.commands["lock screen"] = lambda: self._run("gnome-screensaver-command -l")
        self.commands["log out"] = lambda: self._run("gnome-session-quit --logout")
        self.commands["sleep"] = lambda: self._run("systemctl suspend")
        
        # Apps
        self.commands["open terminal"] = lambda: self._app("gnome-terminal")
        self.commands["open files"] = lambda: self._app("nautilus")
        self.commands["open browser"] = lambda: self._app("firefox")
        self.commands["open settings"] = lambda: self._app("gnome-control-center")
        self.commands["open calculator"] = lambda: self._app("gnome-calculator")
        self.commands["open text editor"] = lambda: self._app("gedit")
        self.commands["open code"] = lambda: self._app("code")
        self.commands["open spotify"] = lambda: self._app("spotify")
        
        # Media
        self.commands["play music"] = lambda: self._mpris("Play")
        self.commands["pause music"] = lambda: self._mpris("PlayPause")
        self.commands["next song"] = lambda: self._mpris("Next")
        self.commands["previous song"] = lambda: self._mpris("Previous")
        self.commands["volume up"] = lambda: self._volume("+10%")
        self.commands["volume down"] = lambda: self._volume("-10%")
        self.commands["mute"] = lambda: self._volume("toggle")
        
        # Screen
        self.commands["screenshot"] = self._screenshot
        self.commands["screen capture"] = self._screenshot
        self.commands["brightness up"] = lambda: self._brightness("+10")
        self.commands["brightness down"] = lambda: self._brightness("-10")
        
        # Network
        self.commands["wifi on"] = lambda: self._run("nmcli radio wifi on")
        self.commands["wifi off"] = lambda: self._run("nmcli radio wifi off")
        self.commands["airplane mode"] = lambda: self._run("nmcli radio all off")
        self.commands["check internet"] = self._check_internet
        self.commands["my ip"] = self._get_ip
        
        # Info
        self.commands["what time is it"] = lambda: f"It's {datetime.now().strftime('%I:%M %p')}"
        self.commands["what day is it"] = lambda: f"Today is {datetime.now().strftime('%A, %B %d, %Y')}"
        self.commands["battery status"] = self._battery
        self.commands["disk space"] = self._disk_space
        self.commands["memory usage"] = self._memory
        self.commands["cpu usage"] = self._cpu
    
    def _run(self, cmd: str) -> str:
        subprocess.run(cmd.split(), capture_output=True)
        return "Done"
    
    def _app(self, name: str) -> str:
        subprocess.Popen([name], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        return f"Opening {name}"
    
    def _system_cmd(self, cmd: str) -> str:
        subprocess.run(["sudo", *cmd.split()], capture_output=True)
        return f"Executing {cmd}"
    
    def _mpris(self, action: str) -> str:
        subprocess.run(["playerctl", action.lower()], capture_output=True)
        return f"Media: {action}"
    
    def _volume(self, change: str) -> str:
        subprocess.run(["amixer", "-q", "set", "Master", change], capture_output=True)
        return f"Volume: {change}"
    
    def _brightness(self, change: str) -> str:
        subprocess.run(["xbacklight", change], capture_output=True)
        return f"Brightness: {change}"
    
    def _screenshot(self) -> str:
        path = os.path.expanduser(f"~/Pictures/screenshot_{datetime.now().strftime('%Y%m%d_%H%M%S')}.png")
        subprocess.run(["scrot", path], capture_output=True)
        return f"Screenshot saved to {path}"
    
    def _check_internet(self) -> str:
        result = subprocess.run(["ping", "-c", "1", "8.8.8.8"], capture_output=True)
        return "Internet is connected" if result.returncode == 0 else "No internet connection"
    
    def _get_ip(self) -> str:
        result = subprocess.run(["hostname", "-I"], capture_output=True, text=True)
        return f"Your IP is {result.stdout.split()[0]}" if result.stdout else "Could not get IP"
    
    def _battery(self) -> str:
        import psutil
        bat = psutil.sensors_battery()
        if bat:
            status = "charging" if bat.power_plugged else "on battery"
            return f"Battery is at {bat.percent}%, {status}"
        return "No battery detected"
    
    def _disk_space(self) -> str:
        import psutil
        disk = psutil.disk_usage("/")
        free_gb = (disk.total - disk.used) / (1024**3)
        return f"You have {free_gb:.1f} GB free disk space ({100 - disk.percent:.0f}% available)"
    
    def _memory(self) -> str:
        import psutil
        mem = psutil.virtual_memory()
        return f"Memory usage: {mem.percent}% ({mem.used / (1024**3):.1f} GB of {mem.total / (1024**3):.1f} GB)"
    
    def _cpu(self) -> str:
        import psutil
        return f"CPU usage: {psutil.cpu_percent(interval=1)}%"
    
    def process(self, text: str) -> Optional[str]:
        text_lower = text.lower()
        for trigger, handler in self.commands.items():
            if trigger in text_lower:
                return handler()
        return None
EXTCMDS

    chmod +x "$MAR_DIR/assistant-core/commands.py"
    chown "$REAL_USER:$REAL_USER" "$MAR_DIR/assistant-core/commands.py"
    log_success "Extended commands created (40+ commands)"
}

# ═══════════════════════════════════════════════════════════════════════════
# MAIN INSTALLATION
# ═══════════════════════════════════════════════════════════════════════════

main() {
    show_banner
    check_root
    detect_user
    
    echo ""
    log_info "Starting Marceline OS installation..."
    echo ""
    
    install_system_packages
    install_ollama
    create_directory_structure
    setup_python_environment
    install_vosk_model
    install_piper_tts
    create_main_config
    create_voice_assistant
    create_gui_overlay
    create_mcp_servers
    create_web_api
    create_plugin_system
    create_theme_system
    create_smart_home_stubs
    create_email_server
    create_vision_server
    create_code_server
    create_music_server
    create_browser_server
    create_download_server
    create_time_server
    create_fetch_server
    create_screenshot_server
    create_process_server
    create_network_server
    create_currency_server
    create_weather_api_server
    create_crypto_server
    create_news_server
    create_github_server
    create_additional_plugins
    create_system_monitor
    create_extended_commands
    create_backup_system
    create_update_mechanism
    create_utilities
    create_systemd_services
    create_autostart
    create_launchers
    setup_audio_permissions
    start_services
    
    show_completion
}

# Run main
main "$@"
