#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════════
# MARCELINE AI - Linux Installation Script
# Voice-activated AI Assistant with Gemini/Ollama integration
# ═══════════════════════════════════════════════════════════════════════════════

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[✓]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[!]${NC} $1"; }
log_error() { echo -e "${RED}[✗]${NC} $1"; }

# Configuration
INSTALL_DIR="$HOME/marceline"
VERSION="2.0.0"

show_banner() {
    echo -e "${PURPLE}"
    echo "╔═══════════════════════════════════════════════════════════════════════════╗"
    echo "║   ███╗   ███╗ █████╗ ██████╗  ██████╗███████╗██╗     ██╗███╗   ██╗███████╗║"
    echo "║   ████╗ ████║██╔══██╗██╔══██╗██╔════╝██╔════╝██║     ██║████╗  ██║██╔════╝║"
    echo "║   ██╔████╔██║███████║██████╔╝██║     █████╗  ██║     ██║██╔██╗ ██║█████╗  ║"
    echo "║   ██║╚██╔╝██║██╔══██║██╔══██╗██║     ██╔══╝  ██║     ██║██║╚██╗██║██╔══╝  ║"
    echo "║   ██║ ╚═╝ ██║██║  ██║██║  ██║╚██████╗███████╗███████╗██║██║ ╚████║███████╗║"
    echo "║   ╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝╚══════╝╚══════╝╚═╝╚═╝  ╚═══╝╚══════╝║"
    echo "║                     Linux AI Voice Assistant v${VERSION}                      ║"
    echo "╚═══════════════════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

install_dependencies() {
    log_info "Installing system dependencies..."
    
    if command -v apt-get &> /dev/null; then
        sudo apt-get update -qq
        sudo apt-get install -y \
            python3 python3-pip python3-venv python3-dev \
            espeak-ng portaudio19-dev python3-pyaudio \
            curl wget git jq ffmpeg \
            xclip scrot wmctrl xdotool libnotify-bin \
            sqlite3 build-essential
    elif command -v dnf &> /dev/null; then
        sudo dnf install -y \
            python3 python3-pip python3-devel \
            espeak-ng portaudio-devel \
            curl wget git jq ffmpeg \
            xclip scrot wmctrl xdotool libnotify \
            sqlite gcc gcc-c++
    elif command -v pacman &> /dev/null; then
        sudo pacman -Sy --noconfirm \
            python python-pip \
            espeak-ng portaudio \
            curl wget git jq ffmpeg \
            xclip scrot wmctrl xdotool libnotify \
            sqlite base-devel
    else
        log_warning "Package manager not detected. Install dependencies manually."
    fi
    
    log_success "System dependencies installed"
}

install_ollama() {
    log_info "Installing Ollama AI..."
    
    if command -v ollama &> /dev/null; then
        log_info "Ollama already installed"
    else
        curl -fsSL https://ollama.com/install.sh | sh
    fi
    
    # Start Ollama service
    if systemctl is-active --quiet ollama; then
        log_info "Ollama service running"
    else
        sudo systemctl enable ollama
        sudo systemctl start ollama
        sleep 5
    fi
    
    # Pull lightweight model
    log_info "Pulling AI model (this may take a few minutes)..."
    ollama pull llama3.2:3b || log_warning "Could not pull model"
    
    log_success "Ollama ready"
}

setup_directories() {
    log_info "Creating directory structure..."
    
    mkdir -p "$INSTALL_DIR"/{assistant,models,data,logs,config}
    mkdir -p "$INSTALL_DIR"/data/{notes,memory,conversations}
    
    log_success "Directories created"
}

setup_python_env() {
    log_info "Setting up Python environment..."
    
    cd "$INSTALL_DIR"
    python3 -m venv venv
    
    source venv/bin/activate
    pip install --upgrade pip
    pip install -q \
        requests \
        SpeechRecognition \
        pyaudio \
        rich \
        typer \
        httpx \
        python-dotenv \
        pyyaml \
        duckduckgo-search \
        edge-tts
    
    log_success "Python environment ready"
}

create_voice_assistant() {
    log_info "Creating Marceline voice assistant..."
    
    cat > "$INSTALL_DIR/assistant/marceline.py" << 'PYEOF'
#!/usr/bin/env python3
"""
Marceline - Linux Voice-Activated AI Assistant
Uses Gemini API (with Ollama fallback) + Speech Recognition
"""

import os
import sys
import time
import json
import subprocess
import sqlite3
import requests
import speech_recognition as sr
from pathlib import Path
from datetime import datetime
from typing import Optional, Dict, List
from dataclasses import dataclass, field
import threading

# Configuration
GEMINI_API_KEY = os.getenv("GEMINI_API_KEY", "")
OLLAMA_URL = "http://localhost:11434/api/generate"
OLLAMA_MODEL = "llama3.2:3b"
DB_PATH = os.path.expanduser("~/marceline/data/memory/marceline.db")
WAKE_WORDS = ["marceline", "hey marceline", "marcy", "hey marcy", "computer"]

class Colors:
    RED = '\033[0;31m'
    GREEN = '\033[0;32m'
    YELLOW = '\033[1;33m'
    BLUE = '\033[0;34m'
    PURPLE = '\033[0;35m'
    CYAN = '\033[0;36m'
    NC = '\033[0m'

def log(msg: str, color: str = Colors.NC):
    ts = datetime.now().strftime("%H:%M:%S")
    print(f"{color}[{ts}]{Colors.NC} {msg}")

@dataclass  
class Context:
    messages: List[Dict] = field(default_factory=list)
    user_name: Optional[str] = None
    last_topic: Optional[str] = None

# ═══════════════════════════════════════════════════════════════════
# AI PROVIDERS
# ═══════════════════════════════════════════════════════════════════

def call_gemini(prompt: str, context: Context) -> str:
    """Call Gemini API"""
    if not GEMINI_API_KEY:
        return call_ollama(prompt, context)
    
    try:
        url = f"https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash-exp:generateContent?key={GEMINI_API_KEY}"
        
        system = "You are Marceline, a helpful voice AI assistant. Keep responses brief (2-3 sentences) and conversational."
        
        # Build history
        contents = [{"role": "user", "parts": [{"text": system}]}, 
                   {"role": "model", "parts": [{"text": "I understand. I'm Marceline, ready to help!"}]}]
        
        for msg in context.messages[-10:]:
            contents.append({
                "role": "user" if msg["role"] == "user" else "model",
                "parts": [{"text": msg["content"]}]
            })
        
        contents.append({"role": "user", "parts": [{"text": prompt}]})
        
        response = requests.post(url, json={"contents": contents}, timeout=30)
        if response.status_code == 200:
            data = response.json()
            return data["candidates"][0]["content"]["parts"][0]["text"]
    except Exception as e:
        log(f"Gemini error: {e}", Colors.YELLOW)
    
    return call_ollama(prompt, context)

def call_ollama(prompt: str, context: Context) -> str:
    """Call local Ollama as fallback"""
    try:
        system = "You are Marceline, a helpful voice AI. Keep responses very brief."
        history = "\n".join([f"{m['role']}: {m['content']}" for m in context.messages[-5:]])
        full_prompt = f"{system}\n\n{history}\n\nUser: {prompt}\nMarceline:"
        
        response = requests.post(OLLAMA_URL, json={
            "model": OLLAMA_MODEL,
            "prompt": full_prompt,
            "stream": False,
            "options": {"num_predict": 100}
        }, timeout=60)
        
        if response.status_code == 200:
            return response.json().get("response", "I couldn't process that.")
    except Exception as e:
        log(f"Ollama error: {e}", Colors.RED)
    
    return "Sorry, I'm having trouble connecting to my AI brain right now."

# ═══════════════════════════════════════════════════════════════════
# SPEECH ENGINE
# ═══════════════════════════════════════════════════════════════════

class SpeechEngine:
    def __init__(self):
        self.recognizer = sr.Recognizer()
        self.recognizer.energy_threshold = 300
        self.recognizer.dynamic_energy_threshold = True
        self.recognizer.pause_threshold = 0.8
        self.microphone = None
        self._init_mic()
    
    def _init_mic(self):
        try:
            self.microphone = sr.Microphone()
            with self.microphone as source:
                self.recognizer.adjust_for_ambient_noise(source, duration=0.5)
            log("Microphone initialized", Colors.GREEN)
        except Exception as e:
            log(f"Microphone error: {e}", Colors.RED)
    
    def listen(self, timeout: float = 3.0) -> str:
        if not self.microphone:
            return ""
        try:
            with self.microphone as source:
                log("Listening...", Colors.CYAN)
                audio = self.recognizer.listen(source, timeout=timeout, phrase_time_limit=6)
            
            # Use Google Speech Recognition (free)
            text = self.recognizer.recognize_google(audio)
            log(f"Heard: {text}", Colors.GREEN)
            return text.lower()
        except sr.WaitTimeoutError:
            return ""
        except sr.UnknownValueError:
            return ""
        except Exception as e:
            log(f"Recognition error: {e}", Colors.YELLOW)
            return ""

def speak(text: str):
    """Text-to-speech using espeak-ng"""
    try:
        # Clean text
        clean = text.replace('"', '\\"').replace("'", "\\'")
        subprocess.run(
            ["espeak-ng", "-v", "en-gb+f4", "-s", "160", "-p", "55", clean],
            capture_output=True
        )
    except Exception as e:
        log(f"TTS error: {e}", Colors.YELLOW)

# ═══════════════════════════════════════════════════════════════════
# COMMAND PROCESSING
# ═══════════════════════════════════════════════════════════════════

def process_command(text: str, context: Context) -> str:
    """Process user commands"""
    text_lower = text.lower()
    
    # Time
    if any(w in text_lower for w in ["time", "what time", "clock"]):
        return f"It's currently {datetime.now().strftime('%I:%M %p')}"
    
    # Date
    if any(w in text_lower for w in ["date", "what day", "today"]):
        return datetime.now().strftime("Today is %A, %B %d, %Y")
    
    # Open apps  
    if "open" in text_lower:
        apps = {
            "firefox": "firefox", "browser": "firefox",
            "terminal": "gnome-terminal", "files": "nautilus",
            "code": "code", "vscode": "code",
            "spotify": "spotify", "music": "spotify"
        }
        for name, cmd in apps.items():
            if name in text_lower:
                try:
                    subprocess.Popen([cmd], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
                    return f"Opening {name}!"
                except:
                    return f"Sorry, I couldn't open {name}"
    
    # Screenshot
    if "screenshot" in text_lower:
        try:
            path = os.path.expanduser(f"~/Pictures/screenshot_{int(time.time())}.png")
            subprocess.run(["scrot", path], capture_output=True)
            return f"Screenshot saved!"
        except:
            return "Couldn't take screenshot"
    
    # Volume
    if "volume" in text_lower:
        if "up" in text_lower:
            subprocess.run(["amixer", "-q", "set", "Master", "10%+"], capture_output=True)
            return "Volume up!"
        elif "down" in text_lower:
            subprocess.run(["amixer", "-q", "set", "Master", "10%-"], capture_output=True)
            return "Volume down!"
        elif "mute" in text_lower:
            subprocess.run(["amixer", "-q", "set", "Master", "toggle"], capture_output=True)
            return "Toggled mute"
    
    # Web search
    if any(w in text_lower for w in ["search for", "look up", "google"]):
        query = text_lower.replace("search for", "").replace("look up", "").replace("google", "").strip()
        if query:
            url = f"https://duckduckgo.com/?q={query.replace(' ', '+')}"
            subprocess.Popen(["firefox", url], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            return f"Searching for {query}"
    
    # AI response
    response = call_gemini(text, context)
    
    # Update context
    context.messages.append({"role": "user", "content": text})
    context.messages.append({"role": "assistant", "content": response})
    
    # Trim context
    if len(context.messages) > 20:
        context.messages = context.messages[-20:]
    
    return response

def check_wake_word(text: str) -> tuple[bool, str]:
    """Check for wake word and extract command"""
    text_lower = text.lower()
    
    for wake in WAKE_WORDS:
        if wake in text_lower:
            idx = text_lower.find(wake)
            command = text[idx + len(wake):].strip()
            return True, command
    
    return False, ""

# ═══════════════════════════════════════════════════════════════════
# MAIN LOOP
# ═══════════════════════════════════════════════════════════════════

def main():
    print()
    print(f"{Colors.PURPLE}{'═' * 60}{Colors.NC}")
    print(f"{Colors.PURPLE}  🧛 MARCELINE - Voice AI Assistant{Colors.NC}")
    print(f"{Colors.PURPLE}  Say 'Hey Marceline' followed by your command{Colors.NC}")
    print(f"{Colors.PURPLE}{'═' * 60}{Colors.NC}")
    print()
    
    context = Context()
    speech = SpeechEngine()
    active = False
    active_timeout = 0
    
    # Greeting
    speak("Hey! I'm Marceline. Say my name when you need me!")
    
    try:
        while True:
            text = speech.listen(timeout=3.0)
            
            if not text:
                # Check if we should deactivate
                if active and time.time() > active_timeout:
                    active = False
                    log("Going back to standby...", Colors.BLUE)
                continue
            
            # Check for wake word if not active
            if not active:
                woke, command = check_wake_word(text)
                if woke:
                    active = True
                    active_timeout = time.time() + 30  # Stay active for 30 seconds
                    
                    if command:
                        log(f"Command: {command}", Colors.CYAN)
                        response = process_command(command, context)
                        log(f"Response: {response}", Colors.GREEN)
                        speak(response)
                    else:
                        speak("Yes? I'm listening!")
                continue
            
            # Active mode - process everything
            active_timeout = time.time() + 30  # Reset timeout
            
            # Exit commands
            if any(w in text for w in ["goodbye", "bye", "stop", "exit", "quit"]):
                speak("Goodbye! Call me if you need anything!")
                active = False
                continue
            
            response = process_command(text, context)
            log(f"Response: {response}", Colors.GREEN)
            speak(response)
            
    except KeyboardInterrupt:
        print(f"\n{Colors.YELLOW}Marceline shutting down...{Colors.NC}")
        speak("Goodbye!")
        sys.exit(0)

if __name__ == "__main__":
    main()
PYEOF

    chmod +x "$INSTALL_DIR/assistant/marceline.py"
    log_success "Voice assistant created"
}

create_config() {
    log_info "Creating configuration..."
    
    cat > "$INSTALL_DIR/config/config.json" << EOF
{
    "version": "$VERSION",
    "name": "Marceline",
    "wake_words": ["marceline", "hey marceline", "marcy", "computer"],
    "voice": {
        "engine": "espeak-ng",
        "voice": "en-gb+f4",
        "rate": 160,
        "pitch": 55
    },
    "llm": {
        "primary": "gemini",
        "fallback": "ollama",
        "model": "llama3.2:3b"
    },
    "features": {
        "voice_activation": true,
        "web_search": true,
        "system_control": true
    }
}
EOF
    
    # Create .env file
    cat > "$INSTALL_DIR/.env" << EOF
# Gemini API Key (optional, falls back to Ollama)
GEMINI_API_KEY=

# Ollama settings
OLLAMA_HOST=http://localhost:11434
OLLAMA_MODEL=llama3.2:3b
EOF

    log_success "Configuration created"
}

create_launcher() {
    log_info "Creating launcher script..."
    
    cat > "$INSTALL_DIR/run.sh" << 'RUNEOF'
#!/bin/bash
cd "$(dirname "$0")"
source .env 2>/dev/null || true
source venv/bin/activate
export GEMINI_API_KEY="${GEMINI_API_KEY}"
python assistant/marceline.py "$@"
RUNEOF
    chmod +x "$INSTALL_DIR/run.sh"
    
    # Create symlink
    if [[ -d /usr/local/bin ]]; then
        sudo ln -sf "$INSTALL_DIR/run.sh" /usr/local/bin/marceline
        log_success "You can now run 'marceline' from anywhere!"
    fi
    
    log_success "Launcher created"
}

show_completion() {
    echo ""
    echo -e "${GREEN}"
    echo "╔═══════════════════════════════════════════════════════════════════════════╗"
    echo "║                                                                           ║"
    echo "║   ✅ MARCELINE INSTALLED SUCCESSFULLY!                                    ║"
    echo "║                                                                           ║"
    echo "╠═══════════════════════════════════════════════════════════════════════════╣"
    echo "║                                                                           ║"
    echo "║   🎤 TO START:                                                            ║"
    echo "║      marceline            (or: ~/marceline/run.sh)                        ║"
    echo "║                                                                           ║"
    echo "║   🔧 OPTIONAL: Add Gemini API key for better responses:                   ║"
    echo "║      Edit: ~/marceline/.env                                               ║"
    echo "║      Add: GEMINI_API_KEY=your_key_here                                    ║"
    echo "║                                                                           ║"
    echo "║   🎙️  VOICE COMMANDS:                                                     ║"
    echo "║      • Say 'Hey Marceline' to activate                                    ║"
    echo "║      • 'What time is it?'                                                 ║"
    echo "║      • 'Open Firefox'                                                     ║" 
    echo "║      • 'Take a screenshot'                                                ║"
    echo "║      • 'Volume up/down'                                                   ║"
    echo "║      • 'Search for...'                                                    ║"
    echo "║      • Any question - uses AI!                                            ║"
    echo "║                                                                           ║"
    echo "╚═══════════════════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

# Main installation
main() {
    show_banner
    
    log_info "Starting Marceline installation..."
    echo ""
    
    install_dependencies
    install_ollama
    setup_directories
    setup_python_env
    create_voice_assistant
    create_config
    create_launcher
    
    show_completion
}

main "$@"
