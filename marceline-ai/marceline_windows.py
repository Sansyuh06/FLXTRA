#!/usr/bin/env python3
"""
Marceline - Windows Voice-Activated AI Assistant
Uses Gemini API + Edge-TTS (Microsoft Neural Voices)
With logging and retry logic
"""

import os
import sys
import time
import subprocess
import requests
import ctypes
import tempfile
import asyncio
import logging
from pathlib import Path
from datetime import datetime
from typing import Optional, List, Dict
from dataclasses import dataclass, field

# ═══════════════════════════════════════════════════════════════════
# LOGGING SETUP
# ═══════════════════════════════════════════════════════════════════

LOG_DIR = Path(__file__).parent / "logs"
LOG_DIR.mkdir(exist_ok=True)
LOG_FILE = LOG_DIR / f"marceline_{datetime.now().strftime('%Y%m%d')}.log"

# Setup file + console logging
logging.basicConfig(
    level=logging.DEBUG,
    format='%(asctime)s [%(levelname)s] %(message)s',
    datefmt='%Y-%m-%d %H:%M:%S',
    handlers=[
        logging.FileHandler(LOG_FILE, encoding='utf-8'),
        logging.StreamHandler(sys.stdout)
    ]
)
logger = logging.getLogger("Marceline")

# Try to import speech recognition
try:
    import speech_recognition as sr
    HAS_SPEECH = True
except ImportError:
    HAS_SPEECH = False
    logger.error("SpeechRecognition not installed. Run: pip install SpeechRecognition pyaudio")

# Try to import edge-tts
try:
    import edge_tts
    HAS_EDGE_TTS = True
except ImportError:
    HAS_EDGE_TTS = False
    logger.warning("edge-tts not installed. Run: pip install edge-tts")

# Configuration
GEMINI_API_KEY = os.getenv("GEMINI_API_KEY", "AIzaSyDV5mWyQ6Ux0rEgX_2d77BwiN5Bl6Y906k")
WAKE_WORDS = ["marceline", "hey marceline", "marcy", "hey marcy", "computer", "hey computer"]
EDGE_TTS_VOICE = "en-US-AriaNeural"

# Rate limiting
LAST_API_CALL = 0
MIN_API_INTERVAL = 2.0  # Minimum seconds between API calls

class Colors:
    RED = '\033[0;31m'
    GREEN = '\033[0;32m'
    YELLOW = '\033[1;33m'
    BLUE = '\033[0;34m'
    PURPLE = '\033[0;35m'
    CYAN = '\033[0;36m'
    NC = '\033[0m'

# Enable ANSI colors on Windows
if sys.platform == 'win32':
    try:
        kernel32 = ctypes.windll.kernel32
        kernel32.SetConsoleMode(kernel32.GetStdHandle(-11), 7)
    except:
        pass

def log(msg: str, color: str = Colors.NC, level: str = "INFO"):
    ts = datetime.now().strftime("%H:%M:%S")
    print(f"{color}[{ts}]{Colors.NC} {msg}")
    
    # Also log to file
    if level == "ERROR":
        logger.error(msg)
    elif level == "WARNING":
        logger.warning(msg)
    elif level == "DEBUG":
        logger.debug(msg)
    else:
        logger.info(msg)

@dataclass
class Context:
    messages: List[Dict] = field(default_factory=list)
    user_name: Optional[str] = None

# ═══════════════════════════════════════════════════════════════════
# GEMINI AI WITH RETRY
# ═══════════════════════════════════════════════════════════════════

