# FLXTRA Build Plan (AEGIS BROWSER)

## Current State Assessment

### Existing Crates
- ✅ flxtra_core - Lightweight utilities, types
- ✅ flxtra_net - Network stack with DoH foundation
- ✅ flxtra_filter - Ad/tracker blocking started
- ✅ flxtra_browser - Main entry point (WebView2-based ATM)
- ✅ flxtra_mcp - MCP tool registry started
- ✅ flxtra_web - Next.js UI shell (separate from core)

### Missing From Master Spec
- ❌ flxtra_ui - UI shell (currently using WebView2)
- ❌ flxtra_render - wgpu compositor (custom rendering pipeline)
- ❌ flxtra_layout - Box model/CSS layout engine
- ❌ flxtra_html - html5ever parser
- ❌ flxtra_css - CSS parser + style resolver
- ❌ flxtra_js - deno_core wrapper for JS runtime
- ❌ flxtra_sandbox - Per-tab process isolation
- ❌ flxtra_agents - Agent coordinator + specialized agents
- ❌ flxtra_llm - Local LLM runtime wrapper
- ❌ flxtra_memory - Encrypted local store (redb)

## Key Architectural Decision Point

**AMBIGUITY #1**: The README uses WebView2 (embedded Chromium), but the Master Prompt specifies custom rendering with wgpu, html5ever, and custom layout. 
- **Question**: Is the scope to:
  A) Extend current WebView2 approach with AI agents + privacy features?
  B) Build complete custom rendering stack per Master Spec?

**For this work**, we will assume **Approach B** (custom rendering as per Master Spec) because:
1. The Master Prompt explicitly calls for this
2. WebView2 dependency means data can still leak through Chromium
3. Per-tab sandboxing requires more control than WebView2 grants

## Implementation Strategy

