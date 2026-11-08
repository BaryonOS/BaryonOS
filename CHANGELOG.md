# Changelog

## 2026-11-09

### Documentation and governance

- The README is reduced to project introduction, architecture overview, status, quick start, and navigation; historical development updates are archived.
- PROJECT-STATE and PROGRESS are separated into current status, criteria register, and historical snapshots.
- CONTRIBUTING and SECURITY now define concrete contribution, merge, and reporting paths.
- The official MPL-2.0 text, licensing scope, covered-file register, and third-party index are added.
- Active core links point to `simonvanieperen/BaryonOS`.

## 2026-11-03

Based on commit range
`ad2b981fe1f2bcccb03cb776b2281d32529d2f00..380111b68688c981eec6abb8fab29174a29f5601`:
14 commits dated 30 October–2 November 2026, inclusive.

### Features

- Materiaal and Calculator were further integrated into the native desktop session, including clipboard support, improved drawing, and recovery from a compositor failure.
- Key state persists across a cold restart; key management gained rotation and revocation.
- An optional real-time clock can supply UTC time to a ring-3 service.
- The native path includes a TPM experiment and an interactive test version of Materiaal. These are development experiments, not release or physical-hardware acceptance.

### Fixes

- Restored native operation of the task selector, proportional pointer positioning, and complete redrawing of dock buttons.
- TCP connection deadlines no longer apply to already-established connections; closed connections are correctly reported as closed.
- Restored reliable handling of NAS operations with uncertain outcomes and isolated native-probe configuration.
- The password field validates input before accepting it and does not echo the entered value to the user interface.

### Maintenance

- Preflight now clearly reports a missing host-test linker.
- CI and verification archives were extended for component integration and native desktop, networking, and storage checks.
- The audio driver can parse bounded PCM format information; this does not establish complete audio-output functionality.

### Breaking changes

- None identified in this commit range.
