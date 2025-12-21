import { GoogleGenerativeAI, GenerativeModel, Content, FunctionCallingMode } from '@google/generative-ai';
import { config } from '../config/environment';
import { mcpManager } from '../mcp/mcp-manager';

const SYSTEM_PROMPT = `You are Marceline, a friendly and powerful voice-activated AI assistant.

## Personality
- You're warm, approachable, and slightly witty - like a helpful friend
- You speak naturally using contractions (I'm, you're, it's) 
- You're proactive but not pushy
- You keep responses concise for voice output (2-3 sentences)

## Your Capabilities (via MCP Tools)
When tools are available, you can:
- **File Operations**: Read, write, search, and manage files
- **Web Search**: Search the internet for information (Brave Search)
- **Memory**: Remember things across conversations  
- **Fetch**: Read content from URLs and websites
- **Notes & Reminders**: Help users take notes and set reminders
- **Calendar**: Manage calendar events

## How to Use Tools
1. When the user asks to read a file, use the filesystem tools
2. When they want to know something current, use web search
3. When they want to remember something, use the memory tool
4. Always confirm what actions you took

## Voice Interaction Guidelines
- Keep responses to 2-3 sentences unless asked for detail
- Avoid bullet points and markdown that doesn't work in speech
- If a topic is complex, give a brief summary first

## Example Interactions
- "What files are in my project?" → Use list_directory tool
- "Search for Python tutorials" → Use brave_search tool  
- "Remember that my meeting is at 3pm" → Use memory tool
- "Read the README file" → Use read_file tool

## Guidelines
- Be honest about your limitations
- Ask for clarification when requests are ambiguous
- Protect user privacy - never share sensitive info
- If a tool fails, explain what went wrong and suggest alternatives`;

interface ConversationHistory {
    messages: Content[];
}

export class MarcelineAI {
    private genAI: GoogleGenerativeAI;
    private conversations: Map<string, ConversationHistory> = new Map();

    constructor() {
        this.genAI = new GoogleGenerativeAI(config.geminiApiKey);
    }

    private getOrCreateConversation(userId: string): ConversationHistory {
        if(!this.conversations.has(userId)) {
            this.conversations.set(userId, { messages: [] });
        }
        return this.conversations.get(userId)!;
    }

    private getModel(): GenerativeModel {
        const tools = mcpManager.getGeminiFunctions();

        if(tools.length > 0) {
            return this.genAI.getGenerativeModel({
                model: 'gemini-2.0-flash-exp',
                systemInstruction: SYSTEM_PROMPT,
                tools: [{ functionDeclarations: tools as any }],
            });
        }

        return this.genAI.getGenerativeModel({
            model: 'gemini-2.0-flash-exp',
            systemInstruction: SYSTEM_PROMPT
        });
    }

    async chat(userId: string, userMessage: string): Promise<string> {
        const conversation = this.getOrCreateConversation(userId);
        const model = this.getModel();

        // Add user message to history
        conversation.messages.push({
            role: 'user',
            parts: [{ text: userMessage }]
        });

        try {
            // Create chat session with history
            const chat = model.startChat({
                history: conversation.messages.slice(0, -1),
            });

            // Send message and get response
            let result = await chat.sendMessage(userMessage);
            let response = result.response;

            // Handle function calling loop
            while(response.candidates?.[0]?.content?.parts?.some(p => 'functionCall' in p)) {
                const functionCalls = response.candidates[0].content.parts.filter(p => 'functionCall' in p);

                const functionResponses = [];

                for(const part of functionCalls) {
                    if('functionCall' in part && part.functionCall) {
                        const fc = part.functionCall;
                        const fcName = fc.name || 'unknown';
                        console.log(`🔧 Calling tool: ${fcName}`);

                        try {
                            const originalName = mcpManager.getOriginalToolName(fcName);
                            const toolResult = await mcpManager.callTool(originalName, (fc.args || {}) as Record<string, unknown>);

                            functionResponses.push({
                                functionResponse: {
                                    name: fcName,
                                    response: { result: toolResult }
                                }
                            });
                            console.log(`   ✅ Tool completed`);
                        } catch(error) {
                            console.error(`   ❌ Tool error:`, error);
                            functionResponses.push({
                                functionResponse: {
                                    name: fcName,
                                    response: { error: String(error) }
                                }
                            });
                        }
                    }
                }

                // Send function responses back to get final answer
                result = await chat.sendMessage(functionResponses);
                response = result.response;
            }

            const textResponse = response.text();

            // Add assistant response to history
            conversation.messages.push({
                role: 'model',
                parts: [{ text: textResponse }]
            });

            // Trim conversation history if too long
            if(conversation.messages.length > 40) {
                conversation.messages = conversation.messages.slice(-30);
            }

            return textResponse;
        } catch(error) {
            console.error('Gemini API error:', error);
            throw error;
        }
    }

    clearHistory(userId: string): void {
        this.conversations.delete(userId);
    }

    getHistory(userId: string): Content[] {
        return this.getOrCreateConversation(userId).messages;
    }
}

// Singleton instance
export const marcelineAI = new MarcelineAI();
