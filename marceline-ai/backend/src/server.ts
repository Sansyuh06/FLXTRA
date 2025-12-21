import express from 'express';
import cors from 'cors';
import { config } from './config/environment';
import { initDatabase, closeDatabase } from './database/connection';
import { mcpManager } from './mcp/mcp-manager';
import chatRoutes from './routes/chat.routes';
import fs from 'fs';
import path from 'path';

const app = express();

// Middleware
app.use(cors());
app.use(express.json());

// Health check
app.get('/health', (req, res) => {
    res.json({
        status: 'ok',
        timestamp: new Date().toISOString(),
        configured: config.isValid(),
        mcpServers: mcpManager.getConnectedServers(),
        mcpTools: mcpManager.getToolCount()
    });
});

// API Routes
app.use('/api/chat', chatRoutes);

// Error handling middleware
app.use((err: Error, req: express.Request, res: express.Response, next: express.NextFunction) => {
    console.error('Unhandled error:', err);
    res.status(500).json({ error: 'Internal server error' });
});

// Startup function
async function start() {
    console.log('\n🧛 Starting Marceline AI Assistant Backend...\n');

    // Initialize database
    try {
        initDatabase();
    } catch(error) {
        console.error('Failed to initialize database:', error);
        process.exit(1);
    }

    // Load MCP configuration if available
    const mcpConfigPath = path.join(__dirname, '../../mcp-config.json');
    if(fs.existsSync(mcpConfigPath)) {
        try {
            const mcpConfig = JSON.parse(fs.readFileSync(mcpConfigPath, 'utf-8'));
            await mcpManager.initializeServers(mcpConfig);
            console.log(`📦 MCP tools available: ${mcpManager.getToolCount()}\n`);
        } catch(error) {
            console.warn('Failed to load MCP configuration:', error);
        }
    } else {
        console.log('ℹ️  No MCP configuration found (mcp-config.json)\n');
    }

    // Validate configuration
    if(!config.isValid()) {
        console.log('ℹ️  Running in demo mode - add GEMINI_API_KEY to enable AI\n');
    } else {
        console.log('✅ Gemini API key configured\n');
    }

    // Start server
    app.listen(config.port, () => {
        console.log(`🚀 Server running on http://localhost:${config.port}`);
        console.log(`📋 Health check: http://localhost:${config.port}/health\n`);
    });
}

// Graceful shutdown
async function shutdown() {
    console.log('\n🛑 Shutting down...');
    await mcpManager.cleanup();
    closeDatabase();
    process.exit(0);
}

process.on('SIGINT', shutdown);
process.on('SIGTERM', shutdown);

// Start the server
start().catch((error) => {
    console.error('Failed to start server:', error);
    process.exit(1);
});
