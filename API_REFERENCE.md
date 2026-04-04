# FLXTRA Public API Reference

Complete API documentation for all 15 crates in the FLXTRA workspace.

---

## Core Types (flxtra_core)

### Error Handling

```rust
/// Unified error type for all FLXTRA operations
#[derive(Debug)]
pub enum FlxtraError {
    ParseError(String),
    NetworkError(String),
    SecurityError(String),
    EncryptionError(String),
    SandboxError(String),
    InvalidInput(String),
}

impl std::fmt::Display for FlxtraError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // Standard error display
    }
}

pub type Result<T> = std::result::Result<T, FlxtraError>;
```

### Common Traits

```rust
/// Async tool interface for MCP tools
#[async_trait]
pub trait McpTool: Send + Sync {
    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult>;
    fn description(&self) -> &str;
    fn name(&self) -> &str;
}

/// Configuration provider
pub trait Config: Send + Sync {
    fn get_bool(&self, key: &str) -> bool;
    fn get_string(&self, key: &str) -> String;
    fn get_i32(&self, key: &str) -> i32;
}
```

---

## Network & Privacy (flxtra_net)

### DNS over HTTPS

```rust
pub struct DoHResolver {
    provider: DnsProvider,  // Cloudflare, Google, Quad9
}

impl DoHResolver {
    pub async fn resolve(&self, domain: &str) -> Result<Vec<IpAddr>> {
        // Resolves domain using DNS-over-HTTPS
        // No plaintext DNS leaks
    }
    
    pub async fn resolve_with_timeout(
        &self,
        domain: &str,
        timeout_ms: u64,
    ) -> Result<Vec<IpAddr>> {
        // Timeout-safe resolution
    }
}

pub enum DnsProvider {
    Cloudflare,  // 1.1.1.1
    Google,      // 8.8.8.8
    Quad9,       // 9.9.9.9
}
```

### HTTP Client

```rust
pub struct PrivacyHttpClient {
    // Enforces HTTPS-only, DoH lookups, telemetry blocking
}

impl PrivacyHttpClient {
    pub async fn get(&self, url: &str) -> Result<String> {
        // GET request through privacy firewall
    }
    
    pub async fn post(&self, url: &str, body: &str) -> Result<String> {
        // POST request through privacy firewall
    }
    
    pub fn set_user_agent(&mut self, agent: &str) {
        // Set custom user agent (privacy-preserving default)
    }
}
```

### Telemetry Firewall

```rust
pub struct TelemetryFirewall;

impl TelemetryFirewall {
    pub fn is_tracking_domain(domain: &str) -> bool {
        // Returns true if domain is known tracker
    }
    
    pub fn should_block_request(url: &str) -> bool {
        // Returns true if request should be blocked
        // Checks: tracking domains, known ad servers, etc.
    }
    
    pub fn register_custom_block(&mut self, pattern: &str) {
        // Add user-created blocklist entries
    }
}

pub const BLOCKED_DOMAINS: &[&str] = &[
    "google-analytics.com",
    "doubleclick.net",
    "facebook.com",
    // ...50+ more
];
```

---

## Content Filtering (flxtra_filter)

### Ad & Tracker Blocking

```rust
pub struct ContentFilter {
    static_rules: Vec<BlockRule>,
    dynamic_classifier: Option<OnnxModel>,
}

pub struct BlockRule {
    pub pattern: String,
    pub category: BlockCategory,
}

pub enum BlockCategory {
    Advertisement,
    Tracking,
    Malware,
    Analytics,
    SocialWidget,
}

impl ContentFilter {
    pub fn should_block_element(
        &self,
        element: &HtmlElement,
        parent_domain: &str,
    ) -> bool {
        // True if element should be filtered from page
    }
    
    pub fn filter_page(&self, html: &str) -> String {
        // Returns filtered HTML with blocked elements removed
    }
    
    pub async fn scan_for_trackers(&self, page: &str) -> Vec<TrackerInfo> {
        // Identify tracking pixels, analytics scripts, etc.
    }
}

pub struct TrackerInfo {
    pub name: String,
    pub domain: String,
    pub script_hash: String,
}
```

