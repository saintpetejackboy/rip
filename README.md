# RIP - Rest In Peace, Vulnerabilities 🪦

<div align="center">

**A blazing-fast terminal-based security scanner that finds exposed secrets and vulnerabilities**

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)](https://opensource.org/licenses/MIT)
[![NPM](https://img.shields.io/badge/NPM-%23CB3837.svg?style=for-the-badge&logo=npm&logoColor=white)](https://www.npmjs.com/package/rip)

</div>

## 🚀 Quick Start

### Install & Run (NPX - Recommended)
```bash
# Run immediately without installation
npx rip

# Or install globally
npm install -g rip
rip --help
```

### Build from Source
```bash
# Clone the repository
git clone https://github.com/saintpetejackboy/rip
cd rip

# Build with Cargo
cargo build --release

# Run the scanner
./target/release/rip --help
```

## ✨ Features

### 🔍 **Local File Scanning**
- **Lightning-fast** secret detection using grep-searcher engine
- **Smart .env parsing** with trivial value filtering
- **Comprehensive coverage** for 15+ programming languages
- **Configurable ignore patterns** (.git, node_modules, etc.)
- **Real-time progress bars** and colored output

### 🌐 **Web Vulnerability Scanning**
- **Exposed file detection** (.env, .git/config, backups)
- **Security header analysis** (CSP, HSTS, X-Frame-Options)
- **Information disclosure** via robots.txt
- **HTTP endpoint enumeration** with smart timeouts

### 🎨 **Beautiful Terminal UI**
- **Interactive setup** with multi-select menus
- **ASCII art branding** and professional styling
- **Severity-based color coding** (Critical → Low)
- **Detailed vulnerability reports** with recommendations

### ⚙️ **Flexible Configuration**
- **Auto mode** for CI/CD pipelines (`--auto`)
- **TOML configuration** files (`.ripconfig.toml`)
- **Environment-based** customization
- **Team-shareable** settings

## 🎯 Usage Examples

### Basic Repository Scan
```bash
# Scan current directory with default settings
rip

# Scan specific directory in auto mode (no prompts)
rip --auto scan /path/to/project

# Scan with web vulnerability checking
rip scan --web --url https://example.com
```

### Interactive Configuration
```bash
# Run interactive setup
rip config

# Show current configuration
rip config --show

# Reset to defaults
rip config --reset
```

### Advanced Options
```bash
# Use custom config file
rip --config ./custom-config.toml scan

# Skip configuration loading
rip --skip-config --auto scan

# Force reconfiguration
rip --reconfigure scan
```

## 📋 Command Reference

### Global Flags
```
--auto           Run with default settings, no interactive prompts
--config <FILE>  Path to configuration file
--reconfigure    Force reconfiguration, ignore existing config
--skip-config    Skip configuration loading/saving
-v, --verbose    Enable verbose output
-p, --path <PATH> Repository path to scan (default: current directory)
-h, --help       Print help
-V, --version    Print version
```

### Commands
```
scan             Scan for vulnerabilities (default operation)
config           Configure RIP settings interactively
version          Display version information
help             Print help message
```

## 🔧 Configuration

RIP uses a `.ripconfig.toml` file for persistent settings:

```toml
repository_path = "."
env_filename = ".env"
env_keys = ["API_KEY", "DATABASE_URL", "STRIPE_SECRET_KEY", "JWT_SECRET"]
file_extensions = ["js", "ts", "jsx", "tsx", "py", "rb", "php", "java", "go", "rs"]
ignore_patterns = ["node_modules", ".git", "target", "dist", "build", ".next"]
enable_web_scan = false
web_url = ""
```

### Supported File Types
- **JavaScript/TypeScript**: `.js`, `.ts`, `.jsx`, `.tsx`
- **Python**: `.py`
- **Ruby**: `.rb`
- **PHP**: `.php`
- **Java**: `.java`
- **Go**: `.go`
- **Rust**: `.rs`
- **C/C++**: `.c`, `.cpp`
- **C#**: `.cs`
- **YAML/JSON**: `.yaml`, `.yml`, `.json`
- **Markup**: `.xml`, `.md`, `.txt`

## 📊 Example Output

```bash
$ rip --auto scan

 ██▀███   ██▓ ██▓███  
▓██ ▒ ██▒▓██▒▓██░  ██▒
▓██ ░▄█ ▒▒██▒▓██░ ██▓▒
▒██▀▀█▄  ░██░▒██▄█▓▒ ▒
░██▓ ▒██▒░██░▒██▒ ░  ░
Rest In Peace, Vulnerabilities

[RIP-SCAN] Starting vulnerability scan...
Scanning path: .
Scanning for 6 environment variable values...

████████████████████████████████████████ 100% [00:00:01] Complete!

Scan Results
Files scanned: 847
Scan duration: 1.23s
Log file: /tmp/rip-20250127_143021.log
Found 4 potential vulnerabilities:

File: ./src/config.js
  Line:12 Key: API_KEY
    const apiKey = 'sk-live_1234567890abcdef'
  Line:18 Key: DATABASE_URL  
    dbUrl: 'postgresql://user:password@db.company.com:5432/prod'

Recommendation: Review these files to ensure secrets are not exposed.

[RIP-SCAN] Scan complete!
```

## 🚀 CI/CD Integration

### GitHub Actions
```yaml
name: Security Scan
on: [push, pull_request]
jobs:
  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run RIP Security Scan
        run: npx rip --auto scan
```

### GitLab CI
```yaml
security_scan:
  script:
    - npx rip --auto scan
  only:
    - merge_requests
    - master
```

## 🔐 Security & Privacy

- **Read-only operations**: RIP never modifies your files
- **Local processing**: No data is sent to external servers
- **Minimal permissions**: Runs with standard user privileges
- **Secure defaults**: Sensitive patterns excluded from logs
- **Open source**: Full transparency with community review

## 🛠️ Development

### Prerequisites
- Rust 1.70+ 
- Cargo package manager
- Git

### Building
```bash
# Clone repository
git clone https://github.com/saintpetejackboy/rip
cd rip

# Install dependencies and build
cargo build

# Run tests
cargo test

# Format code
cargo fmt

# Run linter
cargo clippy
```

### Project Structure
```
rip/
├── src/
│   ├── main.rs           # Application coordinator
│   ├── cli.rs            # Command-line interface
│   ├── config.rs         # Configuration management
│   ├── scanner.rs        # Local file scanning
│   ├── tui.rs            # Terminal user interface
│   └── web_scanner.rs    # Web vulnerability scanning
├── art/                  # ASCII art assets
├── tests/                # Integration tests
├── md/                   # Documentation
├── TODO/                 # Development roadmap
└── Cargo.toml           # Rust dependencies
```

## 📈 Performance

- **Small repos** (< 1MB): Sub-second scanning
- **Medium repos** (1-10MB): 1-5 seconds
- **Large repos** (10-100MB): 5-30 seconds
- **Memory usage**: < 50MB typical
- **CPU usage**: Single-threaded, efficient regex engine

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md) for details.

### Quick Contribution Guide
1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes and add tests
4. Run the full test suite: `cargo test`
5. Format your code: `cargo fmt`
6. Submit a pull request

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙋 Support

- **Documentation**: Check the `md/` directory for detailed guides
- **Issues**: Report bugs on [GitHub Issues](https://github.com/saintpetejackboy/rip/issues)
- **Security**: For security vulnerabilities, email security@saintpetejackboy.com
- **Community**: Join discussions in [GitHub Discussions](https://github.com/saintpetejackboy/rip/discussions)

## 🎯 Roadmap

- [ ] **Enhanced secret detection** with entropy analysis
- [ ] **Git history scanning** for committed secrets
- [ ] **JSON/SARIF output** formats for tool integration
- [ ] **Docker image** for containerized scanning
- [ ] **VS Code extension** with real-time detection
- [ ] **CI/CD plugins** for major platforms

See [TODO/NEXT-STEPS.md](TODO/NEXT-STEPS.md) for detailed roadmap.

---

<div align="center">

**Built with ❤️ by the security community**

[⭐ Star on GitHub](https://github.com/saintpetejackboy/rip) | [📦 View on NPM](https://www.npmjs.com/package/rip) | [📖 Read the Docs](md/)

</div>