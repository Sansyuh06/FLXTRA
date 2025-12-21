import { useState, useEffect, useRef, useCallback } from 'react';
import { useVoiceRecognition } from './hooks/useVoiceRecognition';
import { useTextToSpeech } from './hooks/useTextToSpeech';
import { useChat } from './hooks/useChat';

// Icons as simple SVGs
const MicIcon = () => (
    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
        <path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z" />
        <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
        <line x1="12" y1="19" x2="12" y2="23" />
        <line x1="8" y1="23" x2="16" y2="23" />
    </svg>
);

const SendIcon = () => (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
        <line x1="22" y1="2" x2="11" y2="13" />
        <polygon points="22 2 15 22 11 13 2 9 22 2" />
    </svg>
);

const VolumeIcon = () => (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
        <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5" />
        <path d="M19.07 4.93a10 10 0 0 1 0 14.14" />
        <path d="M15.54 8.46a5 5 0 0 1 0 7.07" />
    </svg>
);

const TrashIcon = () => (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
        <polyline points="3 6 5 6 21 6" />
        <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
    </svg>
);

function App() {
    // Generate a consistent user ID
    const [userId] = useState(() => {
        const stored = localStorage.getItem('marceline_userId');
        if(stored) return stored;
        const newId = `user_${Date.now()}`;
        localStorage.setItem('marceline_userId', newId);
        return newId;
    });

    const [inputText, setInputText] = useState('');
    const [autoSpeak, setAutoSpeak] = useState(true);
    const messagesEndRef = useRef<HTMLDivElement>(null);

    // Chat hook
    const { messages, isLoading, sendMessage, clearMessages } = useChat(userId);

    // Text-to-speech hook
    const { speak, cancel, isSpeaking } = useTextToSpeech();

    // Handle voice command
    const handleVoiceCommand = useCallback(async (command: string) => {
        // Cancel any ongoing speech when new command comes
        cancel();

        const response = await sendMessage(command);
        if(response && autoSpeak) {
            speak(response);
        }
    }, [sendMessage, speak, cancel, autoSpeak]);

    // Voice recognition hook
    const {
        isListening,
        isActive,
        transcript,
        interimTranscript,
        toggleListening,
        error: voiceError
    } = useVoiceRecognition({
        wakeWord: 'hey marceline',
        onWakeWord: () => {
            // Play activation sound or visual feedback
            console.log('Wake word detected!');
        },
        onCommand: handleVoiceCommand
    });

    // Auto-scroll to bottom
    useEffect(() => {
        messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
    }, [messages]);

    // Handle text input submit
    const handleSubmit = async (e?: React.FormEvent) => {
        e?.preventDefault();
        if(!inputText.trim() || isLoading) return;

        const text = inputText;
        setInputText('');

        cancel(); // Cancel any ongoing speech
        const response = await sendMessage(text);
        if(response && autoSpeak) {
            speak(response);
        }
    };

    // Handle Enter key
    const handleKeyDown = (e: React.KeyboardEvent) => {
        if(e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault();
            handleSubmit();
        }
    };

    // Format timestamp
    const formatTime = (date: Date) => {
        return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    };

    // Get status text
    const getStatus = () => {
        if(isSpeaking) return { text: 'Speaking...', class: 'speaking' };
        if(isActive) return { text: 'Listening for command...', class: 'listening' };
        if(isListening) return { text: 'Waiting for "Hey Marceline"', class: 'connected' };
        return { text: 'Click mic to start', class: 'connected' };
    };

    const status = getStatus();

    return (
        <div className="app">
            {/* Header */}
            <header className="header">
                <div className="header-title">
                    <h1>🧛‍♀️ Marceline</h1>
                </div>
                <div className="header-controls">
                    <div className="status-indicator">
                        <span className={`status-dot ${status.class}`} />
                        <span>{status.text}</span>
                    </div>
                </div>
            </header>

            {/* Messages */}
            <div className="messages-container">
                {messages.length === 0 ? (
                    <div className="welcome">
                        <h2>Hey, I'm Marceline! 🦇</h2>
                        <p>Your voice-activated AI assistant.</p>
                        <p>Click the microphone and say:</p>
                        <div className="wake-word">"Hey Marceline, ..."</div>
                        <p style={{ marginTop: '16px', fontSize: '0.875rem' }}>
                            Or just type your message below!
                        </p>
                    </div>
                ) : (
                    messages.map((msg) => (
                        <div key={msg.id} className={`message ${msg.role}`}>
                            <div className="message-content">{msg.content}</div>
                            <div className="message-time">{formatTime(msg.timestamp)}</div>
                        </div>
                    ))
                )}

                {/* Loading indicator */}
                {isLoading && (
                    <div className="message assistant loading">
                        <div className="loading-dots">
                            <span className="loading-dot" />
                            <span className="loading-dot" />
                            <span className="loading-dot" />
                        </div>
                    </div>
                )}

                <div ref={messagesEndRef} />
            </div>

            {/* Input Area */}
            <div className="input-area">
                {/* Voice transcript display */}
                {(transcript || interimTranscript) && (
                    <div className="transcript">
                        {transcript || interimTranscript}
                    </div>
                )}

                {/* Voice error display */}
                {voiceError && (
                    <div className="transcript" style={{ borderColor: '#ef4444', color: '#ef4444' }}>
                        {voiceError}
                    </div>
                )}

                <div className="input-container">
                    {/* Mic button */}
                    <button
                        className={`btn btn-voice ${isListening ? 'listening' : ''}`}
                        onClick={toggleListening}
                        title={isListening ? 'Stop listening' : 'Start listening'}
                    >
                        {isListening ? (
                            <div className="voice-visualizer">
                                <div className="voice-bar" />
                                <div className="voice-bar" />
                                <div className="voice-bar" />
                                <div className="voice-bar" />
                                <div className="voice-bar" />
                            </div>
                        ) : (
                            <MicIcon />
                        )}
                    </button>

                    {/* Text input */}
                    <input
                        type="text"
                        className="text-input"
                        placeholder="Type a message..."
                        value={inputText}
                        onChange={(e) => setInputText(e.target.value)}
                        onKeyDown={handleKeyDown}
                        disabled={isLoading}
                    />

                    {/* Send button */}
                    <button
                        className="btn btn-primary"
                        onClick={() => handleSubmit()}
                        disabled={!inputText.trim() || isLoading}
                        title="Send message"
                    >
                        <SendIcon />
                    </button>

                    {/* TTS toggle */}
                    <button
                        className={`btn btn-icon ${autoSpeak ? '' : 'muted'}`}
                        onClick={() => setAutoSpeak(!autoSpeak)}
                        title={autoSpeak ? 'Mute responses' : 'Enable voice responses'}
                        style={{ opacity: autoSpeak ? 1 : 0.5 }}
                    >
                        <VolumeIcon />
                    </button>

                    {/* Clear chat */}
                    <button
                        className="btn btn-icon"
                        onClick={clearMessages}
                        title="Clear conversation"
                    >
                        <TrashIcon />
                    </button>
                </div>
            </div>
        </div>
    );
}

export default App;
