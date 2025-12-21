import { getDatabase } from './connection';
import { v4 as uuidv4 } from 'uuid';

export interface Message {
  id?: number;
  conversation_id: string;
  role: 'user' | 'assistant';
  content: string;
  timestamp?: string;
  metadata?: Record<string, unknown>;
}

export interface Conversation {
  id: string;
  user_id: string;
  title?: string;
  created_at?: string;
  updated_at?: string;
}

export interface Note {
  id?: number;
  title: string;
  content: string;
  tags?: string;
  created_at?: string;
}

export interface Reminder {
  id?: number;
  message: string;
  remind_at: string;
  completed?: boolean;
  created_at?: string;
}

export interface CalendarEvent {
  id?: number;
  title: string;
  description?: string;
  start_time: string;
  end_time?: string;
  location?: string;
}

// Conversation operations
export function createConversation(userId: string, title?: string): Conversation {
  const db = getDatabase();
  const id = uuidv4();

  db.prepare(`
    INSERT INTO conversations (id, user_id, title) VALUES (?, ?, ?)
  `).run(id, userId, title || 'New Conversation');

  return { id, user_id: userId, title };
}

export function getConversation(conversationId: string): Conversation | undefined {
  const db = getDatabase();
  return db.prepare(`
    SELECT * FROM conversations WHERE id = ?
  `).get(conversationId) as Conversation | undefined;
}

export function getUserConversations(userId: string): Conversation[] {
  const db = getDatabase();
  return db.prepare(`
    SELECT * FROM conversations 
    WHERE user_id = ? 
    ORDER BY updated_at DESC
  `).all(userId) as Conversation[];
}

// Message operations
export function saveMessage(message: Message): Message {
  const db = getDatabase();
  const metadata = JSON.stringify(message.metadata || {});

  const result = db.prepare(`
    INSERT INTO messages (conversation_id, role, content, metadata)
    VALUES (?, ?, ?, ?)
  `).run(message.conversation_id, message.role, message.content, metadata);

  // Update conversation timestamp
  db.prepare(`
    UPDATE conversations SET updated_at = CURRENT_TIMESTAMP WHERE id = ?
  `).run(message.conversation_id);

  return { ...message, id: result.lastInsertRowid as number };
}

export function getConversationMessages(conversationId: string): Message[] {
  const db = getDatabase();
  const rows = db.prepare(`
    SELECT * FROM messages 
    WHERE conversation_id = ? 
    ORDER BY timestamp ASC
  `).all(conversationId) as Array<Message & { metadata: string }>;

  return rows.map(row => ({
    ...row,
    metadata: JSON.parse(row.metadata || '{}')
  }));
}

export function clearConversationMessages(conversationId: string): void {
  const db = getDatabase();
  db.prepare(`DELETE FROM messages WHERE conversation_id = ?`).run(conversationId);
}

// User operations
export function ensureUser(userId: string, name?: string): void {
  const db = getDatabase();
  db.prepare(`
    INSERT OR IGNORE INTO users (id, name) VALUES (?, ?)
  `).run(userId, name || 'User');
}

// Search
export function searchMessages(userId: string, query: string): Message[] {
  const db = getDatabase();
  return db.prepare(`
    SELECT m.* FROM messages m
    JOIN conversations c ON m.conversation_id = c.id
    WHERE c.user_id = ? AND m.content LIKE ?
    ORDER BY m.timestamp DESC
    LIMIT 50
  `).all(userId, `%${query}%`) as Message[];
}

// ═══════════════════════════════════════════════════════
// NOTES OPERATIONS
// ═══════════════════════════════════════════════════════

export function addNote(title: string, content: string, tags?: string): Note {
  const db = getDatabase();
  const result = db.prepare(`
    INSERT INTO notes (title, content, tags) VALUES (?, ?, ?)
  `).run(title, content, tags || '');

  return { id: result.lastInsertRowid as number, title, content, tags };
}

export function getNotes(limit: number = 20): Note[] {
  const db = getDatabase();
  return db.prepare(`
    SELECT * FROM notes ORDER BY created_at DESC LIMIT ?
  `).all(limit) as Note[];
}

