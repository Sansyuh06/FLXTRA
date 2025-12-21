import { Client } from '@modelcontextprotocol/sdk/client/index.js';
import { StdioClientTransport } from '@modelcontextprotocol/sdk/client/stdio.js';

interface MCPServerConfig {
    command: string;
    args?: string[];
    env?: Record<string, string>;
}

interface MCPConfig {
    mcpServers: Record<string, MCPServerConfig>;
}

interface MCPToolInfo {
    serverName: string;
    toolName: string;
    description: string;
    inputSchema: Record<string, unknown>;
}

export class MCPManager {
    private clients: Map<string, Client> = new Map();
    private transports: Map<string, StdioClientTransport> = new Map();
    private toolRegistry: Map<string, MCPToolInfo> = new Map();

    async initializeServers(config: MCPConfig): Promise<void> {
        for(const [name, serverConfig] of Object.entries(config.mcpServers)) {
            try {
                await this.connectServer(name, serverConfig);
                console.log(`✅ MCP Server connected: ${name}`);
            } catch(error) {
                console.error(`❌ Failed to connect MCP server ${name}:`, error);
            }
        }
    }

    private async connectServer(name: string, config: MCPServerConfig): Promise<void> {
        const transport = new StdioClientTransport({
            command: config.command,
            args: config.args || [],
            env: { ...process.env, ...config.env } as Record<string, string>
        });

        const client = new Client({
            name: 'marceline-ai',
            version: '1.0.0'
        }, {
            capabilities: {}
        });

        await client.connect(transport);

        this.clients.set(name, client);
        this.transports.set(name, transport);

        // Register tools from this server
        try {
            const { tools } = await client.listTools();
            for(const tool of tools) {
                const fullName = `${name}__${tool.name}`;
                this.toolRegistry.set(fullName, {
                    serverName: name,
                    toolName: tool.name,
                    description: tool.description || `Tool from ${name}`,
                    inputSchema: tool.inputSchema as Record<string, unknown>
                });
            }
            console.log(`   Registered ${tools.length} tools from ${name}`);
        } catch(error) {
            console.error(`   Failed to list tools from ${name}:`, error);
        }
    }

    async callTool(fullToolName: string, args: Record<string, unknown>): Promise<unknown> {
        const toolInfo = this.toolRegistry.get(fullToolName);
        if(!toolInfo) {
            throw new Error(`Tool not found: ${fullToolName}`);
        }

        const client = this.clients.get(toolInfo.serverName);
        if(!client) {
            throw new Error(`MCP server not connected: ${toolInfo.serverName}`);
        }

        const result = await client.callTool({
            name: toolInfo.toolName,
            arguments: args
        });

        return result;
    }

    // Convert MCP tools to Gemini function declarations
    getGeminiFunctions(): Array<{ name: string; description: string; parameters: Record<string, unknown> }> {
        const functions: Array<{ name: string; description: string; parameters: Record<string, unknown> }> = [];

        for(const [fullName, toolInfo] of this.toolRegistry) {
            // Convert JSON Schema to Gemini format
            const schema = toolInfo.inputSchema;

            functions.push({
                name: fullName.replace(/-/g, '_'), // Gemini doesn't like hyphens in function names
                description: toolInfo.description,
                parameters: {
                    type: 'object',
                    properties: (schema as any).properties || {},
                    required: (schema as any).required || []
                }
            });
        }

        return functions;
    }

    // Get original tool name from Gemini-safe name
    getOriginalToolName(geminiName: string): string {
        // Gemini function names have underscores, but MCP uses hyphens
        // We need to find the matching tool
        for(const [fullName] of this.toolRegistry) {
            if(fullName.replace(/-/g, '_') === geminiName) {
                return fullName;
            }
        }
        return geminiName;
    }

    getConnectedServers(): string[] {
        return Array.from(this.clients.keys());
    }

    getToolCount(): number {
        return this.toolRegistry.size;
    }

    async cleanup(): Promise<void> {
        for(const [name, client] of this.clients) {
            try {
                await client.close();
                console.log(`Closed MCP server: ${name}`);
            } catch(error) {
                console.error(`Error closing ${name}:`, error);
            }
        }
        this.clients.clear();
        this.transports.clear();
        this.toolRegistry.clear();
    }
}

// Singleton instance
export const mcpManager = new MCPManager();
