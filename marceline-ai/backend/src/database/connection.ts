import Database from 'better-sqlite3';
import path from 'path';
import fs from 'fs';
import { config } from '../config/environment';

let db: Database.Database | null = null;

export function initDatabase(): Database.Database {
  // Ensure data directory exists
  const dbDir = path.dirname(config.databasePath);
  if(!fs.existsSync(dbDir)) {
    fs.mkdirSync(dbDir, { recursive: true });
  }

  db = new Database(config.databasePath);

  // Enable WAL mode for better performance
  db.pragma('journal_mode = WAL');

  // Create tables
  db.exec(`
    -- Users table
    CREATE TABLE IF NOT EXISTS users (
      id TEXT PRIMARY KEY,
      name TEXT,
      created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
      preferences TEXT DEFAULT '{}'
    );

    -- Conversations table
    CREATE TABLE IF NOT EXISTS conversations (
      id TEXT PRIMARY KEY,
      user_id TEXT NOT NULL,
      title TEXT,
      created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
      updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
      FOREIGN KEY (user_id) REFERENCES users(id)
    );

    -- Messages table
    CREATE TABLE IF NOT EXISTS messages (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      conversation_id TEXT NOT NULL,
      role TEXT NOT NULL CHECK (role IN ('user', 'assistant')),
      content TEXT NOT NULL,
      timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
      metadata TEXT DEFAULT '{}',
      FOREIGN KEY (conversation_id) REFERENCES conversations(id)
    );

    -- Notes table (like in MARCELINE_FULL.sh)
    CREATE TABLE IF NOT EXISTS notes (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      title TEXT NOT NULL,
      content TEXT NOT NULL,
      tags TEXT DEFAULT '',
      created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
      updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
    );

    -- Reminders table
    CREATE TABLE IF NOT EXISTS reminders (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      message TEXT NOT NULL,
      remind_at DATETIME NOT NULL,
      completed INTEGER DEFAULT 0,
      created_at DATETIME DEFAULT CURRENT_TIMESTAMP
    );

    -- Calendar events table
    CREATE TABLE IF NOT EXISTS calendar_events (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      title TEXT NOT NULL,
      description TEXT DEFAULT '',
      start_time DATETIME NOT NULL,
      end_time DATETIME,
      location TEXT DEFAULT '',
      created_at DATETIME DEFAULT CURRENT_TIMESTAMP
    );

    -- User preferences table
    CREATE TABLE IF NOT EXISTS user_preferences (
      user_id TEXT NOT NULL,
      key TEXT NOT NULL,
      value TEXT NOT NULL,
      updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
      PRIMARY KEY (user_id, key)
    );

    -- Indexes
    CREATE INDEX IF NOT EXISTS idx_messages_conversation 
      ON messages(conversation_id);
    
    CREATE INDEX IF NOT EXISTS idx_messages_timestamp 
      ON messages(timestamp);
    
    CREATE INDEX IF NOT EXISTS idx_conversations_user 
      ON conversations(user_id);

    CREATE INDEX IF NOT EXISTS idx_reminders_remind_at 
      ON reminders(remind_at);

    CREATE INDEX IF NOT EXISTS idx_calendar_start 
      ON calendar_events(start_time);
  `);

  console.log('✅ Database initialized (with notes, reminders, calendar)');
  return db;
}

export function getDatabase(): Database.Database {
  if(!db) {
    throw new Error('Database not initialized. Call initDatabase() first.');
  }
  return db;
}

export function closeDatabase(): void {
  if(db) {
    db.close();
    db = null;
    console.log('Database connection closed');
  }
}