export function searchNotes(query: string): Note[] {
  const db = getDatabase();
  return db.prepare(`
    SELECT * FROM notes 
    WHERE title LIKE ? OR content LIKE ?
    ORDER BY created_at DESC
  `).all(`%${query}%`, `%${query}%`) as Note[];
}

export function deleteNote(noteId: number): void {
  const db = getDatabase();
  db.prepare(`DELETE FROM notes WHERE id = ?`).run(noteId);
}

// ═══════════════════════════════════════════════════════
// REMINDERS OPERATIONS
// ═══════════════════════════════════════════════════════

export function addReminder(message: string, remindAt: string): Reminder {
  const db = getDatabase();
  const result = db.prepare(`
    INSERT INTO reminders (message, remind_at) VALUES (?, ?)
  `).run(message, remindAt);

  return { id: result.lastInsertRowid as number, message, remind_at: remindAt };
}

export function getPendingReminders(): Reminder[] {
  const db = getDatabase();
  const now = new Date().toISOString();
  return db.prepare(`
    SELECT * FROM reminders 
    WHERE remind_at <= ? AND completed = 0
    ORDER BY remind_at ASC
  `).all(now) as Reminder[];
}

export function getUpcomingReminders(): Reminder[] {
  const db = getDatabase();
  return db.prepare(`
    SELECT * FROM reminders 
    WHERE completed = 0
    ORDER BY remind_at ASC
    LIMIT 20
  `).all() as Reminder[];
}

export function completeReminder(reminderId: number): void {
  const db = getDatabase();
  db.prepare(`UPDATE reminders SET completed = 1 WHERE id = ?`).run(reminderId);
}

// ═══════════════════════════════════════════════════════
// CALENDAR OPERATIONS
// ═══════════════════════════════════════════════════════

export function addCalendarEvent(event: Omit<CalendarEvent, 'id'>): CalendarEvent {
  const db = getDatabase();
  const result = db.prepare(`
    INSERT INTO calendar_events (title, description, start_time, end_time, location)
    VALUES (?, ?, ?, ?, ?)
  `).run(event.title, event.description || '', event.start_time, event.end_time || null, event.location || '');

  return { id: result.lastInsertRowid as number, ...event };
}

export function getUpcomingEvents(days: number = 7): CalendarEvent[] {
  const db = getDatabase();
  const now = new Date().toISOString();
  const future = new Date(Date.now() + days * 24 * 60 * 60 * 1000).toISOString();

  return db.prepare(`
    SELECT * FROM calendar_events 
    WHERE start_time >= ? AND start_time <= ?
    ORDER BY start_time ASC
  `).all(now, future) as CalendarEvent[];
}

export function deleteCalendarEvent(eventId: number): void {
  const db = getDatabase();
  db.prepare(`DELETE FROM calendar_events WHERE id = ?`).run(eventId);
}

// ═══════════════════════════════════════════════════════
// USER PREFERENCES
// ═══════════════════════════════════════════════════════

export function setPreference(userId: string, key: string, value: string): void {
  const db = getDatabase();
  db.prepare(`
    INSERT OR REPLACE INTO user_preferences (user_id, key, value, updated_at)
    VALUES (?, ?, ?, CURRENT_TIMESTAMP)
  `).run(userId, key, value);
}

export function getPreference(userId: string, key: string): string | undefined {
  const db = getDatabase();
  const row = db.prepare(`
    SELECT value FROM user_preferences WHERE user_id = ? AND key = ?
  `).get(userId, key) as { value: string } | undefined;

  return row?.value;
}

export function getAllPreferences(userId: string): Record<string, string> {
  const db = getDatabase();
  const rows = db.prepare(`
    SELECT key, value FROM user_preferences WHERE user_id = ?
  `).all(userId) as Array<{ key: string; value: string }>;

  const prefs: Record<string, string> = {};
  for(const row of rows) {
    prefs[row.key] = row.value;
  }
  return prefs;
}
