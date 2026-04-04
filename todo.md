# FLXTRA Build TODO

## Phase 0: Repo Hygiene (IN PROGRESS)

### 0.1 Verify & Clean .gitignore
- [ ] Check that target/, dist/, *.gguf, *.onnx, node_modules, .env are excluded
- [ ] Run `git rm --cached -r target/ dist/` if they're committed
- [ ] Verify `git status` shows clean

### 0.2 Architecture Setup - Add Missing Crate Stubs
- [ ] Add flxtra_html to Cargo.toml workspace.members
- [ ] Add flxtra_css to Cargo.toml workspace.members
- [ ] Add flxtra_layout to Cargo.toml workspace.members
- [ ] Add flxtra_render to Cargo.toml workspace.members
- [ ] Add flxtra_js to Cargo.toml workspace.members
- [ ] Add flxtra_sandbox to Cargo.toml workspace.members
- [ ] Add flxtra_agents to Cargo.toml workspace.members
- [ ] Add flxtra_llm to Cargo.toml workspace.members
- [ ] Add flxtra_memory to Cargo.toml workspace.members
- [ ] Add flxtra_ui to Cargo.toml workspace.members (or confirm plan for this)

### 0.3 Create Crate Stubs
- [ ] Create dir + Cargo.toml for each missing crate
- [ ] Each crate: src/lib.rs with //! doc comment explaining scope
- [ ] Each crate: Add to workspace.dependencies if needed