### Phase 0: Repo Hygiene + Architecture Setup
**Goal**: Clean workspace structure, proper crate layout, verification framework
**Tasks**:
1. [ ] Verify .gitignore excludes target/, *.gguf, *.onnx, node_modules, .env
2. [ ] Move existing WebView2 code to separate "legacy" branch or archive
3. [ ] Create missing crate stubs in Cargo.toml workspace members
4. [ ] Verify `cargo build --release` compiles (may have errors, that's OK)
5. [ ] Set up test infrastructure (cargo test -p flxtra_*)
6. [ ] Document component boundaries in each crate's lib.rs

**Verification**: `cargo build --release` exits 0, `.git status` shows clean tree

### Phase 1: Network + Privacy Foundation
**Goal**: DoH working, filter rules loaded, telemetry firewall active
**Focus Crates**: flxtra_net, flxtra_filter

**Tasks**:
1. [ ] flxtra_net: Verify hickory-dns DNS-over-HTTPS integration
2. [ ] flxtra_net: Implement telemetry firewall (blocks outgoing telemetry patterns)
3. [ ] flxtra_filter: Wire adblock-rust static blocking
4. [ ] flxtra_filter: Create test harness with known tracker domains
5. [ ] Integration test: Tracker domain blocked, DoH verified active

**Verification**: `cargo test -p flxtra_net` and `cargo test -p flxtra_filter` pass

### Phase 2: HTML + CSS + Layout + Rendering Pipeline
**Goal**: Render a simple static page correctly
**Focus Crates**: flxtra_html, flxtra_css, flxtra_layout, flxtra_render

**Tasks**:
1. [ ] flxtra_html: Integrate html5ever, parse DOM to AST
2. [ ] flxtra_css: Integrate selectors crate, resolve CSS rules
3. [ ] flxtra_layout: Implement block/inline box model
4. [ ] flxtra_render: Integrate wgpu, paint layout tree to screen
5. [ ] Test target: Render Wikipedia main page, verify heading hierarchy + link positions

**Verification**: Screenshot matches reference image within pixel tolerance

### Phase 3: JavaScript Runtime + Sandbox
**Goal**: JS execution works, per-tab isolation works, tab crash doesn't crash browser
**Focus Crates**: flxtra_js, flxtra_sandbox

**Tasks**:
1. [ ] flxtra_js: Integrate deno_core + V8
2. [ ] Expose minimal Web APIs (fetch, setTimeout, DOM bridge)
3. [ ] DOM bridge: JS calls map to Rust DOM tree (no raw DOM reference)
4. [ ] flxtra_sandbox: Implement OS-level process isolation (seccomp on Linux, AppContainer on Windows)
5. [ ] IPC protocol: Typed message passing between sandbox + main process
6. [ ] Test: React SPA renders correctly, tab crash recovery works

**Verification**: `cargo test -p flxtra_js` and `cargo test -p flxtra_sandbox` pass

### Phase 4: MCP Tool Bus + Agent Core
**Goal**: Tool registry works, Summarization Agent completes E2E
**Focus Crates**: flxtra_mcp, flxtra_agents (new)

**Tasks**:
1. [ ] flxtra_mcp: Define tool traits as async interfaces
2. [ ] Implement core tools: read_page, get_page_text, click, type_text, scroll, navigate, write_file, read_file
3. [ ] flxtra_agents: Create Coordinator struct
4. [ ] Implement Summarization Agent (routes intent → tool calls → LLM call → result)
5. [ ] Use mock LLM (returns fixed string for now)
6. [ ] E2E test: "summarize this page" → coordinator → agent → UI

**Verification**: Summarization Agent successfully completes on test page

### Phase 5: Local LLM Runtime
**Goal**: Real LLM loaded, fast inference on consumer hardware
**Focus Crates**: flxtra_llm (new)

**Tasks**:
1. [ ] flxtra_llm: Integrate candle or llama-cpp-rs
2. [ ] Model manager: Downloaded local GGUF models
3. [ ] Replace mock LLM with real runtime
4. [ ] Test: Phi-3 Mini summarizes 500-word article in <10s

**Verification**: Phi-3 Mini loads, produces summary <10s on test hardware

### Phase 6: Full Agent Fleet + Memory
**Goal**: All agents work, memory store encrypted, templates save/load
**Focus Crates**: flxtra_agents, flxtra_memory (new)

**Tasks**:
1. [ ] Implement Research Agent
2. [ ] Implement Automation Agent  
3. [ ] Implement Scraping Agent
4. [ ] Implement Security Agent
5. [ ] flxtra_memory: Integrate redb + AES-GCM encryption
6. [ ] Memory panel: View/edit/delete all stored items
7. [ ] Template system: Record + playback automation workflows

**Verification**: All automation scenarios from Master Spec pass

### Phase 7: UI Polish + Task Packs + Templates
**Goal**: Minimal Arc-like UI, offline task packs installed, templates work
**Focus Crates**: flxtra_ui (new, or extend flxtra_browser)

**Tasks**:
1. [ ] Command bar: URL + search + natural language classification
2. [ ] Trust score badge: Real-time site analysis + expandable breakdown
3. [ ] Agent status strip: Live updates while agent is active
4. [ ] Memory panel: Keyboard shortcut to view/manage memory
5. [ ] Package offline task packs: Page Summarizer, PDF Analyzer, Code Explainer, etc.
6. [ ] Template UI: Save/load/run multi-step automations

**Verification**: UI feels minimal (Comet/Arc standard), all task packs install

### Phase 8: Dynamic Ad Blocking + Security Hardening
**Goal**: ONNX classifier detects suspicious scripts, injection attempts blocked, performance optimized
**Focus Crates**: flxtra_filter, flxtra_agents

**Tasks**:
1. [ ] Train/integrate ONNX classifier for fingerprinting detection
2. [ ] Dynamic blocking: Quarantine suspicious scripts, surface to user
3. [ ] Injection test suite: 20 known payloads embedded in web content
4. [ ] Verify all 20 payloads detected + blocked
5. [ ] Cross-tab isolation test: Attempt data leakage, verify fails
6. [ ] Performance profile + optimization

**Verification**: Injection test suite: 20/20 blocked, no regressions

### Phase 9: Release Prep
**Goal**: Distributable binary, documentation complete, market positioning clear
**Tasks**:
1. [ ] Write user README (install, first run, model download, command bar)
2. [ ] Generate binary releases: Linux x86_64, macOS ARM+x86_64, Windows x86_64
3. [ ] GitHub Actions CI: cargo build + cargo test on push
4. [ ] One-page product brief: Three layers + market positioning
5. [ ] Open source marketing: HN, Reddit /r/rust, etc.

**Verification**: Binaries downloadable, docs clear, CI green

---

## Verification Gates (Non-Negotiable)

### After Phase 0
- `cargo build --release` exits 0
- `git status` shows clean tree (no build artifacts committed)

### After Phase 1  
- `cargo test -p flxtra_net` exits 0
- `cargo test -p flxtra_filter` exits 0
- Known tracker domains blocked in integration test
- DoH active (packet inspection confirms)

### After Phase 2
- Simple Wikipedia page renders with correct heading hierarchy
- Link positions match reference screenshot within tolerance
- Images placed correctly

### After Phase 5
- Phi-3 Mini loads
- 500-word article summarized in <10s
- Summary is coherent (manual review)

### After Phase 8
- Injection test suite: 20/20 payloads detected + blocked
- Cross-tab isolation holds (attempted leakage fails)
- No regressions on Phase 7 functionality

### Before Release
- All 9 phase gates pass
- CI is green
- Binaries are downloadable

---

## Critical Dependencies & Assumptions

1. **Rust 1.75+** - Project targets stable
2. **OS-level sandbox support** - Windows AppContainer, Linux seccomp, macOS App Sandbox
3. **llama.cpp compatible runtime** - Candle or llama-cpp-rs works with GGUF models
4. **wgpu support on target platforms** - Must work on Linux, macOS, Windows
5. **deno_core availability** - Used for JS runtime
6. **HTTPS + DNS-over-HTTPS ISPs** - Some ISPs may filter DoH (acceptable tradeoff)

---

## Tool & Crate Manifest

| Crate | Purpose | Key Deps | Status |
|-------|---------|----------|--------|
| flxtra_core | Types, traits, error handling | tokio, serde | ✅ Exists |
| flxtra_net | Network + DoH + telemetry firewall | hickory-dns, rustls, hyper | ✅ Exists |
| flxtra_filter | Static + dynamic ad/tracker blocking | adblock-rust, onnxruntime | ❌ Needs ONNX |
| flxtra_html | HTML parsing | html5ever | ⚠️ Needs creation |
| flxtra_css | CSS parsing + style resolution | selectors | ⚠️ Needs creation |
| flxtra_layout | Box model + layout engine | - | ❌ New |
| flxtra_render | wgpu compositor | wgpu, gpu-alloc | ❌ New |
| flxtra_js | Deno_core + Web APIs | deno_core, rusty_v8 | ❌ New |
| flxtra_sandbox | Per-tab OS isolation | platform-specific syscalls | ❌ New |
| flxtra_mcp | Tool registry + bus | async-trait, tokio | ✅ Exists (stub) |
| flxtra_agents | Coordinator + agents | flxtra_mcp, flxtra_llm | ❌ New |
| flxtra_llm | Local LLM runtime | candle or llama-cpp-rs | ❌ New |
| flxtra_memory | Encrypted redb store | redb, aes-gcm | ❌ New |
| flxtra_ui | Minimal UI (Tauri or native) | tauri or winapi | ⚠️ Needs rework |
| flxtra_browser | Main entry point | all of above | ⚠️ Needs refactor |

---

## Next Immediate Actions (Phase 0)

1. **Verify current build** - Does `cargo build --release` work?
2. **Clean .gitignore** - Remove target/ if committed
3. **Create crate stubs** - Add missing crates to Cargo.toml
4. **Create test structure** - Set up tests/ dirs in each crate
5. **Document boundaries** - Each crate's lib.rs explains its scope
