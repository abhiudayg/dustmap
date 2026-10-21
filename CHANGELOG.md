# Changelog

All notable changes to DustMap will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.1] - 2026-10-21

### Added
- Permanent deletion from the delete dialog. It is opt-in per item, requires a confirmation checkbox, and is written to the audit log as `deleted_permanently`.
- Batch delete from the Top Space Consumers list: select rows (or all visible rows), then review and delete them in the Confirm Deletion dialog. Failed items stay selected and are reported with their reasons.
- Empty folder contents: removes everything inside a folder while keeping the folder, for folders that cannot be deleted themselves.

### Changed
- The path stream in the scan view is a scrolling box that takes the remaining space, and keeps the last 200 paths with the newest at the bottom.
- The Cancel scan button shows "Cancelling…" as soon as it is clicked.
- The menu bar, splash screen, and exported report read the version from `package.json`.

### Fixed
- Cancelling a scan no longer saves the partial result to the scan cache, which made cancelling slow on large scans.
- Purging a Cache & Logs entry removes it from the list and from its real parent folder.
- The scan view fits short windows; the header, radar, and cards keep their sizes.

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
