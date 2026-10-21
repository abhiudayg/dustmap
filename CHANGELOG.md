# Changelog

All notable changes to DustMap will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-10-21

### Added
- Cleanup journal with startup reconciliation, corrupt-journal quarantine, and an acknowledgement step.
- Denylist of protected system paths, enforced on every cleanup and trash move.
- Cleaner pipeline with preview, execute, per-item revalidation, and post-move verification.
- Tool-command cleanup targets, run without a shell, with a timeout.
- Remnant detection for uninstalled apps (App Cleanups), with installed-app and running-process checks.
- App Cleanups removal: selected leftovers move to Trash after confirmation. Caution items need an extra checkbox.
- Local snapshot report via `tmutil`.
- Rules engine: TOML schema, validation, and a bundled fallback set.
- Your Data screen and a Homebrew cask `zap` stanza that matches DustMap's own data locations.
- Scan cache: results are saved per volume and reloaded on launch.
- Live scan progress, with cancel.
- CLI subcommands: `scan`, `summary`, `clean`, `rules`, `audit`, `snapshots`, `remnants`, `journal`, `data`.

### Changed
- Headline sizes use allocated (on-disk) bytes. Logical sizes appear as a secondary figure, and sparse files are marked.
- Unified colour tokens and a single type scale across all screens.

### Known gaps
- Permanent deletion is not available. All removals go to the Trash.
- Some screens are not yet pixel-matched to the reference designs.
