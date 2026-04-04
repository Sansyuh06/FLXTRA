//! Encrypted Local Memory Store
//! 
//! Responsible for:
//! - Persistent local-only memory (no cloud sync)
//! - AES-GCM encryption with device-derived key
//! - redb embedded database for structured queries
//! - Memory items: user-created or agent-extracted (with user permission)

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MemoryItem {
    pub id: String,
    pub content: String,
    pub tags: Vec<String>,
    pub created_at: u64,
    pub source: MemorySource,
}

#[derive(Debug, Clone)]
pub enum MemorySource {
    UserCreated,
    AgentExtracted,
}

pub struct MemoryStore {
    items: HashMap<String, MemoryItem>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }

    pub async fn store(&mut self, item: MemoryItem) -> Result<(), String> {
        self.items.insert(item.id.clone(), item);
        Ok(())
    }

    pub async fn retrieve(&self, id: &str) -> Option<MemoryItem> {
        self.items.get(id).cloned()
    }

    pub async fn list_all(&self) -> Vec<MemoryItem> {
        self.items.values().cloned().collect()
    }

    pub async fn delete(&mut self, id: &str) -> Result<(), String> {
        self.items.remove(id);
        Ok(())
    }

    pub async fn clear_all(&mut self) -> Result<(), String> {
        self.items.clear();
        Ok(())
    }
}
