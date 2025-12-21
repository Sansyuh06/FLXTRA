import { useState, useCallback } from 'react';
import axios from 'axios';

interface Message {
    id: string;
    role: 'user' | 'assistant';
    content: string;
    timestamp: Date;
}

interface UseChatReturn {
    messages: Message[];
    isLoading: boolean;
    error: string | null;
    sendMessage: (content: string) => Promise<string | null>;
    clearMessages: () => void;
    conversationId: string | null;
}

const API_BASE = '/api/chat';

export function useChat(userId: string): UseChatReturn {
    const [messages, setMessages] = useState<Message[]>([]);
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [conversationId, setConversationId] = useState<string | null>(null);

    const sendMessage = useCallback(async (content: string): Promise<string | null> => {
        if(!content.trim()) return null;

        const userMessage: Message = {
            id: `user-${Date.now()}`,
            role: 'user',
            content: content.trim(),
            timestamp: new Date()
        };

        setMessages(prev => [...prev, userMessage]);
        setIsLoading(true);
        setError(null);

        try {
            const response = await axios.post(API_BASE, {
                message: content,
                userId,
                conversationId
            });

            const { response: aiResponse, conversationId: convId } = response.data;

            if(convId && !conversationId) {
                setConversationId(convId);
            }

            const assistantMessage: Message = {
                id: `assistant-${Date.now()}`,
                role: 'assistant',
                content: aiResponse,
                timestamp: new Date()
            };

            setMessages(prev => [...prev, assistantMessage]);
            return aiResponse;

        } catch(err) {
            const errorMessage = axios.isAxiosError(err)
                ? err.response?.data?.error || err.message
                : 'Failed to send message';

            setError(errorMessage);

            // Add error message to chat
            const errorAssistant: Message = {
                id: `error-${Date.now()}`,
                role: 'assistant',
                content: `Sorry, I encountered an error: ${errorMessage}`,
                timestamp: new Date()
            };
            setMessages(prev => [...prev, errorAssistant]);

            return null;
        } finally {
            setIsLoading(false);
        }
    }, [userId, conversationId]);

    const clearMessages = useCallback(async () => {
        try {
            await axios.post(`${API_BASE}/clear`, { userId, conversationId });
        } catch(err) {
            console.error('Failed to clear server-side history:', err);
        }
        setMessages([]);
        setConversationId(null);
        setError(null);
    }, [userId, conversationId]);

    return {
        messages,
        isLoading,
        error,
        sendMessage,
        clearMessages,
        conversationId
    };
}