def call_gemini(prompt: str, context: Context, retries: int = 3) -> str:
    """Call Gemini API with retry logic"""
    global LAST_API_CALL
    
    if not GEMINI_API_KEY:
        return "I need a Gemini API key to respond."
    
    # Rate limiting - wait if needed
    elapsed = time.time() - LAST_API_CALL
    if elapsed < MIN_API_INTERVAL:
        wait_time = MIN_API_INTERVAL - elapsed
        logger.debug(f"Rate limiting: waiting {wait_time:.1f}s")
        time.sleep(wait_time)
    
    for attempt in range(retries):
        try:
            url = f"https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash-exp:generateContent?key={GEMINI_API_KEY}"
            
            system = """You are Marceline, a helpful and friendly voice AI assistant for Windows.
Keep responses brief (2-3 sentences) and conversational."""
            
            contents = [
                {"role": "user", "parts": [{"text": system}]},
                {"role": "model", "parts": [{"text": "Got it! I'm Marceline!"}]}
            ]
            
            for msg in context.messages[-10:]:
                contents.append({
                    "role": "user" if msg["role"] == "user" else "model",
                    "parts": [{"text": msg["content"]}]
                })
            
            contents.append({"role": "user", "parts": [{"text": prompt}]})
            
            LAST_API_CALL = time.time()
            response = requests.post(url, json={"contents": contents}, timeout=30)
            
            logger.debug(f"Gemini API response: {response.status_code}")
            
            if response.status_code == 200:
                data = response.json()
                return data["candidates"][0]["content"]["parts"][0]["text"]
            elif response.status_code == 429:
                wait = (attempt + 1) * 5  # Exponential backoff
                logger.warning(f"Rate limited (429). Waiting {wait}s before retry {attempt+1}/{retries}")
                log(f"Rate limited, waiting {wait}s...", Colors.YELLOW, "WARNING")
                time.sleep(wait)
                continue
            else:
                error_text = response.text[:200]
                logger.error(f"Gemini API error {response.status_code}: {error_text}")
                
        except requests.exceptions.Timeout:
            logger.error("Gemini API timeout")
            log("Request timed out", Colors.RED, "ERROR")
        except Exception as e:
            logger.error(f"Gemini API exception: {str(e)}")
            log(f"Error: {e}", Colors.RED, "ERROR")
        
        if attempt < retries - 1:
            time.sleep(2)
    
    return "I'm having trouble connecting. Please try again in a moment."

# ═══════════════════════════════════════════════════════════════════
# SPEECH ENGINE
# ═══════════════════════════════════════════════════════════════════

class SpeechEngine:
    def __init__(self):
        if not HAS_SPEECH:
            self.recognizer = None
            self.microphone = None
            return
            
        self.recognizer = sr.Recognizer()
        self.recognizer.energy_threshold = 300
        self.recognizer.dynamic_energy_threshold = True
        self.recognizer.pause_threshold = 0.8
        self.microphone = None
        self._init_mic()
    
    def _init_mic(self):
        try:
            mics = sr.Microphone.list_microphone_names()
            logger.info(f"Found {len(mics)} microphone(s)")
            
            self.microphone = sr.Microphone()
            with self.microphone as source:
                self.recognizer.adjust_for_ambient_noise(source, duration=0.5)
            log("✅ Microphone initialized", Colors.GREEN)
            logger.info("Microphone initialized successfully")
        except Exception as e:
            log(f"❌ Microphone error: {e}", Colors.RED, "ERROR")
    
    def listen(self, timeout: float = 3.0) -> str:
        if not self.microphone or not self.recognizer:
            return ""
        
        try:
            with self.microphone as source:
                log("🎤 Listening...", Colors.CYAN)
                audio = self.recognizer.listen(source, timeout=timeout, phrase_time_limit=6)
            
            text = self.recognizer.recognize_google(audio)
            log(f"📝 Heard: {text}", Colors.GREEN)
            logger.info(f"Speech recognized: {text}")
            return text.lower()
            
        except sr.WaitTimeoutError:
            return ""
        except sr.UnknownValueError:
            return ""
        except Exception as e:
            logger.warning(f"Recognition error: {e}")
            return ""

# ═══════════════════════════════════════════════════════════════════
# TEXT-TO-SPEECH
# ═══════════════════════════════════════════════════════════════════

async def edge_tts_speak(text: str):
    try:
        with tempfile.NamedTemporaryFile(suffix=".mp3", delete=False) as f:
            temp_path = f.name
        
        communicate = edge_tts.Communicate(text, EDGE_TTS_VOICE)
        await communicate.save(temp_path)
        
        # Play using Windows Media Player
        subprocess.run(
            f'powershell -c "(New-Object Media.SoundPlayer \'{temp_path}\').PlaySync()"',
            shell=True, capture_output=True,
            creationflags=subprocess.CREATE_NO_WINDOW
        )
        
        try:
            os.unlink(temp_path)
        except:
            pass
            
    except Exception as e:
        logger.error(f"Edge-TTS error: {e}")
        speak_sapi(text)

