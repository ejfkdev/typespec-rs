# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in typespec-rs, please report it responsibly:

- **Email:** Open a GitHub Security Advisory at https://github.com/ejfkdev/typespec-rs/security/advisories/new
- **Do not** file a public GitHub issue for security vulnerabilities

We will acknowledge your report within 48 hours and aim to provide a fix within 7 days.

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | Yes       |

## Scope

This project is a compiler/parser library. Security vulnerabilities in the context of this project include:

- Memory safety issues (buffer overflows, use-after-free, etc.) in the parser or checker
- Denial-of-service via crafted input (infinite loops, excessive memory allocation)
- Code injection through template expansion or decorator processing

Note: This library does not execute arbitrary code, access the network, or write files outside of what the user explicitly requests.
