# 🏛️ DustMap Architecture & Technical Specification

> **Version:** 0.1.0  
> **Target OS:** macOS 13+ (Ventura, Sonoma, Sequoia) — Apple Silicon & Intel  
> **Security Policy:** Zero Data Loss, Trash-First, Offline-Only, No Telemetry  

---

## 1. High-Level System Overview

DustMap is structured as a layered, modular Cargo workspace decoupled from the UI framework, bridged to a high-density macOS client via Tauri 2 IPC and powered by Svelte 5.

```
┌────────────────────────────────────────────────────────────────────────┐
│                        Svelte 5 Reactive Client                        │
│   (VolumeBar, Donut Visualizer, Squarified Treemap, TopFolderList)     │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ Tauri 2 IPC (Strongly-Typed)
┌───────────────────────────────────▼────────────────────────────────────┐
│                    Tauri Native Shell (`dustmap-app`)                  │
│    (State Store, Async Command Dispatcher, Window & Menu Handlers)     │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
    ┌───────────────────────────────┼──────────────────────────────┐
    │                               │                              │
┌───▼──────────────────────────┐ ┌──▼──────────────────────────┐ ┌─▼──────────────────┐
│      `dustmap-core`          │ │      `dustmap-rules`        │ │   `dustmap-cli`    │
│  - Parallel Rayon Scanner    │ │  - TOML Rule Validator      │ │  - Headless Scan   │
│  - In-Memory Flat Index      │ │  - Cache Pattern Matcher    │ │  - JSON Telemetry  │
│  - Post-Order Aggregator     │ │  - Regenerability Schema    │ │  - Script Hooks    │
│  - Trash-First Safety Gate   │ └─────────────────────────────┘ └────────────────────┘
│  - Append-Only Audit Logger  │
│  - APFS Volume Manager       │
└──────────────────────────────┘
```

---

## 2. Core Crates Architecture

### 2.1 `dustmap-core`

The foundational library containing zero frontend dependencies.

#### A. Parallel Traversal Engine (`scanner.rs`)
- Traverses multi-million file APFS hierarchies using a bounded `rayon` work-stealing threadpool combined with `walkdir`.
- **Firmlink & Helper Mount Elimination:** Filters APFS system firmlinks (`/System/Volumes/Data`) and virtual mounts to prevent recursive cycle loops and double-counting.
- **Cancellation & Telemetry:** Exposes atomic cancellation flags (`Arc<AtomicBool>`) and streaming progress channels emitting elapsed seconds, accounted bytes, seen entries, and access permission errors.

#### B. Compact Tree Index & Aggregation (`index.rs`)
- File nodes are stored in a contiguous, flat `Vec<Entry>` addressed by a lightweight newtype `EntryId(u32)`.
- Eliminates heap fragmentation and pointer-chasing overhead of nested tree structures.
- **Post-Order Aggregation Pass:** Traverses in reverse topological order ($O(N)$) to calculate:
  - Exact logical raw bytes (`logical_total`)
  - Physical allocated filesystem blocks (`allocated_total`)
  - Total nested descendant items count (`item_count`)
- Fast sorting methods provide size-descending child listings in $O(K \log K)$ time.

#### C. Metadata & Dual Metric Reconciliation (`metadata.rs`)
- Extends standard POSIX metadata with macOS APFS specifics:
  - **Logical Bytes:** Raw file length from `stat.st_size`.
  - **Allocated Bytes:** Actual storage blocks reserved (`stat.st_blocks * 512`), precisely capturing APFS compression, sparse files, and filesystem cluster boundaries.
  - Inode numbers and device IDs are tracked to detect hardlink deduplication.

#### D. Trash-First Safety Gate & Protection Engine (`safety.rs` & `trash.rs`)
- **System Integrity Protection (SIP) Enforcement:** Rejects any modification to protected OS paths (`/System`, `/usr`, `/bin`, `/sbin`, `/Library`, `/etc`).
- **Path Traversal Shield:** Detects and aborts null-byte injections and directory traversal exploits (`../../`).
- **Boundary Containment:** Enforces that every deletion target strictly resolves within the scanned mount root.
- **Trash-First Execution:** Directs all cleanup actions to macOS native Trash via `trash::delete`, allowing full recovery until the user empties Trash. Permanent deletion requires explicit confirmation and bypass checkboxes.

#### E. Structured Append-Only Audit Logger (`audit.rs`)
- Every deletion and cleanup action is appended to a local JSON-lines log (`~/.dustmap/audit.jsonl`).
- Records timestamp, target path, reclaimed bytes, operator decision (`MovedToTrash` vs `DeletedPermanently`), and outcome.

---

### 2.2 `dustmap-rules`

- Declarative rule engine written in TOML (`rules/*.toml`).
- Validates rule identity, ownership, risk level (`safe`, `review`, `caution`), regenerability notes, and match globs.
- Evaluated against deterministic fixtures in `fixtures/` during CI quality checks.

---