def speak_sapi(text: str):
    try:
        safe_text = text.replace('"', "'").replace('\n', ' ')
        cmd = f'Add-Type -AssemblyName System.Speech; (New-Object System.Speech.Synthesis.SpeechSynthesizer).Speak("{safe_text}")'
        subprocess.run(["powershell", "-Command", cmd], capture_output=True,
                      creationflags=subprocess.CREATE_NO_WINDOW)
    except Exception as e:
        logger.error(f"SAPI TTS error: {e}")

def speak(text: str):
    if HAS_EDGE_TTS:
        try:
            asyncio.run(edge_tts_speak(text))
        except:
            speak_sapi(text)
    else:
        speak_sapi(text)

# ═══════════════════════════════════════════════════════════════════
# WINDOWS SYSTEM CONTROL
# ═══════════════════════════════════════════════════════════════════

def open_app(app_name: str) -> bool:
    """Open a Windows application"""
    app_map = {
        "browser": "start msedge",
        "edge": "start msedge", 
        "chrome": "start chrome",
        "firefox": "start firefox",
        "notepad": "notepad",
        "calculator": "calc",
        "explorer": "explorer",
        "files": "explorer",
        "settings": "start ms-settings:",
        "terminal": "wt",
        "cmd": "cmd",
        "powershell": "powershell",
        "code": "code",
        "vscode": "code",
        "spotify": "start spotify:",
        "discord": "start discord:",
        "task manager": "taskmgr",
        "taskmanager": "taskmgr",
        "task": "taskmgr",  # "open task manager"
    }
    
    app_lower = app_name.lower()
    logger.debug(f"Trying to open app: {app_lower}")
    
    for name, cmd in app_map.items():
        if name in app_lower:
            try:
                subprocess.Popen(cmd, shell=True)
                logger.info(f"Opened app: {name} with command: {cmd}")
                return True
            except Exception as e:
                logger.error(f"Failed to open {name}: {e}")
    return False

def take_screenshot():
    try:
        screenshot_dir = Path.home() / "Pictures" / "Screenshots"
        screenshot_dir.mkdir(exist_ok=True)
        filepath = screenshot_dir / f"marceline_{int(time.time())}.png"
        
        ps_script = f'''
        Add-Type -AssemblyName System.Windows.Forms
        $screen = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
        $bitmap = New-Object System.Drawing.Bitmap($screen.Width, $screen.Height)
        $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
        $graphics.CopyFromScreen($screen.Location, [System.Drawing.Point]::Empty, $screen.Size)
        $bitmap.Save("{filepath}")
        '''
        subprocess.run(["powershell", "-Command", ps_script], 
                      capture_output=True, creationflags=subprocess.CREATE_NO_WINDOW)
        logger.info(f"Screenshot saved: {filepath}")
        return True, str(filepath)
    except Exception as e:
        logger.error(f"Screenshot error: {e}")
        return False, str(e)

