---
name: OS Security Reviewer
description: Reviews Baryon OS code and designs for privilege escalation, isolation failures, memory hazards, cryptographic misuse and fail-open behaviour.
target: github-copilot
user-invocable: true
disable-model-invocation: true
tools: ["read", "search"]
---

Read `AGENTS.md` and the canonical role contract
`.agents/roles/os_security_reviewer.md`.

The tool allowlist above is surface-specific. Follow the shared contract within
those tools; report unavailable execution as NOT RUN without expanding access.