---

## HTML Parsing (flxtra_html)

### DOM Tree

```rust
pub enum NodeType {
    Document,
    Element(String),  // Tag name
    Text(String),
    Comment(String),
}

pub struct DomNode {
    pub node_type: NodeType,
    pub attributes: HashMap<String, String>,
    pub children: Vec<Arc<RwLock<DomNode>>>,
    pub parent: Option<Weak<RwLock<DomNode>>>,
}

impl DomNode {
    pub fn get_attribute(&self, name: &str) -> Option<&str> {
        // Get attribute by name
    }
    
    pub fn query_selector(&self, selector: &str) -> Vec<Arc<RwLock<DomNode>>> {
        // Simple CSS selector support
    }
    
    pub fn to_string(&self) -> String {
        // Serialize DOM back to HTML
    }
}
```

### Parser

```rust
pub struct HtmlParser;

impl HtmlParser {
    pub fn new() -> Self { Self }
    
    pub fn parse(&self, html: &str) -> Arc<RwLock<DomNode>> {
        // Parse HTML string into DOM tree
        // Handles: nested tags, self-closing tags, attributes
    }
    
    pub fn parse_fragment(&self, fragment: &str) -> Vec<Arc<RwLock<DomNode>>> {
        // Parse HTML fragment (no root element required)
    }
}
```

---

## CSS Parsing & Styling (flxtra_css)

### CSS Rules & Specificity

```rust
pub struct CssRule {
    pub selector: String,
    pub properties: HashMap<String, String>,
    pub specificity: (u32, u32, u32),  // (ID, class, element)
}

impl CssRule {
    pub fn get_specificity(&self) -> (u32, u32, u32) {
        self.specificity
    }
    
    pub fn applies_to_element(&self, element: &HtmlElement) -> bool {
        // Returns true if selector matches element
    }
}

pub struct ComputedStyle {
    pub properties: HashMap<String, String>,
}

impl ComputedStyle {
    pub fn get(&self, property: &str) -> Option<&str> {
        // Get computed value for CSS property
    }
    
    pub fn get_with_default(&self, property: &str, default: &str) -> String {
        // Get value or return default
    }
}
```

### Parser

```rust
pub struct CssParser;

impl CssParser {
    pub fn new() -> Self { Self }
    
    pub fn parse(&self, css: &str) -> Vec<CssRule> {
        // Parse CSS string into rules with computed specificity
    }
    
    pub fn parse_inline_style(&self, style: &str) -> HashMap<String, String> {
        // Parse inline style attribute
    }
}
```

---

## Layout Engine (flxtra_layout)

### Box Model

```rust
pub struct LayoutBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub spacing: Spacing,
    pub content: BoxContent,
}

pub struct Spacing {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

pub enum BoxContent {
    Text(String),
    Element(Vec<LayoutBox>),
    Image { src: String, width: f32, height: f32 },
}

impl LayoutBox {
    pub fn content_box(&self) -> (f32, f32, f32, f32) {
        // Returns (x, y, width, height) of content area
        // Excludes padding/margin
    }
    
    pub fn margin_box(&self) -> (f32, f32, f32, f32) {
        // Returns (x, y, width, height) including all spacing
    }
}
```

### Layout Engine

```rust
pub struct LayoutEngine;

impl LayoutEngine {
    pub fn new() -> Self { Self }
    
    pub fn calculate_layout(
        &self,
        dom: &DomNode,
        viewport_width: f32,
        viewport_height: f32,
    ) -> Vec<LayoutBox> {
        // Calculate layout for DOM tree in given viewport
    }
    
    pub fn is_visible(&self, box_: &LayoutBox, viewport: (f32, f32, f32, f32)) -> bool {
        // Check if box is within viewport
    }
}
```

---

## GPU Rendering (flxtra_render)

### Render Commands

