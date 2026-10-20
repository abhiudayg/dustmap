# .verify — DustMap Verification Suite

The `.verify` directory contains the executable quality gates, functional scenarios (`S-*`), and adversarial/jailbreaking safety scenarios (`F-*`) for DustMap.

## Running Verification

```bash
# Run everything: Quality gates, Functional scenarios, and Jailbreaking tests
.verify/run.sh

# Run specific scenarios
.verify/run.sh --only S-SCAN-01,F-SIP-01

# Run without gates (scenarios only)
.verify/run.sh --no-gates

# List all gates and scenarios
.verify/run.sh --list
```

## Scenarios & Quality Gates

| ID | Kind | Severity | Description |
|---|---|---|---|
| `G-BUILD` | Gate | Critical | Rust workspace builds cleanly |
| `G-FMT` | Gate | Medium | Rust formatting is clean |
| `G-CLIPPY` | Gate | High | Clippy reports no warnings or errors |
| `G-TEST` | Gate | Critical | Full test suite passes |
| `G-UI-BUILD` | Gate | Critical | Svelte 5 frontend builds cleanly |
| `S-SCAN-01` | Functional | Critical | Directory hierarchy traversal and byte size calculations |
| `S-SCAN-02` | Functional | High | Hidden directory and dotfile discovery |
| `S-AGG-01` | Functional | Critical | Post-order tree aggregation invariant |
| `S-VOL-01` | Functional | High | System & external volume enumeration |
| `S-RULE-01` | Functional | High | TOML cleanup rule schema parsing and validation |
| `S-AUDIT-01` | Functional | High | Structured append-only audit logging of deletion |
| `S-CLI-01` | Functional | High | CLI execution with text and JSON outputs |
| `S-SIGN-01` | Functional | Critical | Apple Developer ID codesigning verification |
| `F-TRAV-01` | Jailbreak | Critical | Path traversal injection (`../../etc/passwd`) rejected |
| `F-TRAV-02` | Jailbreak | Critical | Null byte path injection (`\0`) rejected |
| `F-SIP-01` | Jailbreak | Critical | SIP/system directory deletion (`/System`, `/usr`, `/bin`) blocked |
| `F-ROOT-01` | Jailbreak | Critical | Deletion target escaping outside authorized scan root blocked |
| `F-SYMLINK-01`| Jailbreak | Critical | Symlink pointing to system directories blocked from deletion |
| `F-PRIV-01` | Jailbreak | High | Application executes without root/sudo elevation |

## Report

The runner writes `.verify/report.json` with a weighted score:
- Critical = 5 points
- High = 3 points
- Medium = 2 points
- Low = 1 point

Verdict is **PASS** only when all critical/high scenarios pass and the score is ≥ 95%.
