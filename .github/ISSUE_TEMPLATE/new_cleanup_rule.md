---
name: New Cleanup Rule Request
about: Propose a new cleanup rule for developer caches or application data
title: "[RULE] "
labels: ["cleanup-rule"]
assignees: ''
---

## Tool / Application Name
e.g. Bun, CocoaPods, Android Studio, Rust sccache

## Cache / Target Location
Exact path pattern (e.g. `~/.bun/install/cache`)

## Why is it safe to delete?
Explain how the tool regenerates this data and what happens on next build/run.

## Risk Assessment
- [ ] Safe (regenerable with no user disruption or loss)
- [ ] Review (regenerable but might take network bandwidth/recompilation time)
- [ ] Caution (contains user-generated or configuration data)
- [ ] Report Only (managed by its own CLI, e.g. Docker)

## Official Documentation / Reference Link
Link to tool documentation confirming this cache directory.
