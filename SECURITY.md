# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.2.x   | :white_check_mark: |
| 0.1.x   | :x:                |

## Reporting a Vulnerability

We take security vulnerabilities seriously. If you discover a security issue, please report it responsibly.

### How to Report

1. **Do NOT** create a public GitHub issue for security vulnerabilities.

2. **Email**: Send a detailed report to the maintainer via GitHub private message or create a private security advisory.

3. **GitHub Security Advisory**: You can also report vulnerabilities through [GitHub's Security Advisory feature](https://github.com/jsleemaster/claude-explorer/security/advisories/new).

### What to Include

Please include the following information in your report:

- Description of the vulnerability
- Steps to reproduce the issue
- Potential impact
- Suggested fix (if any)
- Your contact information for follow-up questions

### Response Timeline

- **Initial Response**: Within 48 hours of receiving the report
- **Status Update**: Within 7 days with our assessment
- **Resolution**: We aim to resolve critical vulnerabilities within 30 days

### After Reporting

- We will acknowledge receipt of your vulnerability report
- We will investigate and validate the issue
- We will work on a fix and coordinate disclosure
- We will credit you in the release notes (unless you prefer to remain anonymous)

### Scope

This security policy applies to:

- The `claude-explorer` binary and its source code
- Dependencies used by the project

### Out of Scope

- Vulnerabilities in Claude Code CLI itself (report to Anthropic)
- Issues in third-party dependencies should be reported to those projects directly, though we appreciate being notified so we can update

## Security Best Practices for Users

- Always download releases from official sources (GitHub releases or crates.io)
- Verify checksums when available
- Keep your installation up to date
- Be cautious when running Claude Explorer in directories with untrusted content

Thank you for helping keep Claude Explorer secure!
