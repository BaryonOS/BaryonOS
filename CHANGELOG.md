# Changelog

## 2026-11-09

### Documentation and governance

- The README is reduced to project introduction, architecture overview, status, quick start, and navigation; historical development updates are archived.
- PROJECT-STATE and PROGRESS are separated into current status, criteria register, and historical snapshots.
- CONTRIBUTING and SECURITY now define concrete contribution, merge, and reporting paths.
- The official MPL-2.0 text, licensing scope, covered-file register, and third-party index are added.
- Active core links point to `simonvanieperen/BaryonOS`.

## 2026-11-03

Based on the commit range
`ad2b981fe1f2bcccb03cb776b2281d32529d2f00..380111b68688c981eec6abb8fab29174a29f5601`:
14 commits, from 30 October through 2 November 2026. The final commit is included.

### Features

- Materiaal and Calculator (the `Rekenmachine` app) are further integrated into the native window session, including the clipboard, updated rendering and recovery after compositor failure.
- Key state persists across a cold restart; key management now includes rotation and revocation.
- An RTC clock can optionally supply UTC time to a ring-3 service.
- The native path includes a TPM probe and an interactively operated Materiaal test version; these are development probes, not release or hardware acceptance.

### Bug fixes

- Fixed native interaction with the command picker, proportional pointer positions and complete redrawing of dock buttons.
- A TCP connection deadline no longer applies to an already established connection; closed connections are correctly reported as closed.
- Restored reliable handling of NAS operations with uncertain outcomes and isolated the native probe configuration.
- The password field validates input before accepting it and does not return the entered value to the control interface.

### Maintenance

- Preflight now clearly reports a missing host-test linker.
- CI and verification archives have been extended for component integration and native desktop, network and storage checks.
- The sound driver can process bounded PCM format information; this is not evidence of complete audio output.

### Breaking changes

- None identified in this commit series.
