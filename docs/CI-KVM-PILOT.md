# Experimental hosted KVM pilot

## Status and acceptance boundary

The KVM workflow is a manual, non-required experiment on the standard
`ubuntu-24.04` GitHub-hosted runner. It does not replace any required check,
release gate, or recorded TCG baseline. Required workflows continue to omit
`NOS_QEMU_ACCEL`, which preserves the exact `tcg,thread=single` command.

A successful pilot is performance and compatibility evidence for the selected
hosted runner image. It is not evidence that KVM is available on every future
runner image, and it is not permission to move acceptance to a persistent or
self-hosted runner.

## Selection contract

Host tools select acceleration through one narrow contract:

- an absent `NOS_QEMU_ACCEL` selects `tcg,thread=single`;
- the exact value `kvm` selects KVM only after `/dev/kvm` is confirmed to be a
  readable and writable character device;
- every other value fails before QEMU starts.

There is no automatic accelerator discovery and no KVM-to-TCG fallback. This
keeps a pilot failure distinguishable from a slower TCG run.

## Pilot scope

The manual matrix runs three existing native gates with otherwise unchanged
machine, CPU, memory, firmware, device, network-isolation, and evidence
contracts:

1. joint native system normal and cold-recovery acceptance;
2. M6e volume acceptance, including all nine mutation and recovery boots;
3. one native window-session clipboard gate using the fault-injection probe.

Every produced QEMU report is checked for a passing verdict and the exact
`-accel kvm` argument. Compact reports are retained independently from the
test verdict; artifact-storage failure cannot convert a failed native gate into
a pass. Manual dispatch can additionally retain the raw images, frames, and
builds for one day without changing the gate result.

## Operation and rollback

The workflow is started explicitly from **Experimental hosted KVM pilot** in
GitHub Actions. A missing or inaccessible `/dev/kvm` is an expected fail-closed
outcome and must not be interpreted as a product regression.

Rollback consists of removing the manual workflow and the opt-in selector.
Because TCG remains the default and all required workflows omit the selector,
rollback does not require a migration of branch-protection checks or evidence
formats.
