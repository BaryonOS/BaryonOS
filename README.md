# Baryon OS

**Baryon OS is an experimental, independent operating system with its own x86_64 kernel in Rust, capability-based isolation, and a product model centered on user-controlled material and data.**

> [!WARNING]
> Baryon OS is research and development software. There is no supported production release yet, and the system is not suitable for daily use or sensitive production data.

## Project overview

Baryon OS does not use the Linux, BSD, XNU, Windows, or Android kernel as its foundation. The native chain runs from UEFI/OVMF through a custom Rust loader into a `no_std` kernel, isolated user processes, system services, drivers, an SDK, and native applications.

The technical architecture and product experience are developed in parallel. **Material** (called **Materiaal** in the current implementation) is the data and permission model around documents and collections; applications act as bounded tools around that material. Familiar desktop conventions remain the default for ordinary interaction.

## Core principles

- **Independent kernel and system services.** The kernel, process boundaries, and native services are developed within the project.
- **Explicit authority.** Processes receive bounded capabilities instead of implicit broad access.
- **Isolation and recovery.** A failure in one application or service should not automatically take down the entire session or unrelated processes.
- **User control over data.** Local storage and explicitly connected, client-side encrypted external storage define the intended trust boundary.
- **Evidence-based claims.** Host tests, QEMU evidence, and physical-hardware tests are reported separately; a prototype is not treated as product acceptance.
- **European orientation.** Autonomy, privacy, and user control are design goals without sacrificing normal connected functionality or product quality.

## Architecture

```text
UEFI / OVMF
    ↓
custom Rust loader
    ↓
no_std Rust kernel
    ↓
processes + capabilities + IPC
    ↓
userspace services + drivers
    ↓
SDK + native applications + desktop
```

| Area | Location |
|---|---|
| Loader | `boot/` |
| Kernel | `kernel/` and `crates/kernel-core/` |
| Capability and IPC logic | `crates/capability-core/` |
| Drivers | `drivers/` |
| System services | `services/` |
| SDK and example applications | `sdk/` |
| User processes | `userspace/` |
| Native UI and product prototype | `ui/` |
| Tests and development tooling | `tests/` and `tools/` |

Further reading: [architecture](docs/ARCHITECTURE.md) · [kernel and security contract](docs/implementation/KERNEL-AND-SECURITY-CONTRACT.md) · [product ambition](docs/DESIGN-AMBITION.md).

## Current status

The codebase already contains an independent boot chain, process and capability infrastructure, userspace services, storage and recovery tracks, driver work, an SDK, and native UI/session components. Not every component is fully integrated or suitable for physical hardware yet. Formal acceptance status is determined only by the fixed progress criteria.

<!-- OS-PROGRESS:BEGIN -->
<a id="voortgang-naar-een-volwaardig-zelfstandig-os"></a>
## Progress toward a complete independent OS

![Baryon OS: 39% of the fixed product roadmap](docs/assets/progress.svg)

**39/100 weighted acceptance points.** This is a planning metric for demonstrably accepted functionality, not a time estimate, security guarantee, or claim of parity with existing operating systems.

[Criteria and evidence](docs/PROGRESS.md) · [Roadmap](docs/ROADMAP.md) · [Current project status](docs/PROJECT-STATE.md)
<!-- OS-PROGRESS:END -->

## Build and test

The current development route is documented in [DEVELOPMENT.md](docs/DEVELOPMENT.md). The minimal QEMU path is:

```sh
python tools/preflight.py --strict
python tools/build.py
python tools/run.py --smoke
```

A successful build is not a full acceptance test. See [TEST-STRATEGY.md](docs/TEST-STRATEGY.md) and [docs/verification/](docs/verification/) for test strategy and evidence levels.

## Roadmap and work

The long-term route progresses from independent boot and isolation to storage, native product integration, network security, physical reference hardware, and an experimental community release.

- [Roadmap and technical milestones](docs/ROADMAP.md)
- [Current criteria and score](docs/PROGRESS.md)
- [Current execution status](docs/PROJECT-STATE.md)
- [Issues](https://github.com/simonvanieperen/BaryonOS/issues)
- [Discussions](https://github.com/simonvanieperen/BaryonOS/discussions)
- [GitHub Wiki](https://github.com/simonvanieperen/BaryonOS/wiki)
- [English Wiki source and publication guide](docs/WIKI.md)

Historical development checkpoints are stored under [docs/history/](docs/history/) rather than in this README. The former Dutch README is preserved as a dated [9 November 2026 snapshot](docs/history/README-2026-11-09.nl.md); it is not the current implementation specification.

## Contributing

Contributions are welcome within the existing architecture and security boundaries. Read [CONTRIBUTING.md](CONTRIBUTING.md), the relevant `AGENTS.md` files, and the applicable contract specifications before changing a subsystem.

Pull requests are checked by CI and automated review. A successful automated review does not replace a required native test or independent human security audit.

## Security

Do **not** disclose exploitable vulnerability details in a public issue. See [SECURITY.md](SECURITY.md) for the current reporting policy and [SECURITY-AND-QUANTUM.md](docs/SECURITY-AND-QUANTUM.md) for the technical security model.

## License

Project-owned software listed in [MPL-2.0-FILES.txt](MPL-2.0-FILES.txt) is made available under the **Mozilla Public License 2.0**. See [LICENSE](LICENSE) and [LICENSING.md](LICENSING.md). Third-party code, fonts, and other components retain their own terms; see [THIRD-PARTY.md](THIRD-PARTY.md).

The **Baryon OS** name, project identity, and any trademark rights are not granted by the software license.

<a id="waarom-de-naam-baryon-os"></a>
## Why “Baryon”?

The name refers to baryons as composite particles and uses that composition as a metaphor for a system in which a small kernel, separate services, and a distinct user environment form one coherent whole. The metaphor is not a technical claim of safety or stability.