```rust
pub enum RenderCommandType {
    DrawRect {
        color: (u8, u8, u8, u8),  // RGBA
        border_radius: f32,
    },
    DrawText {
        font_size: f32,
        color: (u8, u8, u8, u8),
        text: String,
    },
    DrawImage {
        src: String,
        filter: ImageFilter,
    },
    DrawBorder {
        color: (u8, u8, u8, u8),
        width: f32,
        style: BorderStyle,
    },
}

pub enum BorderStyle {
    Solid,
    Dashed,
    Dotted,
}

pub enum ImageFilter {
    None,
    Blur(f32),
    Grayscale(f32),
    Brightness(f32),
}

pub struct RenderCommand {
    pub command_type: RenderCommandType,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
```

### Compositor

```rust
pub struct Compositor {
    commands: Vec<RenderCommand>,
}

impl Compositor {
    pub fn new() -> Self { Self { commands: vec![] } }
    
    pub fn add_command(&mut self, cmd: RenderCommand) {
        self.commands.push(cmd);
    }
    
    pub fn render(&self, width: u32, height: u32) -> Vec<u8> {
        // Render command buffer to RGBA pixel buffer
        // Returns Vec<u8> of length width * height * 4 (RGBA)
    }
    
    pub fn clear(&mut self) {
        self.commands.clear();
    }
}
```

---

## JavaScript Runtime (flxtra_js)

### JS Values

```rust
pub enum JsValue {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Object(HashMap<String, JsValue>),
    Function(Arc<dyn Fn(Vec<JsValue>) -> JsValue>),
}

impl JsValue {
    pub fn as_string(&self) -> Option<&str> {
        if let Self::String(s) = self { Some(s) } else { None }
    }
    
    pub fn as_number(&self) -> Option<f64> {
        if let Self::Number(n) = self { Some(*n) } else { None }
    }
    
    pub fn as_bool(&self) -> Option<bool> {
        if let Self::Boolean(b) = self { Some(*b) } else { None }
    }
    
    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Null | Self::Undefined => false,
            Self::Boolean(b) => *b,
            _ => true,
        }
    }
}
```

### Runtime

```rust
pub struct JsRuntime {
    globals: HashMap<String, JsValue>,
}

impl JsRuntime {
    pub fn new() -> Self {
        // Creates runtime with console, document, window globals
    }
    
    pub async fn execute(&self, code: &str) -> Result<Option<JsValue>> {
        // Execute JavaScript code
        // Returns last expression value
    }
    
    pub fn set_global(&mut self, name: &str, value: JsValue) {
        self.globals.insert(name.to_string(), value);
    }
    
    pub fn get_global(&self, name: &str) -> Option<&JsValue> {
        self.globals.get(name)
    }
}

pub struct Console {
    output: Arc<Mutex<Vec<String>>>,
}

impl Console {
    pub async fn log(&self, args: Vec<JsValue>) {
        // Log arguments to output buffer
    }
    
    pub async fn get_output(&self) -> Vec<String> {
        self.output.lock().await.clone()
    }
}
```

---

## Sandboxing (flxtra_sandbox)

### IPC Protocol

```rust
pub enum IpcMessage {
    Navigate {
        url: String,
    },
    Click {
        selector: String,
    },
    Type {
        selector: String,
        text: String,
    },
    GetDom,
    Response {
        success: bool,
        data: Option<String>,
    },
}

#[derive(Debug, Clone)]
pub struct SandboxTab {
    id: String,
    is_alive: Arc<Mutex<bool>>,
    messages: Arc<Mutex<Vec<IpcMessage>>>,
}

impl SandboxTab {
    pub async fn send_message(&self, msg: IpcMessage) {
        let mut msgs = self.messages.lock().await;
        msgs.push(msg);
    }
    
    pub async fn receive_message(&self) -> Option<IpcMessage> {
        let mut msgs = self.messages.lock().await;
        msgs.pop()
    }
    
    pub async fn is_alive(&self) -> Option<()> {
        let alive = self.is_alive.lock().await;
        if *alive { Some(()) } else { None }
    }
}
```