def set_volume(delta: int):
    try:
        key = 175 if delta > 0 else 174  # Volume up / down
        for _ in range(abs(delta) // 10):
            subprocess.run(
                ["powershell", f"(New-Object -ComObject WScript.Shell).SendKeys([char]{key})"],
                capture_output=True, creationflags=subprocess.CREATE_NO_WINDOW
            )
        logger.info(f"Volume adjusted: {delta}")
    except Exception as e:
        logger.error(f"Volume error: {e}")

# ═══════════════════════════════════════════════════════════════════
# COMMAND PROCESSING
# ═══════════════════════════════════════════════════════════════════

def process_command(text: str, context: Context) -> str:
    """Process user commands - local commands first, then AI"""
    text_lower = text.lower()
    logger.info(f"Processing command: {text}")
    
    # Time
    if any(w in text_lower for w in ["time", "what time", "clock"]):
        return f"It's currently {datetime.now().strftime('%I:%M %p')}"
    
    # Date
    if any(w in text_lower for w in ["date", "what day", "today"]):
        return datetime.now().strftime("Today is %A, %B %d, %Y")
    
    # Open apps - check for "open" or "launch"
    if any(w in text_lower for w in ["open", "launch", "start"]):
        # Extract app name
        for prefix in ["open", "launch", "start"]:
            if prefix in text_lower:
                app_part = text_lower.split(prefix, 1)[-1].strip()
                if open_app(app_part):
                    return f"Opening {app_part}!"
                break
    
    # Screenshot
    if "screenshot" in text_lower:
        success, result = take_screenshot()
        if success:
            return "Screenshot saved!"
        return "Couldn't take screenshot"
    
    # Volume
    if "volume" in text_lower:
        if "up" in text_lower:
            set_volume(20)
            return "Volume up!"
        elif "down" in text_lower:
            set_volume(-20)
            return "Volume down!"
        elif "mute" in text_lower:
            subprocess.run(
                ["powershell", "(New-Object -ComObject WScript.Shell).SendKeys([char]173)"],
                capture_output=True, creationflags=subprocess.CREATE_NO_WINDOW
            )
            return "Toggled mute"
    
    # Web search
    if any(w in text_lower for w in ["search for", "look up", "google", "search"]):
        query = text_lower
        for prefix in ["search for", "look up", "google", "search"]:
            query = query.replace(prefix, "")
        query = query.strip()
        if query:
            url = f"https://www.google.com/search?q={query.replace(' ', '+')}"
            subprocess.Popen(f'start "" "{url}"', shell=True)
            logger.info(f"Web search: {query}")
            return f"Searching for {query}"
    
    # Lock computer
    if any(w in text_lower for w in ["lock computer", "lock screen", "lock pc"]):
        subprocess.run("rundll32.exe user32.dll,LockWorkStation", shell=True)
        return "Locking computer!"
    
    # AI response for everything else
    response = call_gemini(text, context)
    
    context.messages.append({"role": "user", "content": text})
    context.messages.append({"role": "assistant", "content": response})
    
    if len(context.messages) > 20:
        context.messages = context.messages[-20:]
    
    return response

def check_wake_word(text: str) -> tuple:
    text_lower = text.lower()
    
    for wake in WAKE_WORDS:
        if wake in text_lower:
            idx = text_lower.find(wake)
            command = text[idx + len(wake):].strip()
            logger.debug(f"Wake word detected: {wake}, command: {command}")
            return True, command
    
    return False, ""

# ═══════════════════════════════════════════════════════════════════
# MAIN
# ═══════════════════════════════════════════════════════════════════

def show_banner():
    print()
    print(f"{Colors.PURPLE}{'═' * 65}{Colors.NC}")
    print(f"{Colors.PURPLE}  🧛 MARCELINE - Voice AI Assistant{Colors.NC}")
    print(f"{Colors.PURPLE}  Say 'Hey Marceline' or 'Computer' to activate{Colors.NC}")
    print(f"{Colors.PURPLE}{'═' * 65}{Colors.NC}")
    print()
    print(f"{Colors.CYAN}Log file: {LOG_FILE}{Colors.NC}")
    print()
    print(f"{Colors.YELLOW}Press Ctrl+C to exit{Colors.NC}")
    print()

def main():
    show_banner()
    
    logger.info("=" * 50)
    logger.info("Marceline starting up")
    logger.info(f"Python version: {sys.version}")
    logger.info(f"Edge-TTS available: {HAS_EDGE_TTS}")
    logger.info(f"Speech recognition available: {HAS_SPEECH}")
    logger.info("=" * 50)
    
    if not HAS_SPEECH:
        log("Install: pip install SpeechRecognition pyaudio", Colors.RED, "ERROR")
        return
    
    context = Context()
    speech = SpeechEngine()
    active = False
    active_timeout = 0
    
    log("Starting Marceline...", Colors.GREEN)
    speak("Hey! I'm Marceline. Say my name when you need me!")
    
    try:
        while True:
            text = speech.listen(timeout=3.0)
            
            if not text:
                if active and time.time() > active_timeout:
                    active = False
                    log("Standby mode", Colors.BLUE)
                continue
            
            if not active:
                woke, command = check_wake_word(text)
                if woke:
                    active = True
                    active_timeout = time.time() + 30
                    
                    if command:
                        log(f"Command: {command}", Colors.CYAN)
                        response = process_command(command, context)
                        log(f"Response: {response}", Colors.GREEN)
                        speak(response)
                    else:
                        speak("Yes?")
                continue
            
            active_timeout = time.time() + 30
            
            if any(w in text for w in ["goodbye", "bye", "stop", "sleep"]):
                speak("Okay, call me if you need anything!")
                active = False
                continue
            
            response = process_command(text, context)
            log(f"Response: {response}", Colors.GREEN)
            speak(response)
            
    except KeyboardInterrupt:
        logger.info("Marceline shutting down")
        print(f"\n{Colors.YELLOW}Shutting down...{Colors.NC}")
        speak("Goodbye!")

if __name__ == "__main__":
    main()
