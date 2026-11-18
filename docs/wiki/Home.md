# Baryon OS

**Baryon OS is an independent operating system in development, with its own kernel, a familiar desktop and explicit access permissions for apps and services.** This wiki provides an English introduction to the source code, design agreements and verifiable test results.

**Authority pointers reviewed:** 17 November 2026. Current contracts, progress and execution status are maintained in the [repository](https://github.com/BaryonOS/BaryonOS), not copied into this wiki. **Historical code and evidence baseline:** [`6e429de`](https://github.com/BaryonOS/BaryonOS/commit/6e429de98b5c692ccceaa12973818eecb1622062), reviewed on 8 November 2026 and translated into English on 9 November 2026. Historical evidence remains bound to that source; this wiki is not automatically updated.

## Where to start

| Goal | Read |
|---|---|
| Understand what already works and what remains open | [Project status and roadmap](https://github.com/BaryonOS/BaryonOS/wiki/Project-status-and-roadmap) |
| Understand the kernel, services and main source directories | [Architecture and code map](https://github.com/BaryonOS/BaryonOS/wiki/Architecture-and-code-map) |
| Build the project or reproduce an existing probe | [Building and testing](https://github.com/BaryonOS/BaryonOS/wiki/Building-and-testing) |
| Understand the data, AI, NAS and security requirements | [Privacy and security](https://github.com/BaryonOS/BaryonOS/wiki/Privacy-and-security) |
| Prepare a change and deliver a verifiable result | [Development workflow](https://github.com/BaryonOS/BaryonOS/wiki/Development-workflow) |
| Check sources or update this wiki | [Sources and wiki maintenance](https://github.com/BaryonOS/BaryonOS/wiki/Sources-and-wiki-maintenance) |

## Concepts and terminology

- Start here: [What is Baryon OS?](https://github.com/BaryonOS/BaryonOS/wiki/What-is-Baryon-OS) · [Why Baryon?](https://github.com/BaryonOS/BaryonOS/wiki/Why-Baryon) · [FAQ](https://github.com/BaryonOS/BaryonOS/wiki/FAQ)
- Understand the design: [Architecture Overview](https://github.com/BaryonOS/BaryonOS/wiki/Architecture-Overview) · [Material](https://github.com/BaryonOS/BaryonOS/wiki/Material) · [Security Model](https://github.com/BaryonOS/BaryonOS/wiki/Security-Model)
- Follow the direction: [Roadmap Explained](https://github.com/BaryonOS/BaryonOS/wiki/Roadmap-Explained) · [Privacy and Local-first](https://github.com/BaryonOS/BaryonOS/wiki/Privacy-and-Local-first) · [Research Tracks](https://github.com/BaryonOS/BaryonOS/wiki/Research-Tracks)
- Try and contribute: [Build and Run](https://github.com/BaryonOS/BaryonOS/wiki/Build-and-Run) · [Testing and Evidence](https://github.com/BaryonOS/BaryonOS/wiki/Testing-and-Evidence) · [Repository Guide](https://github.com/BaryonOS/BaryonOS/wiki/Repository-Guide) · [Glossary](https://github.com/BaryonOS/BaryonOS/wiki/Glossary)

## Product objective

The project builds an independent OS using its own code. Windows, macOS, Linux, BSD and seL4 do not supply its base kernel. The initial development environment is **x86_64 with UEFI in QEMU**. Rust with `no_std` forms the core; separate processes and bounded permissions isolate components. Development tools and reviewed libraries are permitted. [Project agreements](https://github.com/BaryonOS/BaryonOS/blob/main/AGENTS.md)

Interaction should be familiar: setup, login, desktop, taskbar, search, windows and keyboard shortcuts. **Materiaal** is the data and permissions model underlying documents and tools. An original visual identity and recognisable interaction work together; merely restyling an existing desktop environment does not fulfil the project assignment. [Product direction](https://github.com/BaryonOS/BaryonOS/blob/main/AGENTS.md)

Personal storage is local or, following an explicit choice, encrypted on a paired NAS or with an external storage provider. Built-in system AI must run exclusively on the user's own device. The [binding privacy contract](https://github.com/BaryonOS/BaryonOS/blob/main/docs/LOCAL-PRIVACY-CONTRACT.md) defines these requirements; [Privacy and security](Privacy-and-security) explains their evidence limits.

## Project status

The current score, accepted criteria and evidence are recorded in the [criterion register](https://github.com/BaryonOS/BaryonOS/blob/main/docs/progress.json), under the rules in [PROGRESS.md](https://github.com/BaryonOS/BaryonOS/blob/main/docs/PROGRESS.md). The score is a planning indicator, not elapsed development time or a security percentage. The [current checkpoint](https://github.com/BaryonOS/BaryonOS/blob/main/docs/PROJECT-STATE.md) records execution status; the [release assignment](https://github.com/BaryonOS/BaryonOS/blob/main/docs/FULL-TEST-RELEASE.md) defines full acceptance. This wiki keeps no separate score or status register.

Always read results alongside their source commit, test profile and limitations. Host tests, native QEMU probes, physical-hardware tests and human audits demonstrate different things. Creating or translating this wiki does not run new OS tests or change acceptance points.

## Direct repository links

[Source code](https://github.com/BaryonOS/BaryonOS) · [Current project status](https://github.com/BaryonOS/BaryonOS/blob/main/docs/PROJECT-STATE.md) · [Licensing](https://github.com/BaryonOS/BaryonOS/blob/main/LICENSING.md) · [Security reporting](https://github.com/BaryonOS/BaryonOS/blob/main/SECURITY.md) · [Issues](https://github.com/BaryonOS/BaryonOS/issues) · [Pull requests](https://github.com/BaryonOS/BaryonOS/pulls) · [GitHub Actions](https://github.com/BaryonOS/BaryonOS/actions)