### 0.4 Verification
- [ ] `cargo build --release` exits 0 (may have warnings)
- [ ] `cargo test` compiles (may fail, that's OK)
- [ ] All crates listed in `cargo metadata --format-version 1 | jq '.packages | length'`

### 0.5 Component Boundary Documentation
- [ ] flxtra_core: Update lib.rs with scope
- [ ] flxtra_net: Update lib.rs with scope
- [ ] flxtra_filter: Update lib.rs with scope
- [ ] flxtra_mcp: Update lib.rs with scope
- [ ] flxtra_browser: Update lib.rs with scope
- [ ] Each new crate: lib.rs explains responsibility + API boundary

---

## Phase 1: Network + Privacy Foundation

### 1.1 flxtra_net: DoH Verification
- [ ] Verify hickory-dns DNS-over-HTTPS feature is enabled
- [ ] Implement DnsResolver struct with DoH as default
- [ ] Create integration test: resolve example.com over DoH

### 1.2 flxtra_net: Telemetry Firewall
- [ ] Define telemetry patterns (fingerprinting APIs, beacons, etc.)
- [ ] Implement TelemetryFilter that blocks outgoing requests matching patterns
- [ ] Create test: request to analytics.google.com is blocked

### 1.3 flxtra_filter: Static Blocking
- [ ] Verify adblock-rust is in dependencies
- [ ] Load filter rules (uBlock Origin format)
- [ ] Implement StaticBlocker trait

### 1.4 flxtra_filter: Integration Test
- [ ] Create list of 50+ known tracker domains
- [ ] Test each is blocked by StaticBlocker
- [ ] Create test: visit page with trackers, verify they're blocked

### 1.5 Phase 1 Verification
- [ ] `cargo test -p flxtra_net` exits 0
- [ ] `cargo test -p flxtra_filter` exits 0

---

## Phase 2: HTML + CSS + Layout + Rendering

### 2.1 flxtra_html: HTML Parser
- [ ] Add html5ever to dependencies
- [ ] Implement HtmlParser using html5ever
- [ ] Create DOM AST types in flxtra_core
- [ ] Test: Parse simple HTML document

### 2.2 flxtra_css: CSS Parser
- [ ] Add selectors crate to dependencies
- [ ] Implement CssParser + StyleResolver
- [ ] Test: Parse simple CSS stylesheet

### 2.3 flxtra_layout: Box Model
- [ ] Define BoxModel types (block, inline, inline-block)
- [ ] Implement layout engine for simple pages
- [ ] Test: Calculate layout for simple HTML

### 2.4 flxtra_render: wgpu Integration
- [ ] Add wgpu to dependencies
- [ ] Implement Compositor using wgpu
- [ ] Test: Render simple colored box

### 2.5 End-to-End Rendering
- [ ] Wire HtmlParser → CssParser → LayoutEngine → Compositor
- [ ] Test: Render Wikipedia main page
- [ ] Compare screenshot with reference image (pixel tolerance)

### 2.6 Phase 2 Verification
- [ ] Screenshot test passes within tolerance

---

## Phase 3: JavaScript Runtime + Sandbox

### 3.1 flxtra_js: deno_core Integration
- [ ] Add deno_core + v8 to dependencies
- [ ] Implement JsRuntime wrapper
- [ ] Test: Execute simple JS snippet

### 3.2 Web APIs Exposure
- [ ] Implement fetch API bridge
- [ ] Implement setTimeout bridge
- [ ] Implement DOM mutation API (limited, no raw access)
- [ ] Test: fetch() works in JS, returns correct data

### 3.3 DOM Bridge
- [ ] Design IPC protocol for DOM → JS communication
- [ ] Implement bridge that maps JS DOM calls to Rust DOM tree
- [ ] Test: JS can read DOM but not modify Rust internals

### 3.4 flxtra_sandbox: OS Isolation
- [ ] Implement Linux: seccomp-bpf + namespaces
- [ ] Implement Windows: AppContainer
- [ ] Implement macOS: App Sandbox
- [ ] Test: Spawn tab process, crash it, verify main browser stable

### 3.5 IPC Protocol
- [ ] Define typed message format
- [ ] Implement message bus between sandbox + main
- [ ] Test: Messages flow correctly both directions

### 3.6 Phase 3 Verification
- [ ] `cargo test -p flxtra_js` exits 0
- [ ] `cargo test -p flxtra_sandbox` exits 0
- [ ] Tab crash recovery test passes

---

## Phase 4: MCP Tool Bus + Agent Core

### 4.1 flxtra_mcp: Tool Registry
- [ ] Define Tool trait (async interface)
- [ ] Implement ToolRegistry
- [ ] Test: Register tool, call tool, receive result

### 4.2 Core Tools: Read Operations
- [ ] Implement read_page tool (returns semantic DOM structure)
- [ ] Implement get_page_text tool (full page text)
- [ ] Implement web_search tool
- [ ] Test each tool independently

### 4.3 Core Tools: Write Operations
- [ ] Implement click tool
- [ ] Implement type_text tool
- [ ] Implement scroll tool
- [ ] Implement navigate tool
- [ ] Test each with mock page

### 4.4 Core Tools: File & Memory
- [ ] Implement write_file tool
- [ ] Implement read_file tool
- [ ] Implement memory_read/write/delete tools
- [ ] Test file permissions work

### 4.5 flxtra_agents: Coordinator Creation
- [ ] Define Agent trait
- [ ] Create Coordinator struct (routes intents to agents)
- [ ] Create event stream (observations flow back)
- [ ] Test: Intent → Coordinator → Agent → Result

### 4.6 Summarization Agent (First Agent)
- [ ] Implement intent classification
- [ ] Implement planning step (numbered steps)
- [ ] Implement execution loop (call tools one at a time)
- [ ] Implement verification step
- [ ] Use mock LLM for now

### 4.7 E2E Test
- [ ] User types "summarize this page"
- [ ] Coordinator routes to Summarization Agent
- [ ] Agent calls read_page + get_page_text
- [ ] Agent calls mock LLM
- [ ] Result displays in UI

### 4.8 Phase 4 Verification
- [ ] Summarization Agent E2E test passes
- [ ] All tools callable + return correct types

---

## Phase 5: Local LLM Runtime

### 5.1 flxtra_llm: LLM Runtime Integration
- [ ] Choose: candle vs llama-cpp-rs (prefer candle for pure Rust)
- [ ] Add chosen crate to dependencies
- [ ] Implement LlmRuntime wrapper

### 5.2 Model Manager
- [ ] Implement model downloader (downloads GGUF models)
- [ ] Store models in ~/.flxtra/models/ or similar
- [ ] Test: Download Phi-3 Mini model

### 5.3 Integration with Agent
- [ ] Replace mock LLM with real LlmRuntime
- [ ] Test: Summarization Agent uses real model
- [ ] time the inference: <10s on test hardware for Phi-3 Mini

### 5.4 Phase 5 Verification
- [ ] Phi-3 Mini loads
- [ ] Summarization of 500-word article completes in <10s
- [ ] Summary is coherent

---

## Phase 6: Full Agent Fleet + Memory

### 6.1 flxtra_memory: Local Encrypted Store
- [ ] Add redb to dependencies
- [ ] Implement MemoryStore with AES-GCM encryption
- [ ] Device key derivation (local only)
- [ ] Test: Write + read + delete memory item

### 6.2 Research Agent
- [ ] Define agent scope (gather information from multiple sources)
- [ ] Implement plan → execute (use web_search tool)
- [ ] Test: Research Agent finds info about a company

### 6.3 Automation Agent
- [ ] Define agent scope (fill forms, automate interactions)
- [ ] Implement plan → execute (use click, type_text, navigate tools)
- [ ] Test: Fill out form with saved profile data

### 6.4 Scraping Agent
- [ ] Define agent scope (extract structured data from pages)
- [ ] Implement smart page structure detection
- [ ] Test: Extract product prices from e-commerce site

### 6.5 Security Agent
- [ ] Define agent scope (compute risk scores, detect threats)
- [ ] Implement risk scoring from: trackers, scripts, permissions, phishing patterns
- [ ] Test: Score known safe/unsafe sites correctly

### 6.6 Memory Panel UI
- [ ] Show all memory items as cards (tagged, timestamped)
- [ ] View / Edit / Delete each item
- [ ] Clear all option
- [ ] Source tags ("you told me" vs "extracted")

### 6.7 Template System
- [ ] Record multi-step agent workflows as templates
- [ ] Save templates as TOML files (local only)
- [ ] Load + execute templates on demand
- [ ] Test: Record job search flow, replay it

### 6.8 Phase 6 Verification
- [ ] All agents complete their intended tasks
- [ ] Memory items persist + encrypt correctly
- [ ] Templates save/load/replay correctly

---

## Phase 7: UI Polish + Task Packs + Templates

### 7.1 Command Bar
- [ ] Implement minimal command bar (center screen, persistent)
- [ ] URL input
- [ ] Search query input
- [ ] Natural language command input with local classification (no cloud)
- [ ] Autocomplete suggestions (local, from history + predefined)

### 7.2 Trust Score Badge
- [ ] Real-time site analysis (trackers blocked, scripts, permissions, phishing)
- [ ] Display as color (green/yellow/red) + number (0-100)
- [ ] Expandable breakdown with full details
- [ ] Test: Known safe site shows green, known phishing site shows red

### 7.3 Agent Status Strip
- [ ] Show when agent is active
- [ ] Display current action (reading page, extracting data, calling LLM, etc.)
- [ ] Disappear when idle
- [ ] Real-time updates

### 7.4 Memory Panel
- [ ] Keyboard shortcut to toggle visibility
- [ ] Show all items as scrollable cards
- [ ] Each card: view, edit, delete buttons
- [ ] Clear all button (confirm dialog)

### 7.5 Offline Task Packs
- [ ] Package 1: Page Summarizer (local assets + agent config)
- [ ] Package 2: PDF Analyzer
- [ ] Package 3: Code Explainer
- [ ] Package 4: Dataset Cleaner
- [ ] Package 5: Note-Taker
- [ ] Package 6: Form Filler
- [ ] Installation: Download + extract to ~/.flxtra/taskpacks/

### 7.6 Phase 7 Verification
- [ ] UI feels minimal (Comet/Arc standard)
- [ ] All task packs install + declare correctly
- [ ] Commands execute expected agents

---

## Phase 8: Dynamic Ad Blocking + Security Hardening

### 8.1 ONNX Classifier Integration
- [ ] Train or find pre-trained ONNX classifier for fingerprinting scripts
- [ ] Integrate with flxtra_filter
- [ ] Test: Classifier identifies suspicious scripts vs benign ones

### 8.2 Dynamic Blocking
- [ ] Evaluate non-static-blocked scripts with ONNX model
- [ ] Quarantine suspicious scripts (don't execute)
- [ ] Surface to user with explanation
- [ ] Allow user to approve specific scripts

### 8.3 Injection Attack Prevention
- [ ] Create test suite: 20 known injection payloads
- [ ] Embed each in real HTML content
- [ ] Agent should detect + block all 20
- [ ] Log each detection
- [ ] Test: 20/20 blocked, user surfaced the attempts

### 8.4 Cross-Tab Isolation Testing
- [ ] Attempt to share data between tabs via various methods
- [ ] localStorage, sessionStorage, IndexedDB - all tab-scoped
- [ ] Shared JavaScript object - impossible
- [ ] Network tagging - maintained
- [ ] Test: No data leakage detected

### 8.5 Performance Profiling + Optimization
- [ ] Profile agent execution times
- [ ] Profile rendering times
- [ ] Profile LLM inference times
- [ ] Identify bottlenecks + optimize top 3
- [ ] Measure impact

### 8.6 Phase 8 Verification
- [ ] Injection test suite: 20/20 blocked
- [ ] Cross-tab isolation test passes
- [ ] No regressions on Phase 7

---

## Phase 9: Release Prep

### 9.1 Documentation
- [ ] Write user README (installation, first run, model download, command bar examples)
- [ ] Write architecture.md (three-layer model explanation)
- [ ] Write CONTRIBUTING.md
- [ ] Write SECURITY.md (privacy guarantees)

### 9.2 Binary Releases
- [ ] Set up GitHub Actions CI/CD
- [ ] Create build matrix: Linux x86_64, macOS ARM64, macOS x86_64, Windows x86_64
- [ ] Generate release artifacts
- [ ] Test: Download + run binary on each platform

### 9.3 Product Positioning
- [ ] Write one-page brief: three layers + why different from Brave/Arc/Comet
- [ ] Create comparison table (features vs competitors)
- [ ] Prepare HN / Reddit post

### 9.4 Launch
- [ ] Tag release v0.1.0
- [ ] Publish binaries
- [ ] Post on HN, /r/rust, privacy communities
- [ ] Collect feedback

### 9.5 Phase 9 Verification
- [ ] All binaries downloadable
- [ ] Docs are clear + complete
- [ ] CI is green

---

## Cross-Cutting Concerns (Throughout All Phases)

### Testing
- [ ] Each phase has integration tests that verify gate
- [ ] No feature ships without tests
- [ ] `cargo test` is run after each phase before proceeding

### Security Review
- [ ] Each crate reviewed for: telemetry, fingerprinting vectors, data exfil
- [ ] Injection patterns checked in Phase 8
- [ ] Privacy guarantee audit per Section 8 of Master Prompt

### Documentation
- [ ] Each crate: lib.rs explains scope
- [ ] Each phase: Verification gates documented
- [ ] README kept current

### Performance Targets
- [ ] Agent response time: <3s for typical task (excluding LLM)
- [ ] LLM inference: <10s for 500-word summary (Phi-3 Mini)
- [ ] Page render: <100ms for static page
- [ ] Memory: <500MB base + model size

---

END OF TODO
