//! Natural Language Processing for Command Parsing
//!
//! Features:
//! - Intent recognition from natural language
//! - Entity extraction (URLs, selectors, text)
//! - Workflow generation from commands
//! - Context-aware parsing

use std::collections::HashMap;
use regex::Regex;
use serde::{Deserialize, Serialize};
use flxtra_core::{Result, FlxtraError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedCommand {
    pub intent: Intent,
    pub entities: HashMap<String, String>,
    pub confidence: f32,
    pub original_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Intent {
    // Navigation
    Navigate,
    GoBack,
    GoForward,
    Refresh,

    // Interaction
    Click,
    Type,
    Scroll,
    Wait,

    // Data operations
    Search,
    FillForm,
    ExtractData,
    Login,

    // Multi-step tasks
    BookFlight,
    MakePurchase,
    ResearchTopic,
    CompareProducts,

    // Advanced
    AutomateTask,
    MonitorPage,
    TakeScreenshot,

    // Unknown
    Unknown,
}

pub struct NLPParser {
    intent_patterns: HashMap<Intent, Vec<Regex>>,
    entity_patterns: HashMap<String, Regex>,
}

impl NLPParser {
    pub fn new() -> Self {
        let mut intent_patterns = HashMap::new();
        let mut entity_patterns = HashMap::new();

        // Navigation patterns
        intent_patterns.insert(Intent::Navigate, vec![
            Regex::new(r"(?i)(go to|navigate to|visit|open)\s+(.+)").unwrap(),
            Regex::new(r"(?i)(browse|surf)\s+(.+)").unwrap(),
        ]);

        intent_patterns.insert(Intent::GoBack, vec![
            Regex::new(r"(?i)(go back|back|previous)").unwrap(),
        ]);

        intent_patterns.insert(Intent::GoForward, vec![
            Regex::new(r"(?i)(go forward|forward|next)").unwrap(),
        ]);

        intent_patterns.insert(Intent::Refresh, vec![
            Regex::new(r"(?i)(refresh|reload)").unwrap(),
        ]);

        // Interaction patterns
        intent_patterns.insert(Intent::Click, vec![
            Regex::new(r"(?i)(click|tap)\s+(on\s+)?(.+)").unwrap(),
            Regex::new(r"(?i)(select|choose)\s+(.+)").unwrap(),
        ]);

        intent_patterns.insert(Intent::Type, vec![
            Regex::new(r"(?i)(type|enter|input)\s+(.+)\s+(in|into)\s+(.+)").unwrap(),
            Regex::new(r"(?i)(fill|write)\s+(.+)").unwrap(),
        ]);

        intent_patterns.insert(Intent::Scroll, vec![
            Regex::new(r"(?i)(scroll|move)\s+(up|down|left|right)").unwrap(),
        ]);

        intent_patterns.insert(Intent::Wait, vec![
            Regex::new(r"(?i)(wait|pause)\s+(\d+)\s*(seconds?|minutes?)").unwrap(),
        ]);

        // Data operation patterns
        intent_patterns.insert(Intent::Search, vec![
            Regex::new(r"(?i)(search|find|look for)\s+(.+)").unwrap(),
            Regex::new(r"(?i)(google|bing)\s+(.+)").unwrap(),
        ]);

        intent_patterns.insert(Intent::FillForm, vec![
            Regex::new(r"(?i)(fill|complete)\s+(the\s+)?form").unwrap(),
            Regex::new(r"(?i)(enter|input)\s+(.+)\s+information").unwrap(),
        ]);

        intent_patterns.insert(Intent::Login, vec![
            Regex::new(r"(?i)(log in|login|sign in|signin)\s+(to|into)\s+(.+)").unwrap(),
            Regex::new(r"(?i)(authenticate|auth)\s+with\s+(.+)").unwrap(),
        ]);

        // Multi-step task patterns
        intent_patterns.insert(Intent::BookFlight, vec![
            Regex::new(r"(?i)(book|reserve)\s+(a\s+)?flight").unwrap(),
            Regex::new(r"(?i)(fly|travel)\s+(from|to)\s+(.+)").unwrap(),
        ]);

        intent_patterns.insert(Intent::MakePurchase, vec![
            Regex::new(r"(?i)(buy|purchase|order)\s+(.+)").unwrap(),
            Regex::new(r"(?i)(add\s+to\s+cart|checkout)").unwrap(),
        ]);

        intent_patterns.insert(Intent::ResearchTopic, vec![
            Regex::new(r"(?i)(research|investigate|study)\s+(.+)").unwrap(),
            Regex::new(r"(?i)(find\s+information\s+about|learn\s+about)\s+(.+)").unwrap(),
        ]);

        intent_patterns.insert(Intent::CompareProducts, vec![
            Regex::new(r"(?i)(compare|comparison)\s+(.+)").unwrap(),
            Regex::new(r"(?i)(vs|versus|vs\.)\s+(.+)").unwrap(),
        ]);

        // Advanced patterns
        intent_patterns.insert(Intent::AutomateTask, vec![
            Regex::new(r"(?i)(automate|repeat|do)\s+(.+)").unwrap(),
            Regex::new(r"(?i)(create\s+workflow|make\s+task)\s+(.+)").unwrap(),
        ]);

        intent_patterns.insert(Intent::TakeScreenshot, vec![
            Regex::new(r"(?i)(screenshot|capture|photo)\s+(of\s+)?(.+)").unwrap(),
        ]);

        // Entity patterns
        entity_patterns.insert("url".to_string(),
            Regex::new(r"https?://[^\s]+").unwrap());
        entity_patterns.insert("email".to_string(),
            Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap());
        entity_patterns.insert("phone".to_string(),
            Regex::new(r"\+?\d{1,4}?[-.\s]?\(?\d{1,3}?\)?[-.\s]?\d{1,4}[-.\s]?\d{1,4}[-.\s]?\d{1,9}").unwrap());
        entity_patterns.insert("date".to_string(),
            Regex::new(r"\d{1,2}[/-]\d{1,2}[/-]\d{2,4}").unwrap());
        entity_patterns.insert("time".to_string(),
            Regex::new(r"\d{1,2}:\d{2}(\s?[ap]m)?").unwrap());
        entity_patterns.insert("price".to_string(),
            Regex::new(r"\$?\d+(\.\d{2})?").unwrap());

        Self {
            intent_patterns,
            entity_patterns,
        }
    }

    /// Parse natural language command
    pub fn parse_command(&self, text: &str) -> Result<ParsedCommand> {
        let (intent, confidence) = self.recognize_intent(text);
        let entities = self.extract_entities(text);

        Ok(ParsedCommand {
            intent,
            entities,
            confidence,
            original_text: text.to_string(),
        })
    }

    /// Recognize intent from text
    fn recognize_intent(&self, text: &str) -> (Intent, f32) {
        let mut best_intent = Intent::Unknown;
        let mut best_confidence = 0.0;

        for (intent, patterns) in &self.intent_patterns {
            for pattern in patterns {
                if let Some(captures) = pattern.captures(text) {
                    let confidence = self.calculate_confidence(text, &captures);
                    if confidence > best_confidence {
                        best_confidence = confidence;
                        best_intent = intent.clone();
                    }
                }
            }
        }

        (best_intent, best_confidence)
    }

    /// Calculate confidence score for pattern match
    fn calculate_confidence(&self, text: &str, captures: &regex::Captures) -> f32 {
        let matched_text = captures.get(0).unwrap().as_str();
        let match_ratio = matched_text.len() as f32 / text.len() as f32;

        // Boost confidence for exact matches and longer matches
        let exact_boost = if matched_text.to_lowercase() == text.to_lowercase() { 0.3 } else { 0.0 };
        let length_boost = if matched_text.len() > 10 { 0.2 } else { 0.0 };

        (match_ratio + exact_boost + length_boost).min(1.0)
    }

    /// Extract entities from text
    fn extract_entities(&self, text: &str) -> HashMap<String, String> {
        let mut entities = HashMap::new();

        for (entity_type, pattern) in &self.entity_patterns {
            if let Some(capture) = pattern.find(text) {
                entities.insert(entity_type.clone(), capture.as_str().to_string());
            }
        }

        // Extract additional context-specific entities
        self.extract_context_entities(text, &mut entities);

        entities
    }

    /// Extract context-specific entities
    fn extract_context_entities(&self, text: &str, entities: &mut HashMap<String, String>) {
        // Extract search terms
        if text.to_lowercase().contains("search") || text.to_lowercase().contains("find") {
            if let Some(search_match) = Regex::new(r"(?i)(search|find|look for)\s+(.+)").unwrap().captures(text) {
                if let Some(term) = search_match.get(2) {
                    entities.insert("search_term".to_string(), term.as_str().trim().to_string());
                }
            }
        }

        // Extract form fields
        if text.to_lowercase().contains("fill") || text.to_lowercase().contains("enter") {
            let field_patterns = vec![
                ("username", r"(?i)(user\s*name|login|email)"),
                ("password", r"(?i)(password|pass)"),
                ("first_name", r"(?i)(first\s*name|fname)"),
                ("last_name", r"(?i)(last\s*name|lname)"),
                ("address", r"(?i)(address|street)"),
                ("city", r"(?i)(city|town)"),
                ("zip", r"(?i)(zip|postal)"),
            ];

            for (field_name, pattern) in field_patterns {
                if Regex::new(pattern).unwrap().is_match(text) {
                    entities.insert(format!("field_{}", field_name), field_name.to_string());
                }
            }
        }

        // Extract quantities and amounts
        if let Some(number_match) = Regex::new(r"\b(\d+)\b").unwrap().find(text) {
            entities.insert("quantity".to_string(), number_match.as_str().to_string());
        }
    }

    /// Generate workflow from parsed command
    pub fn generate_workflow(&self, parsed: &ParsedCommand) -> Result<super::workflows::Workflow> {
        use super::workflows::{Workflow, WorkflowStep};

        let steps = match parsed.intent {
            Intent::Navigate => {
                if let Some(url) = parsed.entities.get("url") {
                    vec![WorkflowStep::NavigateTo(url.clone())]
                } else {
                    vec![WorkflowStep::NavigateTo("https://www.google.com".to_string())]
                }
            }

            Intent::Search => {
                if let Some(term) = parsed.entities.get("search_term") {
                    vec![
                        WorkflowStep::NavigateTo("https://www.google.com".to_string()),
                        WorkflowStep::LocateElement {
                            description: "search box".to_string(),
                            locator: super::vision::ElementLocator::Placeholder("search".to_string()),
                        },
                        WorkflowStep::TypeIntoElement {
                            selector: "search box".to_string(),
                            text: term.clone(),
                        },
                        WorkflowStep::PressKey("enter".to_string()),
                    ]
                } else {
                    vec![WorkflowStep::NavigateTo("https://www.google.com".to_string())]
                }
            }

            Intent::Login => {
                vec![
                    WorkflowStep::LocateElement {
                        description: "username field".to_string(),
                        locator: super::vision::ElementLocator::Placeholder("username".to_string()),
                    },
                    WorkflowStep::TypeIntoElement {
                        selector: "username".to_string(),
                        text: parsed.entities.get("username").unwrap_or(&"".to_string()).clone(),
                    },
                    WorkflowStep::LocateElement {
                        description: "password field".to_string(),
                        locator: super::vision::ElementLocator::Placeholder("password".to_string()),
                    },
                    WorkflowStep::TypeIntoElement {
                        selector: "password".to_string(),
                        text: parsed.entities.get("password").unwrap_or(&"".to_string()).clone(),
                    },
                    WorkflowStep::ClickElement("login button".to_string()),
                ]
            }

            Intent::FillForm => {
                vec![
                    WorkflowStep::AnalyzeScreen,
                    WorkflowStep::LocateFormFields,
                    WorkflowStep::FillFieldsFromMemory,
                    WorkflowStep::VerifyCompletion,
                ]
            }

            Intent::TakeScreenshot => {
                vec![WorkflowStep::TakeScreenshot]
            }

            Intent::BookFlight => {
                vec![
                    WorkflowStep::NavigateTo("https://www.google.com/flights".to_string()),
                    WorkflowStep::LocateElement {
                        description: "from field".to_string(),
                        locator: super::vision::ElementLocator::Placeholder("from".to_string()),
                    },
                    WorkflowStep::TypeIntoElement {
                        selector: "from".to_string(),
                        text: parsed.entities.get("from").unwrap_or(&"".to_string()).clone(),
                    },
                    WorkflowStep::LocateElement {
                        description: "to field".to_string(),
                        locator: super::vision::ElementLocator::Placeholder("to".to_string()),
                    },
                    WorkflowStep::TypeIntoElement {
                        selector: "to".to_string(),
                        text: parsed.entities.get("to").unwrap_or(&"".to_string()).clone(),
                    },
                    WorkflowStep::ClickElement("search flights".to_string()),
                ]
            }

            _ => vec![
                WorkflowStep::AnalyzeScreen,
                WorkflowStep::ExtractText,
            ],
        };

        Ok(Workflow {
            id: uuid::Uuid::new_v4().to_string(),
            name: parsed.original_text.clone(),
            steps,
            timeout: std::time::Duration::from_secs(60),
            metadata: HashMap::new(),
        })
    }

    /// Get supported intents
    pub fn get_supported_intents(&self) -> Vec<Intent> {
        self.intent_patterns.keys().cloned().collect()
    }

    /// Add custom intent pattern
    pub fn add_intent_pattern(&mut self, intent: Intent, pattern: &str) -> Result<()> {
        let regex = Regex::new(pattern).map_err(|e| FlxtraError::Other(anyhow::anyhow!("Invalid regex pattern: {}", e)))?;
        self.intent_patterns.entry(intent).or_insert_with(Vec::new).push(regex);
        Ok(())
    }
}