### Manager

```rust
pub struct SandboxManager {
    tabs: Arc<Mutex<Vec<SandboxTab>>>,
}

impl SandboxManager {
    pub async fn new() -> Self { Self { tabs: Arc::new(Mutex::new(vec![])) } }
    
    pub async fn create_tab(&self, url: &str) -> SandboxTab {
        // Creates new isolated tab process
        // Returns tab handle for IPC communication
    }
    
    pub async fn close_tab(&self, tab_id: &str) -> Result<()> {
        // Terminates sandboxed process
    }
    
    pub async fn list_tabs(&self) -> Vec<SandboxTab> {
        self.tabs.lock().await.clone()
    }
}
```

---

## Agent System (flxtra_agents)

### Intent Classification

```rust
pub enum Intent {
    Summarize,
    Research,
    Automate,
    Scrape,
    Analyze,
}

pub struct AgentTask {
    pub intent: Intent,
    pub input: String,
    pub steps: Vec<String>,
}

impl AgentTask {
    pub fn add_step(&mut self, step: String) {
        self.steps.push(step);
    }
}
```

### Coordinator

```rust
pub struct Coordinator {
    agents: HashMap<Intent, Arc<dyn Agent>>,
}

impl Coordinator {
    pub async fn new() -> Self {
        // Initialize coordinator with 5 agents
    }
    
    pub async fn classify_intent(&self, input: &str) -> Intent {
        // Classify user input to Intent type
    }
    
    pub async fn create_plan(&self, task: &AgentTask) -> Vec<String> {
        // Generate numbered execution steps
    }
    
    pub async fn execute_task(&self, task: &AgentTask) -> Result<String> {
        // Execute task through appropriate agent
    }
    
    pub async fn verify_task(&self, task: &AgentTask, result: &str) -> bool {
        // Verify result satisfies original intent
    }
}
```

### Agent Trait

```rust
#[async_trait]
pub trait Agent: Send + Sync {
    async fn execute(&self, task: &AgentTask) -> Result<String>;
    fn intent_type(&self) -> Intent;
    fn description(&self) -> &str;
}
```

### Specialized Agents

```rust
pub struct SummarizationAgent;

impl SummarizationAgent {
    pub async fn summarize(&self, text: &str, max_words: usize) -> Result<String> {
        // Summarize text using local LLM
    }
}

pub struct ResearchAgent;

impl ResearchAgent {
    pub async fn research(&self, query: &str) -> Result<Vec<SearchResult>> {
        // Research topic, return results
    }
}

pub struct AutomationAgent;

impl AutomationAgent {
    pub async fn automate(&self, actions: Vec<Action>) -> Result<String> {
        // Execute series of actions (form filling, etc.)
    }
}

pub struct ScrapingAgent;

impl ScrapingAgent {
    pub async fn scrape(&self, url: &str, selector: &str) -> Result<Vec<String>> {
        // Extract data using CSS selector
    }
}

pub struct SecurityAgent;

impl SecurityAgent {
    pub async fn analyze_site(&self, url: &str) -> Result<SiteScore> {
        // Analyze site for security/privacy issues
    }
}

pub struct SiteScore {
    pub score: u32,           // 0-100
    pub level: RiskLevel,
    pub trackers_blocked: u32,
}

pub enum RiskLevel {
    Safe,      // 66-100
    Caution,   // 34-65
    Dangerous, // 0-33
}
```

---

## LLM Runtime (flxtra_llm)

### Model Management

