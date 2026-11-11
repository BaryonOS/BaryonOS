<a id="beveiligingsbeleid"></a>

# Security policy

Baryon OS is experimental research and development software. There is no supported production release, security SLA or guarantee for daily use with sensitive data.

<a id="ondersteunde-versies"></a>

## Supported versions

| Version | Security status |
|---|---|
| Current `main` | Active development branch; security issues are investigated without a guaranteed response time |
| Older commits and development branches | No structured support |
| Production releases | Not available yet |

A green CI run or successful review does not mean a version is safe for production use.

<a id="een-kwetsbaarheid-privé-melden"></a>
<a id="report-a-vulnerability-privately"></a>

## Reporting a vulnerability

**Do not publish exploitable details in a regular issue, discussion, pull request or log.**

GitHub Private Vulnerability Reporting is enabled for this repository.
Use [Report a vulnerability](https://github.com/BaryonOS/BaryonOS/security/advisories/new)
to report vulnerability details privately to the repository maintainers.
This button is also available on the public
[Security page](https://github.com/BaryonOS/BaryonOS/security).
Signing in to GitHub is required to submit a report.

If the reporting route is unavailable, do not publish vulnerability details elsewhere; retry the private report later.

## Information to include in a private report

Include the following information where known:

- affected component and commit or release;
- minimal reproducible steps and, where possible, a minimal demonstration;
- expected and actual behaviour;
- potential impact and the conditions under which the issue occurs;
- required privileges and attack vector;
- test environment, tools used and relevant logs with secrets and personal data removed;
- whether the issue is already publicly known;
- any proposed fix or temporary mitigation.

Do not include passwords, keys or real user data in the report.

<a id="opvolging-en-openbaarmaking"></a>

## Follow-up and disclosure

Follow-up questions and coordination on a solution and disclosure take place
through the private report. No acknowledgement, assessment or remediation
deadline is guaranteed. Coordinate disclosure of exploitable details with the
repository maintainers through the private report first.

## Testing boundaries

Test only local emulator images or explicitly authorised test hardware. Do not test systems, accounts or infrastructure without permission. Do not use production keys or real user data for reproduction.

## Technical security model

Architecture, threat model, capability boundaries, updates and post-quantum research are documented in [docs/SECURITY-AND-QUANTUM.md](docs/SECURITY-AND-QUANTUM.md) and the specifications under [docs/implementation/](docs/implementation/).

This document defines the **reporting process**; it is not evidence that the described security properties are fully implemented.

<a id="grenzen-en-verdere-toetsing"></a>

## Boundaries and further verification

A private reporting channel is not evidence of system security or a completed
release procedure. Before a public test release, supported versions, triage
and patch/advisory procedures, production signing keys, key management,
recovery from key compromise and external review still need to be established
and verified.
