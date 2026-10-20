# Security & Safety Policy

## Reporting Security & Data-Loss Issues

Storage tools touch critical user data. We take security vulnerabilities and unintended data loss with the utmost seriousness.

- **Private Reporting:** Please report security vulnerabilities or data loss privately via GitHub Private Vulnerability Reporting or via email to `security@dustmap.dev`.
- **Response Targets:**
  - Acknowledgment: within 3 business days
  - Triage: within 7 days
  - Remediation plan / patch: within 30 days for high-severity issues

## Data-Loss Incident Response

Any confirmed incident of unintended data loss triggers:
1. Immediate demotion or removal of the offending cleanup rule.
2. An advisory notice pinned to the repository.
3. A public postmortem explaining the root cause and remediation.
4. A regression test added to the safety regression suite before any new release.

## Supported Versions

| Version | Supported |
|---|---|
| 0.1.x | Yes |