```rust
pub struct LlmModel {
    pub name: String,
    pub size_mb: u32,
    pub quantization: Quantization,
    pub path: PathBuf,
}

pub enum Quantization {
    Q4,      // 4-bit (3-5 tokens/sec on CPU)
    Q8,      // 8-bit (1-2 tokens/sec on CPU)
    Fp16,    // 16-bit (very slow on CPU)
}

pub struct LlmRuntime {
    models: HashMap<String, LlmModel>,
}

impl LlmRuntime {
    pub async fn new() -> Self { Self { models: HashMap::new() } }
    
    pub async fn load_model(&mut self, name: &str) -> Result<()> {
        // Load GGUF model into memory
        // Uses mmap for efficiency
    }
    
    pub async fn complete(
        &self,
        prompt: &str,
        max_tokens: usize,
    ) -> Result<String> {
        // Generate text using loaded model (no cloud calls)
    }
    
    pub async fn available_models(&self) -> Vec<&str> {
        // List locally available models
    }
}
```

### Model Manager

```rust
pub struct ModelManager;

impl ModelManager {
    pub async fn download_model(
        &self,
        name: &str,
        quantization: Quantization,
    ) -> Result<PathBuf> {
        // Download model from source (e.g., Hugging Face)
        // Stores in ~/.cache/flxtra/models/
    }
    
    pub async fn list_available(&self) -> Vec<ModelInfo> {
        // List all available models from sources
    }
}

pub struct ModelInfo {
    pub name: String,
    pub source: String,
    pub size_mb: u32,
}
```

---

## Memory Store (flxtra_memory)

### Data Models

```rust
pub struct MemoryItem {
    pub id: String,
    pub content: String,
    pub tags: Vec<String>,
    pub created_at: SystemTime,
    pub source: MemorySource,
}

pub enum MemorySource {
    UserCreated,
    AgentExtracted,
}

pub struct MemoryStore {
    items: Arc<Mutex<HashMap<String, MemoryItem>>>,
}
```

### CRUD Operations

```rust
impl MemoryStore {
    pub async fn new() -> Self {
        // Initialize encrypted local store
    }
    
    pub async fn store(&self, item: &MemoryItem) -> Result<()> {
        // Store encrypted item locally
    }
    
    pub async fn retrieve(&self, id: &str) -> Result<MemoryItem> {
        // Retrieve and decrypt item
    }
    
    pub async fn list_all(&self) -> Result<Vec<MemoryItem>> {
        // List all stored items
    }
    
    pub async fn search(&self, query: &str) -> Result<Vec<MemoryItem>> {
        // Search items by content/tags
    }
    
    pub async fn delete(&self, id: &str) -> Result<()> {
        // Delete item permanently
    }
    
    pub async fn clear_all(&self) -> Result<()> {
        // Delete all items
    }
}
```

---

## UI Components (flxtra_ui)

### Command Bar

```rust
pub struct CommandBar {
    input: String,
    suggestions: Vec<String>,
    execute_callback: Option<Arc<dyn Fn(String) -> BoxFuture<'static, Result<String>>>>,
}

impl CommandBar {
    pub async fn set_input(&mut self, text: &str) {
        self.input = text.to_string();
    }
    
    pub async fn get_suggestions(&self) -> Vec<String> {
        // Return matching suggestions
    }
    
    pub async fn execute(&self) -> Result<String> {
        // Execute current command
    }
}
```

### Trust Badge

```rust
pub struct TrustBadge {
    score: u32,  // 0-100
}

impl TrustBadge {
    pub async fn get_score(&self) -> u32 {
        self.score
    }
    
    pub async fn set_score(&mut self, score: u32) {
        self.score = (score).min(100);
    }
    
    pub fn get_color(&self) -> (u8, u8, u8) {
        match self.score {
            66..=100 => (76, 175, 80),   // Green
            34..=65 => (255, 193, 7),    // Yellow
            0..=33 => (244, 67, 54),     // Red
            _ => (200, 200, 200),        // Gray
        }
    }
}
```

### Agent Strip

```rust
pub struct AgentStrip {
    is_active: bool,
    current_action: String,
    progress: f32,  // 0.0 - 1.0
}

impl AgentStrip {
    pub async fn set_action(&mut self, action: String) {
        self.current_action = action;
        self.is_active = true;
    }
    
    pub async fn set_progress(&mut self, progress: f32) {
        self.progress = (progress).clamp(0.0, 1.0);
    }
    
    pub async fn clear(&mut self) {
        self.is_active = false;
        self.current_action.clear();
        self.progress = 0.0;
    }
}
```

