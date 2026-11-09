# Security Policy

Baryon OS is experimental research and development software. There is no supported production release, security SLA, or guarantee for daily use with sensitive data.

## Supported versions

| Version | Security status |
|---|---|
| Current `main` | Active development branch; security issues are investigated without a guaranteed response time |
| Older commits and development branches | No structured support |
| Production releases | Not available yet |

A green CI run or successful review does not mean a version is safe for production use.

## Reporting a vulnerability

**Do not publish exploitable details in a regular issue, discussion, pull request, or log.**

The repository does not currently expose a verifiable private reporting channel through the project configuration available here. Until GitHub Private Vulnerability Reporting or another official private channel is enabled:

1. open only a minimal public issue titled **`Security contact requested`**;
2. include no vulnerability details, exploit code, secrets, personal data, or sensitive logs;
3. wait for a maintainer-designated private channel before sharing technical details.

Enabling a permanent private reporting channel remains a governance requirement before a public test release.

## Information to include in a private report

Once a private channel is available, include:

- affected component and commit or release;
- minimal reproducible steps;
- expected and actual behavior;
- potential impact;
- required privileges and attack vector;
- relevant logs with secrets and personal data removed;
- whether the issue is already publicly known.

## Testing boundaries

Test only local emulator images or explicitly authorized test hardware. Do not test systems, accounts, or infrastructure without permission. Do not use production keys or real user data for reproduction.

## Technical security model

Architecture, threat model, capability boundaries, updates, and post-quantum research are documented in [docs/SECURITY-AND-QUANTUM.md](docs/SECURITY-AND-QUANTUM.md) and the specifications under [docs/implementation/](docs/implementation/).

This document defines the **reporting process**; it is not evidence that the described security properties are fully implemented.
