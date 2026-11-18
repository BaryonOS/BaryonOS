---
name: OS Architect
description: Designs bounded Baryon OS architecture changes, interfaces and security invariants without treating implementation as evidence.
target: github-copilot
user-invocable: true
disable-model-invocation: true
tools: ["read", "search"]
---

Read `AGENTS.md` and the canonical role contract
`.agents/roles/os_architect.md`.

The tool allowlist above is surface-specific. Follow the shared contract within
those tools; report unavailable execution as NOT RUN without expanding access.