### Memory Panel

```rust
pub struct MemoryEntry {
    pub id: String,
    pub content: String,
    pub tags: Vec<String>,
}

pub struct MemoryPanel {
    items: Arc<Mutex<HashMap<String, MemoryEntry>>>,
    is_visible: Arc<Mutex<bool>>,
}

impl MemoryPanel {
    pub async fn toggle_visibility(&self) {
        let mut visible = self.is_visible.lock().await;
        *visible = !*visible;
    }
    
    pub async fn add_item(&self, item: &MemoryEntry) {
        let mut items = self.items.lock().await;
        items.insert(item.id.clone(), item.clone());
    }
    
    pub async fn delete_item(&self, id: &str) -> Result<()> {
        let mut items = self.items.lock().await;
        items.remove(id);
        Ok(())
    }
    
    pub async fn get_items(&self) -> Vec<MemoryEntry> {
        self.items.lock().await.values().cloned().collect()
    }
    
    pub async fn clear_all(&self) {
        self.items.lock().await.clear();
    }
}
```

### UI Controller

```rust
pub struct UiController {
    pub command_bar: CommandBar,
    pub trust_badge: TrustBadge,
    pub agent_strip: AgentStrip,
    pub memory_panel: MemoryPanel,
}

impl UiController {
    pub async fn render(&self) -> String {
        // Render all UI components
    }
    
    pub async fn handle_input(&mut self, event: InputEvent) {
        // Route input to appropriate component
    }
}
```

---

## Tool Registry (flxtra_mcp)

### MCP Tool Bus

```rust
pub trait McpTool: Send + Sync {
    async fn execute(&self, args: ToolArgs) -> Result<ToolResult>;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn schema(&self) -> serde_json::Value;  // JSON Schema for args
}

pub struct ToolArgs {
    pub params: HashMap<String, serde_json::Value>,
}

pub struct ToolResult {
    pub success: bool,
    pub data: Option<String>,
    pub error: Option<String>,
}
```

### Tool Registry

```rust
pub struct ToolRegistry {
    tools: Arc<Mutex<HashMap<String, Arc<dyn McpTool>>>>,
}

impl ToolRegistry {
    pub async fn register(&self, name: String, tool: Arc<dyn McpTool>) {
        let mut tools = self.tools.lock().await;
        tools.insert(name, tool);
    }
    
    pub async fn call(&self, tool_name: &str, args: ToolArgs) -> Result<ToolResult> {
        let tools = self.tools.lock().await;
        if let Some(tool) = tools.get(tool_name) {
            tool.execute(args).await
        } else {
            Err(FlxtraError::InvalidInput(format!("Tool '{}' not found", tool_name)))
        }
    }
    
    pub async fn list_tools(&self) -> Vec<String> {
        self.tools.lock().await.keys().cloned().collect()
    }
}
```

---

## Async Runtime

All FLXTRA APIs are async by default using tokio:

```rust
#[tokio::main]
async fn main() {
    let coordinator = Coordinator::new().await;
    let result = coordinator.execute_task(&task).await;
}
```

For synchronous code, use `tokio::task::block_in_place`:

```rust
let result = tokio::task::block_in_place(|| {
    tokio::runtime::Handle::current().block_on(async {
        coordinator.execute_task(&task).await
    })
});
```

---

## Feature Flags

Control compilation with Cargo features:

```toml
[features]
default = ["network", "gpu-rendering", "llm"]
network = []            # Internet access
gpu-rendering = []      # wgpu integration
llm = []                # Local LLM support
quantized-models = []   # 4-bit quantization
debug-logs = []         # Verbose logging
```

Build with features:

```bash
cargo build --features "network,llm"
```

---

**API Version**: 0.1.0  
**Last Updated**: 2026-04-05  
**Stability**: Stable (API frozen for 0.1.x releases)
