import { Router, Request, Response } from 'express';
import { marcelineAI } from '../api/claude-handler';
import {
    saveMessage,
    createConversation,
    getConversation,
    getConversationMessages,
    clearConversationMessages,
    ensureUser,
    addNote,
    getNotes,
    searchNotes,
    deleteNote,
    addReminder,
    getUpcomingReminders,
    completeReminder,
    getPendingReminders,
    addCalendarEvent,
    getUpcomingEvents,
    deleteCalendarEvent
} from '../database/services';
import { config } from '../config/environment';

const router = Router();

// ═══════════════════════════════════════════
// CHAT ROUTES
// ═══════════════════════════════════════════

// Send a message
router.post('/', async (req: Request, res: Response) => {
    try {
        const { message, userId, conversationId } = req.body;

        if(!message || typeof message !== 'string') {
            return res.status(400).json({ error: 'Message is required' });
        }

        if(!config.isValid()) {
            return res.status(503).json({
                error: 'Gemini API not configured. Please add GEMINI_API_KEY to your environment.',
                demo: true
            });
        }

        const uid = userId || 'default-user';
        ensureUser(uid);

        // Get or create conversation
        let convId = conversationId;
        if(!convId) {
            const conv = createConversation(uid, message.slice(0, 50));
            convId = conv.id;
        }

        // Save user message
        saveMessage({
            conversation_id: convId,
            role: 'user',
            content: message
        });

        // Get AI response
        const response = await marcelineAI.chat(uid, message);

        // Save assistant message
        saveMessage({
            conversation_id: convId,
            role: 'assistant',
            content: response
        });

        res.json({
            response,
            conversationId: convId
        });

    } catch(error) {
        console.error('Chat error:', error);
        res.status(500).json({
            error: 'Failed to process message',
            details: error instanceof Error ? error.message : 'Unknown error'
        });
    }
});

// Clear conversation history
router.post('/clear', async (req: Request, res: Response) => {
    const { userId, conversationId } = req.body;
    const uid = userId || 'default-user';

    marcelineAI.clearHistory(uid);

    if(conversationId) {
        clearConversationMessages(conversationId);
    }

    res.json({ success: true });
});

// Get conversation history
router.get('/history/:conversationId', async (req: Request, res: Response) => {
    const { conversationId } = req.params;
    const messages = getConversationMessages(conversationId);
    res.json({ messages });
});

// ═══════════════════════════════════════════
// NOTES ROUTES
// ═══════════════════════════════════════════

router.post('/notes', async (req: Request, res: Response) => {
    try {
        const { title, content, tags } = req.body;
        if(!title || !content) {
            return res.status(400).json({ error: 'Title and content required' });
        }
        const note = addNote(title, content, tags);
        res.json({ success: true, note });
    } catch(error) {
        res.status(500).json({ error: 'Failed to create note' });
    }
});

router.get('/notes', async (req: Request, res: Response) => {
    const limit = parseInt(req.query.limit as string) || 20;
    const notes = getNotes(limit);
    res.json({ notes });
});

router.get('/notes/search', async (req: Request, res: Response) => {
    const query = req.query.q as string || '';
    const notes = searchNotes(query);
    res.json({ notes });
});

router.delete('/notes/:id', async (req: Request, res: Response) => {
    const id = parseInt(req.params.id);
    deleteNote(id);
    res.json({ success: true });
});

// ═══════════════════════════════════════════
// REMINDERS ROUTES
// ═══════════════════════════════════════════

router.post('/reminders', async (req: Request, res: Response) => {
    try {
        const { message, remind_at } = req.body;
        if(!message || !remind_at) {
            return res.status(400).json({ error: 'Message and remind_at required' });
        }
        const reminder = addReminder(message, remind_at);
        res.json({ success: true, reminder });
    } catch(error) {
        res.status(500).json({ error: 'Failed to create reminder' });
    }
});

router.get('/reminders', async (req: Request, res: Response) => {
    const reminders = getUpcomingReminders();
    res.json({ reminders });
});

router.get('/reminders/pending', async (req: Request, res: Response) => {
    const reminders = getPendingReminders();
    res.json({ reminders });
});

router.post('/reminders/:id/complete', async (req: Request, res: Response) => {
    const id = parseInt(req.params.id);
    completeReminder(id);
    res.json({ success: true });
});

// ═══════════════════════════════════════════
// CALENDAR ROUTES
// ═══════════════════════════════════════════

router.post('/calendar', async (req: Request, res: Response) => {
    try {
        const { title, description, start_time, end_time, location } = req.body;
        if(!title || !start_time) {
            return res.status(400).json({ error: 'Title and start_time required' });
        }
        const event = addCalendarEvent({ title, description, start_time, end_time, location });
        res.json({ success: true, event });
    } catch(error) {
        res.status(500).json({ error: 'Failed to create event' });
    }
});

router.get('/calendar', async (req: Request, res: Response) => {
    const days = parseInt(req.query.days as string) || 7;
    const events = getUpcomingEvents(days);
    res.json({ events });
});

router.delete('/calendar/:id', async (req: Request, res: Response) => {
    const id = parseInt(req.params.id);
    deleteCalendarEvent(id);
    res.json({ success: true });
});

export default router;