### 2.3 `dustmap-cli`

- Command-line companion utility providing fast, scriptable disk auditing without launching the GUI:
  ```bash
  dustmap scan /Applications --json
  dustmap summary ~
  ```

---

## 3. Frontend Architecture (`app/src/`)

### 3.1 Svelte 5 Runes State Management (`store.svelte.ts`)
- Replaces traditional store subscriptions with Svelte 5 reactivity runes (`$state`, `$derived`, `$effect`).
- **Volume Cache Engine:** Caches scan results per mount point (`volumeCache[mount_point]`). Switching between internal SSDs and external USB drives is instantaneous without re-scanning.
- **Dynamic Donut & Treemap Reactivity:**
  - When viewing root (`/`), displays volume-wide telemetry and top-level categories.
  - When descending into a subfolder (e.g., `/Applications` or `~/Library`), the donut chart, sector arcs, center readouts, and right-hand inspector instantly scope down to that specific directory.

### 3.2 Design System — Obsidian Telemetry
- Strictly adheres to the Stitch Obsidian Telemetry design language:
  - Background: `#0B0E14` (Deep Space Dark) and `#101319` (Telemetry Charcoal)
  - Primary Accents: `#F97316` (Engine Amber / Laser Orange) and `#14B8A6` (APFS Teal)
  - Risk / Purgeable: `#F43F5E` (Purge Rose)
  - Typography: Inter for UI labels, JetBrains Mono for storage telemetry and numbers.

---

## 4. Verification & Jailbreak Hardening Suite (`.verify/`)

DustMap includes a 23-point verification suite executed via `.verify/run.sh`:

| Test Class | Scope | Description |
|------------|-------|-------------|
| `G-BUILD` | Workspace | Clean compilation across all crates |
| `G-FMT` | Formatting | Strict `rustfmt` formatting check |
| `G-CLIPPY` | Static Analysis | Zero clippy warnings with `-D warnings` |
| `G-TEST` | Unit Tests | Full test suite across core, rules, and CLI |
| `G-UI-BUILD` | Frontend | Vite + Bun production bundle verification |
| `S-SCAN-01..02` | Functional | Traversal correctness and hidden file detection |
| `S-AGG-01` | Invariant | Post-order parent byte sum equals descendants sum |
| `S-VOL-01` | Volume | Real APFS and external disk enumeration |
| `S-RULE-01` | Rules | TOML schema validation and integrity |
| `S-AUDIT-01` | Audit | Append-only audit trail logging |
| `S-CLI-01` | CLI | Terminal binary execution and JSON generation |
| `S-SIGN-01` | Signing | Apple Developer ID codesigning verification |
| `F-TRAV-01..02` | Jailbreak | Path traversal (`..`) and null byte injection rejection |
| `F-SIP-01` | Jailbreak | Prevention of deletion targeting SIP system roots |
| `F-ROOT-01` | Jailbreak | Boundary escaping rejection outside scanned target |
| `F-SYMLINK-01` | Jailbreak | Traversal containment on symlinks pointing to root |
| `F-PRIV-01` | Jailbreak | Execution verification without root/sudo elevation |
| `F-CYCLE-01` | Jailbreak | Cyclic directory traversal immunity |

---

## 5. macOS Codesigning & Distribution

- Production binaries and `.app` bundles are code-signed with hardened runtime and entitlements:
  - **Identity:** `Developer ID Application: Abhiuday Gupta (9HSMVVUMVX)`
  - **Entitlements:** Full Disk Access (`entitlements.plist`)
  - **Distribution:** Compressed Universal/ARM64 Disk Image (`DustMap_0.1.0_aarch64.dmg`) with background presentation and Applications symlink.

## 6. Implementation Status (PRD v3.1)

The PRD is not fully complete. Core-engine items are implemented and tested. The open items are UI parity with the reference designs, permanent deletion, and one unverified release path.

| Area | Module(s) | Status |
|---|---|---|
| Journal and reconciliation | `journal.rs` | Done, with tests |
| Denylist and safety gate | `safety.rs`, `trash.rs` | Done, with tests |
| Cleaner pipeline | `cleaner.rs` | Done, with tests; Trash only |
| Tool-command targets | `toolcmd.rs` | Done, with tests |
| Remnant detection | `remnants.rs` | Done, with tests; removal via Trash in App Cleanups |
| Local snapshot report | `snapshots.rs` | Done |
| Rules engine | `dustmap-rules` | Done, with tests |
| Your Data and cask zap | `data.rs`, `packaging/` | Done, with tests |
| Scan cache | `scan_cache.rs` | Implemented; restart flow not yet verified end-to-end |
| CLI subcommands | `dustmap-cli` | Done |
| Permanent deletion | — | Not implemented; decision pending |
| Overview, delete modal, folder explorer, splash | `app/src/components/` | Not yet matched to the reference designs |
| Notarized CI release | `.github/workflows/release.yml` | Notarization verified locally; CI run not yet verified |
