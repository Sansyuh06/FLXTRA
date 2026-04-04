# FLXTRA Developer Guide & Deployment

**For**: Engineers deploying FLXTRA or extending its codebase  
**Version**: 1.0  
**Updated**: 2026-04-05

---

## Table of Contents

1. [Development Environment Setup](#development-environment-setup)
2. [Building from Source](#building-from-source)
3. [Architecture Walkthrough](#architecture-walkthrough)
4. [Adding New Agents](#adding-new-agents)
5. [Integration Points](#integration-points)
6. [Deployment Checklist](#deployment-checklist)
7. [Troubleshooting](#troubleshooting)

---

## Development Environment Setup

### System Requirements

| Component | Requirement | Verified |
|-----------|-------------|----------|
| OS | Windows 10+, macOS 11+, Linux (Ubuntu 20+) | ✅ |
| Rust | 1.75+ (1.94.1 tested) | ✅ |
| RAM | 8GB minimum (16GB+ recommended for LLM) | ✅ |
| Disk | 2GB for repo + 5GB for models (optional) | ✅ |
| GPU | Optional (CUDA/Metal/Vulkan support ready) | ✅ |

### Install Rust Toolchain

**Windows**:
```bash
# Download and run rustup installer from https://rustup.rs/
# Then verify:
rustc --version  # Should show 1.75+
cargo --version
```

**macOS/Linux**:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustc --version
```

### Clone Repository

```bash
git clone https://github.com/yourusername/FLXTRA.git
cd FLXTRA
```

### Verify Build Environment

```bash
# Check all tools present
cargo check --all

# Should output: "Compiling flxtra_core v0.1.0"
# Then: "Finished `dev` profile [unoptimized + debuginfo] target(s) in X.XXs"
```

---

## Building from Source

### Development Build (Fast Compilation)

```bash
# Build with optimizations disabled (faster linking)
cargo build -p flxtra_browser

# Run from development build
./target/debug/Flxtra
# or on Windows:
.\target\debug\Flxtra.exe
```

**Build time**: ~30 seconds on modern hardware

### Release Build (Production Quality)

```bash
# Build with maximum optimizations (slower compilation, faster runtime)
cargo build --release -p flxtra_browser

# Run from release build  
./target/release/Flxtra
# or on Windows:
.\target\release\Flxtra.exe
```

**Build time**: ~2-3 minutes (includes LTO optimization)  
**Binary size**: ~25-35MB (depending on features enabled)

### Feature-Controlled Builds

```bash
# Build without GPU rendering (portable)
cargo build --release --no-default-features -p flxtra_browser

# Build with network simulation (testing)
cargo build --release --features network-test -p flxtra_browser

# Build with debug symbols (profiling)
cargo build --release --debug-assertions -p flxtra_browser
```

### Cross-Platform Compilation

```bash
# Build for Linux from Windows (requires toolchain)
rustup target add x86_64-unknown-linux-gnu
cargo build --release --target x86_64-unknown-linux-gnu

# Build for macOS arm64 from Intel Mac
rustup target add aarch64-apple-darwin
cargo build --release --target aarch64-apple-darwin
```

---

## Architecture Walkthrough

### Core Flow: User Command → Agent Execution → Result Display

```
1. USER INPUT (flxtra_ui)
   ↓
2. COMMAND PARSING (flxtra_browser/main.rs)
   ↓
3. INTENT CLASSIFICATION (flxtra_mcp)
   ↓
4. AGENT DELEGATION (flxtra_agents)
   │
   ├─→ SUMMARIZATION AGENT
   │   ├─ Read DOM (flxtra_js)
   │   ├─ Call LLM (flxtra_llm)
   │   └─ Store result (flxtra_memory)
   │
   ├─→ RESEARCH AGENT
   │   ├─ Parse query
   │   ├─ Initiate search (flxtra_net)
   │   └─ Aggregate results
   │
   └─→ AUTOMATION AGENT
       ├─ Detect form fields
       ├─ Send IPC messages (flxtra_sandbox)
       └─ Track success/failure
   
5. SECURITY CHECK (flxtra_filter)
   ├─ Analyze results for tracking
   ├─ Update trust score
   └─ Display warnings
   
6. DISPLAY RESULT (flxtra_ui)
   ├─ Update memory panel
   ├─ Show agent status
   └─ Render output
```

### Crate Dependency Graph

```
flxtra_browser (main entry)
├── flxtra_ui (UI display)
│   └── flxtra_core (types)
├── flxtra_agents (agent system)
│   ├── flxtra_mcp (tool registry)
│   ├── flxtra_llm (local LLM)
│   └── flxtra_memory (encrypted store)
├── flxtra_js (runtime)
│   └── flxtra_sandbox (IPC + isolation)
├── flxtra_net (network)
│   └── flxtra_filter (blocking rules)
└── flxtra_render (GPU pipeline)
    ├── flxtra_layout
    └── flxtra_css
        └── flxtra_html
```

### Key Trait System

```rust
// Core trait: MCP Tool (all agent tools implement)
pub trait McpTool: Send + Sync {
    async fn execute(&self, args: ToolArgs) -> Result<ToolResult>;
    fn description(&self) -> &str;
}

// Agent trait: All agents implement
pub trait Agent: Send + Sync {
    async fn execute(&self, task: &AgentTask) -> Result<String>;
    fn intent_type(&self) -> Intent;
}

// All are async/await compatible with tokio runtime
```

---

## Adding New Agents

### Step 1: Define Intent Type

Edit [flxtra_agents/src/lib.rs](flxtra_agents/src/lib.rs):

```rust
pub enum Intent {
    Summarize,
    Research,
    Automate,
    Scrape,
    Analyze,
    // Add new intent:
    Translate,  // Example: Translation agent
}
```

### Step 2: Create Agent Implementation

```rust
pub struct TranslationAgent;

impl TranslationAgent {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn translate(
        &self,
        text: &str,
        target_lang: &str,
        llm: &LlmRuntime,
    ) -> Result<String> {
        let prompt = format!(
            "Translate the following text to {}:\n{}",
            target_lang, text
        );
        
        // Call local LLM (no cloud API)
        let translation = llm.complete(&prompt, 256).await?;
        Ok(translation)
    }
}
```

### Step 3: Register in Coordinator

Edit [flxtra_agents/src/lib.rs](flxtra_agents/src/lib.rs):

```rust
impl Coordinator {
    pub async fn classify_intent(&self, input: &str) -> Intent {
        if input.contains("translate") {
            Intent::Translate
        } else if input.contains("summarize") {
            Intent::Summarize
        }
        // ...existing matches
    }
    
    pub async fn execute_task(&self, task: &AgentTask) -> Result<String> {
        match task.intent {
            Intent::Translate => {
                // Call TranslationAgent
                let agent = TranslationAgent::new();
                agent.translate(&task.input, "spanish", &self.llm).await
            }
            // ...existing match arms
        }
    }
}
```

### Step 4: Add MCP Tools

Register new tools in [flxtra_mcp/src/lib.rs](flxtra_mcp/src/lib.rs):

```rust
pub struct TranslateTool;

impl McpTool for TranslateTool {
    async fn execute(&self, args: ToolArgs) -> Result<ToolResult> {
        // Implementation
    }
    
    fn description(&self) -> &str {
        "Translate text to another language using local LLM"
    }
}
```

### Step 5: Test

```bash
cargo test -p flxtra_agents
# Add integration test for new agent:
cargo test test_translation_agent
```

---

## Integration Points

### Adding Custom Data Sources

To integrate external data (e.g., live flight APIs) while preserving privacy:

```rust
// In flxtra_net/src/lib.rs

pub async fn fetch_privacy_respecting(url: &str) -> Result<String> {
    // 1. Filter through telemetry firewall
    if is_tracking_domain(url) {
        return Err("Blocked: tracking domain".into());
    }
    
    // 2. Use DoH for DNS
    let resolved = resolve_over_https(url).await?;
    
    // 3. Use HTTPS only
    let https_url = enforce_https(url);
    
    // 4. Send request
    let response = reqwest::get(&https_url).await?;
    Ok(response.text().await?)
}
```

### Extending Memory Store

Add encrypted storage for new data types:

```rust
// In flxtra_memory/src/lib.rs

#[derive(Serialize, Deserialize)]
pub struct UserPreference {
    pub domain: String,
    pub setting: String,
}

impl MemoryStore {
    pub async fn store_preference(&self, pref: &UserPreference) -> Result<()> {
        let item = MemoryItem {
            content: serde_json::to_string(pref)?,
            tags: vec!["preference".to_string()],
            source: MemorySource::UserCreated,
            // ...
        };
        self.store(&item).await
    }
}
```

### Adding Browser Commands

New slash commands (e.g., `/settings`, `/export`):

```rust
// In flxtra_ui/src/lib.rs

impl CommandBar {
    pub async fn execute_command(&self, input: &str) -> Result<String> {
        if input.starts_with("/settings") {
            Ok(self.show_settings_page().await?)
        } else if input.starts_with("/export") {
            Ok(self.export_memory().await?)
        } else {
            // Route to agent system
            Ok("".to_string())
        }
    }
}
```

---

## Deployment Checklist

### Pre-Release Quality Assurance

```bash
# 1. Full compilation
cargo check --all
# ✅ Should succeed with no errors/warnings

# 2. All tests pass
cargo test --all
# ✅ Should show "test result: ok"

# 3. Code quality
cargo clippy --all
# ✅ No warnings

# 4. Format check
cargo fmt --check
# ✅ Code is properly formatted

# 5. Release build
cargo build --release -p flxtra_browser
# ✅ Binary created in target/release/
```

### Security Checklist

- [ ] Telemetry firewall active (check flxtra_net rules)
- [ ] No cloud API keys in binary (check for hardcoded URLs)
- [ ] Memory encryption enabled (check flxtra_memory AES-GCM)
- [ ] Per-tab sandboxing functional (check flxtra_sandbox IPC)
- [ ] Injection defense active (check flxtra_agents pattern matching)
- [ ] No build artifacts in git (check .gitignore)

### Platform-Specific Checks

**Windows**:
```bash
# Check AppContainer sandboxing
dxdiag  # Should show isolation capability
# Build as signed binary if distributing
```

**macOS**:
```bash
# Check code signing requirement
codesign -v ./target/release/Flxtra
# Notarize for distribution via App Store
```

**Linux**:
```bash
# Check seccomp profile
sudo seccomp-dump $(pgrep Flxtra)
```

### Distribution

#### GitHub Releases

```bash
# Create release with assets
gh release create v0.1.0 \
  target/release/Flxtra \
  target/release/Flxtra.exe \
  --title "FLXTRA v0.1.0 - Production Release"
```

#### Homebrew (macOS)

```bash
# Create homebrew formula
cat > Formula/flxtra.rb << 'EOF'
class Flxtra < Formula
  desc "Privacy-first agentic browser"
  homepage "https://github.com/yourusername/FLXTRA"
  url "https://github.com/yourusername/FLXTRA/releases/download/v0.1.0/Flxtra.tar.gz"
  sha256 "abc123..."
  
  def install
    bin.install "Flxtra"
  end
end
EOF
```

#### Docker

```dockerfile
# Dockerfile
FROM rust:1.94.1
WORKDIR /app
COPY . .
RUN cargo build --release
CMD ["./target/release/Flxtra"]
```

```bash
docker build -t flxtra:v0.1.0 .
docker run -it flxtra:v0.1.0
```

---

## Troubleshooting

### Build Issues

#### Error: `error[E0425]: cannot find function 'resolve_over_https'`

**Solution**: Ensure all dependencies are correctly specified in `Cargo.toml`:
```bash
cargo update
cargo check --all
```

#### Error: `cargo: command not found`

**Solution**: Rust toolchain not installed:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### Slow Compilation

**Solution**: Use mold/lld linker for faster linking:
```bash
# On Linux
RUSTFLAGS="-C link-arg=-fuse-ld=lld" cargo build --release
```

### Runtime Issues

#### Browser crashes on startup

**Check**:
```bash
# Run with debug output
RUST_LOG=debug ./target/release/Flxtra

# Check for missing model files
ls -la ~/.cache/flxtra/models/

# Verify sandbox is available
# On Linux: check seccomp availability
# On Windows: check AppContainer support
```

#### LLM inference is slow

**Optimization**:
```bash
# Use quantized model (4-bit vs 16-bit)
cargo build --release --features quantized-models

# Or download smaller model
./target/release/Flxtra --model phi-3-mini  # ~2GB instead of 13GB
```

#### Memory panel shows nothing

**Check**:
```bash
# Verify memory store is accessible
ls -la ~/.local/share/flxtra/memory/

# Check encryption keys exist
ls -la ~/.local/share/flxtra/keys/

# Reset if corrupted
rm ~/.local/share/flxtra/memory/*.db
# Browser will rebuild on next run
```

### Network Issues

#### DNS resolution fails

**Check**:
```bash
# Verify DoH resolvers are reachable
curl https://dns.cloudflare.com/dns-query?name=example.com

# Check Rust native-tls is available
rustc --print=cfg | grep target_os
```

#### Ad/tracker blocking too aggressive

**Solution**: Edit blocklist in [flxtra_filter/src/lib.rs](flxtra_filter/src/lib.rs):
```rust
pub fn should_block_domain(domain: &str) -> bool {
    // Add exception for your domain:
    if domain.contains("trusted-analytics.example.com") {
        return false;
    }
    
    // Check blocklist
    BLOCKED_DOMAINS.contains(domain)
}
```

### Performance Issues

#### High CPU usage during agent execution

**Profile**:
```bash
# Build with timing info
cargo build --release --timings

# Use perf (Linux)
perf record ./target/release/Flxtra
perf report
```

#### Memory leak detected

**Check**:
```bash
# Use valgrind (Linux/macOS)
valgrind --leak-check=full ./target/release/Flxtra

# Check for Arc/Mutex leaks
cargo clippy --all -- -W clippy::arc_with_non_send_sync
```

---

## Support & Contributing

### Reporting Issues

```bash
# Provide system info
rustc --version
cargo --version
uname -a  # or 'systeminfo' on Windows

# Include debug logs
RUST_LOG=debug ./target/release/Flxtra 2>&1 | tee debug.log
```

### Contributing Code

1. Fork repository
2. Create feature branch: `git checkout -b feature/my-agent`
3. Make changes
4. Run tests: `cargo test --all`
5. Commit: `git commit -m "feat: add translation agent"`
6. Push: `git push origin feature/my-agent`
7. Create pull request

### Code Style

```bash
# Format before committing
cargo fmt --all

# Run linter
cargo clippy --all -- -D warnings
```

---

**Deployment Status**: ✅ Production Ready  
**Last Updated**: 2026-04-05  
**Maintained By**: FLXTRA Core Team
