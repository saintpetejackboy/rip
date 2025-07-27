# RIP Architecture Overview

## 🏗️ System Architecture

RIP follows a modular, clean architecture pattern with clear separation of concerns:

```
┌─────────────────────────────────────────────────────────────┐
│                        CLI Layer                            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │   clap      │  │ dialoguer   │  │    colored          │ │
│  │ (parsing)   │  │    (TUI)    │  │   (styling)         │ │
│  └─────────────┘  └─────────────┘  └─────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                       │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │    main     │  │   config    │  │        tui          │ │
│  │ (coordinator)│  │ (settings)  │  │  (interactive)      │ │
│  └─────────────┘  └─────────────┘  └─────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                    Scanning Layer                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │   scanner   │  │web_scanner  │  │    indicatif        │ │
│  │   (local)   │  │  (remote)   │  │   (progress)        │ │
│  └─────────────┘  └─────────────┘  └─────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                    Engine Layer                            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │grep-searcher│  │   reqwest   │  │       serde         │ │
│  │ (file scan) │  │ (HTTP client)│  │  (serialization)   │ │
│  └─────────────┘  └─────────────┘  └─────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## 📦 Module Breakdown

### 🎯 Core Modules

#### `main.rs` - Application Coordinator
- **Responsibility**: Orchestrates the entire application flow
- **Key Functions**:
  - Command-line argument processing
  - Configuration loading with fallback logic
  - Scan workflow coordination
  - Error handling and reporting
- **Dependencies**: All other modules
- **Entry Points**: `main()`, `load_config()`, `run_scan()`

#### `cli.rs` - Command Line Interface  
- **Responsibility**: Defines CLI structure and argument parsing
- **Key Components**:
  - `Cli` struct with clap derives
  - `Commands` enum for subcommands
  - Flag definitions and help text
- **Features**: Auto-completion, validation, help generation
- **Integration**: Used by main.rs for argument processing

#### `config.rs` - Configuration Management
- **Responsibility**: Handles all configuration logic
- **Key Features**:
  - TOML serialization/deserialization
  - `.env` file parsing with smart filtering
  - Default configuration generation
  - File I/O operations
- **Data Structures**: `Config` struct with comprehensive settings
- **Storage**: `.ripconfig.toml` file in project root

### 🔍 Scanning Modules

#### `scanner.rs` - Local File Scanner
- **Responsibility**: High-performance local file scanning
- **Engine**: grep-searcher with regex matching
- **Key Features**:
  - Recursive directory traversal
  - File extension filtering
  - Ignore pattern support
  - Progress tracking with indicatif
  - Detailed result reporting
- **Output**: `ScanResults` with matches and metadata
- **Performance**: Optimized for large codebases

#### `web_scanner.rs` - Web Vulnerability Scanner
- **Responsibility**: Remote HTTP endpoint analysis
- **Engine**: reqwest HTTP client with async/await
- **Vulnerability Types**:
  - Exposed sensitive files (.env, .git, backups)
  - Missing security headers
  - Information disclosure via robots.txt
  - Insecure header configurations
- **Output**: `WebScanResults` with severity classification
- **Features**: Timeout handling, user-agent spoofing

### 🎨 User Interface Modules

#### `tui.rs` - Terminal User Interface
- **Responsibility**: Interactive configuration setup
- **Components**:
  - Multi-select menus with dialoguer
  - Input prompts with validation
  - Colored output for better UX
  - Configuration summary display
- **Flow**: Repository → Env File → Keys → Extensions → Patterns → Web → Save
- **Integration**: Called by main.rs for interactive mode

## 🔄 Data Flow

### 1. Initialization Flow
```
User Input → CLI Parsing → Config Loading → Mode Selection
     ↓
Interactive Setup (if needed) → Config Validation → Scan Preparation
```

### 2. Scanning Flow
```
Config → Environment Parsing → File Discovery → Pattern Matching
     ↓
Progress Tracking → Result Aggregation → Output Formatting → Log Generation
```

### 3. Web Scanning Flow (Optional)
```
URL Input → HTTP Client Setup → Endpoint Testing → Header Analysis
     ↓
Vulnerability Classification → Severity Assessment → Report Generation
```

## 🎯 Design Patterns

### 1. **Builder Pattern**
- Used in `SearcherBuilder` for grep configuration
- Used in `Client::builder()` for HTTP client setup
- Provides flexible, readable configuration

### 2. **Strategy Pattern**
- Different scanning strategies (local vs web)
- Configurable ignore patterns and file extensions
- Extensible for future scanner types

### 3. **Observer Pattern**
- Progress reporting via indicatif
- Real-time scan status updates
- Non-blocking UI updates

### 4. **Factory Pattern**
- Configuration creation with defaults
- Regex matcher instantiation
- Result structure generation

## 🔧 Key Technologies

### Core Dependencies
- **clap 4.5**: CLI argument parsing and help generation
- **serde 1.0**: Serialization for config files
- **toml 0.8**: Human-readable configuration format
- **anyhow 1.0**: Error handling and propagation
- **tokio 1.0**: Async runtime for HTTP operations

### Scanning Engine
- **grep-searcher 0.1**: High-performance file content search
- **grep-matcher 0.1**: Pattern matching engine
- **grep-regex 0.1**: Regex support for complex patterns
- **regex 1.0**: Additional regex utilities

### User Interface
- **dialoguer 0.11**: Interactive terminal prompts
- **indicatif 0.17**: Progress bars and spinners
- **colored 2.0**: Terminal color support
- **chrono 0.4**: Timestamp generation for logs

### Network & HTTP
- **reqwest 0.12**: HTTP client with rustls
- **rustls**: Pure Rust TLS implementation (no OpenSSL)

## ⚡ Performance Characteristics

### Scanning Performance
- **File I/O**: Memory-mapped files via grep-searcher
- **Pattern Matching**: Optimized regex engine
- **Concurrency**: Single-threaded with async I/O for web
- **Memory Usage**: Minimal footprint, streaming processing

### Scalability Metrics
- **Small repos** (< 1MB): Sub-second scanning
- **Medium repos** (1-10MB): 1-5 second scanning  
- **Large repos** (10-100MB): 5-30 second scanning
- **Enterprise repos** (100MB+): Configurable timeouts

## 🔐 Security Design

### Defensive Principles
- **Read-only operations**: No file modification capabilities
- **Sandboxed scanning**: No code execution
- **Minimal privileges**: Standard user permissions only
- **No data exfiltration**: Results stored locally only

### Data Handling
- **Sensitive data**: Never logged or transmitted
- **Temporary files**: Secure cleanup in /tmp
- **Configuration**: User-controlled, no defaults with credentials
- **Network requests**: Optional, user-initiated only

## 🔮 Extensibility Points

### 1. **Scanner Plugins**
- New scanner types can implement common interface
- Examples: binary analysis, dependency scanning, git history

### 2. **Output Formats**  
- Additional result formatters (JSON, SARIF, HTML)
- Custom report templates
- Integration with external tools

### 3. **Configuration Sources**
- Environment variables
- Remote configuration
- Team-shared policies

### 4. **Web Scanner Modules**
- Custom vulnerability checks
- Domain-specific scanners
- Integration with security APIs

---

This architecture provides a solid foundation for RIP's current functionality while enabling future enhancements and community contributions.