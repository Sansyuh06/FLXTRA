import dotenv from 'dotenv';
import path from 'path';

// Load environment variables
dotenv.config();

export const config = {
    // Server
    port: parseInt(process.env.PORT || '3001', 10),

    // API Keys
    geminiApiKey: process.env.GEMINI_API_KEY || '',

    // Database
    databasePath: process.env.DATABASE_PATH || './data/marceline.db',

    // Validation
    isValid(): boolean {
        if(!this.geminiApiKey || this.geminiApiKey === 'your_gemini_api_key_here') {
            console.warn('⚠️  Warning: GEMINI_API_KEY not set. AI features will be disabled.');
            return false;
        }
        return true;
    }
};
