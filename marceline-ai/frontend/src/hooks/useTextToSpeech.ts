import { useState, useCallback, useEffect, useRef } from 'react';

interface UseTextToSpeechOptions {
    voice?: string;
    rate?: number;
    pitch?: number;
    volume?: number;
}

interface UseTextToSpeechReturn {
    speak: (text: string) => void;
    cancel: () => void;
    isSpeaking: boolean;
    voices: SpeechSynthesisVoice[];
    selectedVoice: SpeechSynthesisVoice | null;
    setVoice: (voice: SpeechSynthesisVoice) => void;
}

export function useTextToSpeech(options: UseTextToSpeechOptions = {}): UseTextToSpeechReturn {
    const { rate = 1, pitch = 1, volume = 1 } = options;

    const [isSpeaking, setIsSpeaking] = useState(false);
    const [voices, setVoices] = useState<SpeechSynthesisVoice[]>([]);
    const [selectedVoice, setSelectedVoice] = useState<SpeechSynthesisVoice | null>(null);

    const utteranceRef = useRef<SpeechSynthesisUtterance | null>(null);

    // Load available voices
    useEffect(() => {
        const loadVoices = () => {
            const availableVoices = speechSynthesis.getVoices();
            setVoices(availableVoices);

            // Try to find a good default voice
            if(!selectedVoice && availableVoices.length > 0) {
                // Prefer female English voices for Marceline
                const preferred = availableVoices.find(v =>
                    v.lang.startsWith('en') &&
                    (v.name.includes('Female') || v.name.includes('Samantha') || v.name.includes('Zira'))
                );
                setSelectedVoice(preferred || availableVoices[0]);
            }
        };

        loadVoices();
        speechSynthesis.addEventListener('voiceschanged', loadVoices);

        return () => {
            speechSynthesis.removeEventListener('voiceschanged', loadVoices);
        };
    }, [selectedVoice]);

    const speak = useCallback((text: string) => {
        // Cancel any ongoing speech
        speechSynthesis.cancel();

        // Clean text for better speech
        const cleanText = text
            .replace(/\*\*/g, '') // Remove bold markdown
            .replace(/\*/g, '')   // Remove italic markdown
            .replace(/_/g, '')    // Remove underscores
            .replace(/`/g, '')    // Remove code ticks
            .replace(/#{1,6}\s/g, '') // Remove headers
            .replace(/\n+/g, '. '); // Convert newlines to pauses

        const utterance = new SpeechSynthesisUtterance(cleanText);
        utteranceRef.current = utterance;

        if(selectedVoice) {
            utterance.voice = selectedVoice;
        }
        utterance.rate = rate;
        utterance.pitch = pitch;
        utterance.volume = volume;

        utterance.onstart = () => setIsSpeaking(true);
        utterance.onend = () => setIsSpeaking(false);
        utterance.onerror = () => setIsSpeaking(false);

        speechSynthesis.speak(utterance);
    }, [selectedVoice, rate, pitch, volume]);

    const cancel = useCallback(() => {
        speechSynthesis.cancel();
        setIsSpeaking(false);
    }, []);

    const setVoice = useCallback((voice: SpeechSynthesisVoice) => {
        setSelectedVoice(voice);
    }, []);

    return {
        speak,
        cancel,
        isSpeaking,
        voices,
        selectedVoice,
        setVoice
    };
}
