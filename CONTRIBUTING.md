# Contributing to DustMap

Thank you for your interest in contributing to DustMap! DustMap is an open-source, fast, and transparent storage analyzer and conservative cleaner for macOS.

## Core Philosophy

- **Trust before popularity:** Storage tools touch user data. We prioritize transparent numbers, explainable recommendations, Trash-first deletion, and no telemetry.
- **DCO Sign-off:** All commits must be signed off with Developer Certificate of Origin (`git commit -s`).
- **No telemetry:** We do not track users or collect data.

## Getting Started

### Prerequisites

- macOS 13+ (Apple Silicon or Intel)
- Rust 1.80+ (`rustup`)
- Node.js 20+ & npm

### Building the Project

DustMap uses a Cargo workspace:

```bash
# Clone the repository
git clone https://github.com/abhiudayg/dustmap.git
cd dustmap

# Build the Rust workspace (core, cli, rules, app)
cargo build --workspace

# Run tests
cargo test --workspace

# Check formatting and clippy
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

## Contributing Cleanup Rules (No Rust required!)

Cleanup knowledge lives in declarative TOML files in `rules/`. You can contribute or update rules without writing any Rust code!

1. Create or edit a rule in `rules/<rule-id>.toml`.
2. Follow the TOML schema specified in PRD §9.1.
3. Validate your rule using `dustmap rules validate` or `cargo test -p dustmap-rules`.
4. Submit a Pull Request.

## Pull Request Checklist

- [ ] All Rust tests pass: `cargo test --workspace`
- [ ] Code is formatted: `cargo fmt --all`
- [ ] Clippy has no warnings: `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] Commits have DCO sign-off (`-s`)
- [ ] Documentation updated where relevant
