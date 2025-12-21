import { useState, useEffect, useRef, useCallback } from 'react';

interface SpeechRecognitionResult {
    transcript: string;
    isFinal: boolean;
}

interface UseVoiceRecognitionOptions {
    wakeWord?: string;
    continuous?: boolean;
    language?: string;
    onWakeWord?: () => void;
    onCommand?: (command: string) => void;
}

interface UseVoiceRecognitionReturn {
    isListening: boolean;
    isActive: boolean;
    transcript: string;
    interimTranscript: string;
    startListening: () => void;
    stopListening: () => void;
    toggleListening: () => void;
    error: string | null;
}

// Type declarations for Web Speech API
interface SpeechRecognitionEvent extends Event {
    results: SpeechRecognitionResultList;
    resultIndex: number;
}

interface SpeechRecognition extends EventTarget {
    continuous: boolean;
    interimResults: boolean;
    lang: string;
    start: () => void;
    stop: () => void;
    abort: () => void;
    onresult: ((event: SpeechRecognitionEvent) => void) | null;
    onerror: ((event: Event & { error: string }) => void) | null;
    onend: (() => void) | null;
    onstart: (() => void) | null;
}

declare global {
    interface Window {
        SpeechRecognition: new () => SpeechRecognition;
        webkitSpeechRecognition: new () => SpeechRecognition;
    }
}

export function useVoiceRecognition(options: UseVoiceRecognitionOptions = {}): UseVoiceRecognitionReturn {
    const {
        wakeWord = 'hey marceline',
        continuous = true,
        language = 'en-US',
        onWakeWord,
        onCommand
    } = options;

    const [isListening, setIsListening] = useState(false);
    const [isActive, setIsActive] = useState(false);
    const [transcript, setTranscript] = useState('');
    const [interimTranscript, setInterimTranscript] = useState('');
    const [error, setError] = useState<string | null>(null);

    const recognitionRef = useRef<SpeechRecognition | null>(null);
    const activeTimeoutRef = useRef<number | null>(null);

    const initRecognition = useCallback(() => {
        const SpeechRecognition = window.SpeechRecognition || window.webkitSpeechRecognition;

        if(!SpeechRecognition) {
            setError('Speech recognition not supported in this browser');
            return null;
        }

        const recognition = new SpeechRecognition();
        recognition.continuous = continuous;
        recognition.interimResults = true;
        recognition.lang = language;

        recognition.onstart = () => {
            setIsListening(true);
            setError(null);
        };

        recognition.onresult = (event: SpeechRecognitionEvent) => {
            let finalTranscript = '';
            let interim = '';

            for(let i = event.resultIndex; i < event.results.length; i++) {
                const result = event.results[i];
                const text = result[0].transcript;

                if(result.isFinal) {
                    finalTranscript += text;
                } else {
                    interim += text;
                }
            }

            setInterimTranscript(interim);

            if(finalTranscript) {
                const lowerTranscript = finalTranscript.toLowerCase();

                // Check for wake word
                if(!isActive && lowerTranscript.includes(wakeWord.toLowerCase())) {
                    setIsActive(true);
                    onWakeWord?.();

                    // Extract command after wake word
                    const wakeIndex = lowerTranscript.indexOf(wakeWord.toLowerCase());
                    const afterWake = finalTranscript.substring(wakeIndex + wakeWord.length).trim();

                    if(afterWake) {
                        setTranscript(afterWake);
                        onCommand?.(afterWake);
                        deactivateAfterTimeout();
                    }
                } else if(isActive) {
                    setTranscript(finalTranscript);
                    onCommand?.(finalTranscript);
                    deactivateAfterTimeout();
                }
            }
        };

        recognition.onerror = (event) => {
            if(event.error === 'no-speech') {
                // Ignore no-speech errors, just restart
                return;
            }
            setError(`Speech recognition error: ${event.error}`);
            setIsListening(false);
        };

        recognition.onend = () => {
            // Auto-restart if we should still be listening
            if(isListening && recognitionRef.current) {
                try {
                    recognitionRef.current.start();
                } catch(e) {
                    // Already started, ignore
                }
            } else {
                setIsListening(false);
            }
        };

        return recognition;
    }, [continuous, language, wakeWord, isActive, onWakeWord, onCommand, isListening]);

    const deactivateAfterTimeout = useCallback(() => {
        if(activeTimeoutRef.current) {
            clearTimeout(activeTimeoutRef.current);
        }
        activeTimeoutRef.current = window.setTimeout(() => {
            setIsActive(false);
            setTranscript('');
            setInterimTranscript('');
        }, 30000); // Deactivate after 30 seconds of inactivity
    }, []);

    const startListening = useCallback(() => {
        if(!recognitionRef.current) {
            recognitionRef.current = initRecognition();
        }

        if(recognitionRef.current) {
            try {
                recognitionRef.current.start();
            } catch(e) {
                // Already started
            }
        }
    }, [initRecognition]);

    const stopListening = useCallback(() => {
        if(recognitionRef.current) {
            recognitionRef.current.stop();
            setIsListening(false);
            setIsActive(false);
            setTranscript('');
            setInterimTranscript('');
        }
    }, []);

    const toggleListening = useCallback(() => {
        if(isListening) {
            stopListening();
        } else {
            startListening();
        }
    }, [isListening, startListening, stopListening]);

    // Cleanup on unmount
    useEffect(() => {
        return () => {
            if(recognitionRef.current) {
                recognitionRef.current.stop();
            }
            if(activeTimeoutRef.current) {
                clearTimeout(activeTimeoutRef.current);
            }
        };
    }, []);

    return {
        isListening,
        isActive,
        transcript,
        interimTranscript,
        startListening,
        stopListening,
        toggleListening,
        error
    };
}
