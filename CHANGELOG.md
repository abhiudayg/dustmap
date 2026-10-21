# Changelog

All notable changes to DustMap will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-10-21

### Initial Release

DustMap is a high-performance, conservative macOS disk space visualizer and cleaner built with a modular Rust workspace core, Tauri 2 shell, and Svelte 5 runes-based frontend.

#### Added
- **High-Performance Storage Traversal**:
  - Multi-threaded parallel recursive filesystem scanner powered by `rayon` and `walkdir`.
  - APFS container and volume enumeration using `sysinfo` with real-time capacity and firm-link awareness.
  - Zero-lag in-memory indexing (`Index` / `EntryId`) with bottom-up post-order directory size aggregation.
- **Interactive Visualizers**:
  - Responsive squarified treemap layout with custom category color coding and nested drill-down navigation.
  - Dynamic telemetry sector donut breakdown scoped to subfolders with live item counts and sizes.
  - Sleek top-folders panel with proportional size bars, Finder-style rank, and one-click actions.
  - Breadcrumb path bar for seamless directory traversal.
  - Minimalist single-line volume selector with detailed disk usage breakdown.
- **Conservative Safety Pipeline**:
  - Two-tier deletion model with default Trash-first routing (`trash` crate).
  - Explicit confirmation modal requiring deliberate user acknowledgment for permanent deletion.
  - Hardened security barriers: SIP system directory protection, path traversal rejection (`../`), and null-byte injection prevention.
  - Append-only structured JSON audit log recording all deletion operations for auditability.
- **Reporting & Export**:
  - Native telemetry and storage report export saving formatted JSON to `~/Downloads` with instant Finder reveal and UI toast notification.
- **Dual Delivery & Distribution**:
  - Developer ID signed and Apple-notarized macOS disk image (`DustMap_0.1.0_aarch64.dmg`) with drag-and-drop `/Applications` installer.
  - Hardened standalone CLI binary (`dustmap-darwin-arm64`) supporting human-readable terminal output and `--json` mode.
  - Official Homebrew tap formula and cask at `abhiudayg/homebrew-dustmap`.
- **Security & Quality**:
  - 100% verification score across 23 functional and adversarial security jailbreak tests (`.verify/run.sh`).
  - Zero third-party runtime analytics or network requests.
